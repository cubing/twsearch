#!/usr/bin/env bun

// TOOD: remove this once https://github.com/oven-sh/bun/issues/5846 is implemented.
// TODO: turn this into a package?

import { exit } from "node:process";
import { satisfies } from "compare-versions";
import { Path } from "path-class";
import { PrintableShellCommand } from "printable-shell-command";

const { engines } = await new Path("./package.json").readJSON<{
  engines: Record<string, string>;
}>();

let exitCode = 0;

async function checkEngine(
  engineID: string,
  versionCommand: PrintableShellCommand,
  options?: { trimPrefix?: string },
) {
  const engineRequirement = engines[engineID];

  let engineVersion: string;
  try {
    engineVersion = (await versionCommand.stdout().text()).trim();
  } catch (_) {
    console.error(
      `Command failed while getting version:

  ${versionCommand.getPrintableCommand({ mainIndentation: "    " })}`,
    );
    exitCode = 1;
    return;
  }
  if (options?.trimPrefix) {
    if (!engineVersion.startsWith(options.trimPrefix)) {
      throw new Error(
        "Version command output does not start with the expected prefix.",
      );
    }
    engineVersion = engineVersion.slice(options.trimPrefix.length);
  }

  if (!satisfies(engineVersion, engineRequirement)) {
    console.error(
      `Current version of \`${engineID}\` is out of date: ${engineVersion}`,
    );
    console.error(`Version of \`${engineID}\` required: ${engineRequirement}`);
    exitCode = 1;
    return;
  }
}

async function checkEngines(): Promise<void> {
  await Promise.all([
    checkEngine("bun", new PrintableShellCommand("bun", ["--version"])),
    checkEngine("node", new PrintableShellCommand("node", ["--version"])),
    checkEngine("rv", new PrintableShellCommand("rv", ["--version"]), {
      trimPrefix: "rv ",
    }),
  ]);
}

await checkEngines();
exit(exitCode);
