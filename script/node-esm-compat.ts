#!/usr/bin/env bun

import { readFile, writeFile } from "node:fs/promises";

const filePath = new URL(
  "../.temp/rust-wasm/twsearch_wasm.js",
  import.meta.url,
);

let modified = false; // For idempotence

let contents = await readFile(filePath, "utf-8");
const lines = [
  `// Mangled so that bundlers don't try to inline the source.
const node_fs_promises_mangled = "node:-fs/pr-omises";
const node_fs_promises_unmangled = () => node_fs_promises_mangled.replace(/-/g, "");
`,
];
for (const line of contents.split("\n")) {
  if (line.trim() === "input = fetch(input);") {
    lines.push(`        try {
            input = await fetch(input);
        } catch (e) {
            if (!(e instanceof TypeError)) {
                throw e;
            }
            input = await (await import(node_fs_promises_unmangled())).readFile(input);
        }`);
    modified = true;
  } else if (line.includes("new URL") && line.includes("import.meta.url")) {
    lines.push(
      `throw new Error("Default \`wasm-pack\` WASM loading code path triggered! This is currently not supported for \`twsearch\` due to incompatibility with some bundlers.");`,
    );
  } else {
    lines.push(line);
  }
}
if (modified) {
  contents = lines.join("\n");
  await writeFile(filePath, contents);
}
