/**
 * Example: Obfuscate a root seed by expanding it into a large text file.
 *
 * Note that this implementation does not perform any error correction or
 * tampering — it only spreads out the information for the root seed in such a
 * way that it is impractical to share in that form using anything other than a
 * direct data transfer.
 *
 */

import { writeFile } from "node:fs/promises";
import { toHex } from "./helpers";

const NUM_SHARDS = 1024;
const ARRAY_BUFFER_LENGTH = 32;

function randomUint8Array(): Uint8Array<ArrayBuffer> {
  const arrayBuffer = new Uint8Array(ARRAY_BUFFER_LENGTH);
  globalThis.crypto.getRandomValues(arrayBuffer);
  return arrayBuffer;
}

function xorUint8Arrays(
  arrays: Uint8Array<ArrayBuffer>[],
  start: Uint8Array<ArrayBuffer> = new Uint8Array(ARRAY_BUFFER_LENGTH),
): Uint8Array<ArrayBuffer> {
  const remainder = new Uint8Array(start); // Create a copy to avoid modifying the input.
  for (const array of arrays) {
    for (let i = 0; i < ARRAY_BUFFER_LENGTH; i++) {
      remainder[i] ^= array[i];
    }
  }
  return remainder;
}

function toShards(v: Uint8Array<ArrayBuffer>): Uint8Array<ArrayBuffer>[] {
  const shards = new Array(NUM_SHARDS - 1).fill(0).map(randomUint8Array);
  shards.push(xorUint8Arrays(shards, v));
  return shards;
}

function fromShards(v: Uint8Array<ArrayBuffer>[]): Uint8Array<ArrayBuffer> {
  return xorUint8Arrays(v);
}

const rootSeed = new Uint8Array(ARRAY_BUFFER_LENGTH);
globalThis.crypto.getRandomValues(rootSeed);
rootSeed[0] = 0x67;
rootSeed[1] = 0x00;

console.log(`Original root seed: ${rootSeed.toHex()}`);
console.log(rootSeed.toHex());

const shards = toShards(rootSeed);
// This following output can be written to a 67KB file:
console.log(shards.map(toHex).join("\n"));
await writeFile("text.txt", shards.map(toHex).join("\n"));

// Reconstruct the root seed.
console.log(`Reconstructed root seed: ${fromShards(shards).toHex()}`);
