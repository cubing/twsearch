#!/usr/bin/env bun

import { readFile, writeFile } from "node:fs/promises";

const filePath = new URL("../.temp/rust-wasm/twips_wasm.js", import.meta.url);

let modified = false; // For idempotence

let contents = await readFile(filePath, "utf-8");
const lines = [];
for (const line of contents.split("\n")) {
  if (line.trim() === "input = fetch(input);") {
    lines.push(`        try {
            input = await fetch(input);
        } catch (e) {
            if (!(e instanceof TypeError)) {
                throw e;
            }
            input = await (await globalThis.process.getBuiltinModule("node:fs/promises")).readFile(input);
        }`);
    modified = true;
  } else if (line.includes("new URL") && line.includes("import.meta.url")) {
    lines.push(
      `throw new Error("Default \`wasm-pack\` WASM loading code path triggered! This is currently not supported for \`twips\` due to incompatibility with some bundlers.");`,
    );
  } else {
    lines.push(line);
  }
}
if (modified) {
  contents = lines.join("\n");
  await writeFile(filePath, contents);
}
