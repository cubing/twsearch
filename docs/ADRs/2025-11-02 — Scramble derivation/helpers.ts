#!/usr/bin/env -S bun run --

export const SHA256 = (v: ArrayBuffer) =>
  globalThis.crypto.subtle.digest("SHA-256", v);
export const concat = (a: ArrayBuffer, b: ArrayBuffer) =>
  new Blob([a, b]).arrayBuffer();

export const fromHex = (s: string) => Uint8Array.fromHex(s).buffer;
export const fromASCII = (s: string) => new TextEncoder().encode(s).buffer;

export function validate(
  parentDerivationSeed: ArrayBuffer /* 32 bytes */,
  salt: ArrayBuffer /* any length */,
) {
  if (parentDerivationSeed.byteLength !== 32) {
    throw new Error("Invalid parent derivation seed byte length.");
  }

  if (new DataView(parentDerivationSeed).getUint8(0) !== 0x67) {
    throw new Error("Invalid protocol sentinel.");
  }

  for (const byte of new Uint8Array(salt)) {
    if (byte > 0x7f) {
      throw new Error("Salt is not ASCII.");
    }
  }
}

export function expectEqual(observed: ArrayBuffer, expected: ArrayBuffer) {
  if (observed.byteLength !== expected.byteLength) {
    throw new Error("❌ Mismatching buffer lengths.");
  }
  const observedView = new DataView(observed);
  const expectedView = new DataView(expected);

  for (let i = 0; i < observed.byteLength; i++) {
    if (observedView.getUint8(i) !== expectedView.getUint8(i)) {
      throw new Error(`❌ Buffers mismatch at byte index: ${i}

Observed: ${new Uint8Array(observed).toHex()}
Expected: ${new Uint8Array(expected).toHex()}
        `);
    }
  }

  console.log("✅ Equal");
}
