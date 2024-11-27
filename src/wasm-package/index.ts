import { Alg } from "cubing/alg";
import type { KPattern, KPuzzleDefinition } from "cubing/kpuzzle";
import {
  default as init,
  wasmRandomScrambleForEvent as rawWasmRandomScrambleForEvent,
  wasmTwsearch as rawWasmTwsearch,
} from "../../.temp/rust-wasm/twsearch_wasm";

let cachedInitWrapper: Promise<void> | undefined;
async function initWrapper(): Promise<void> {
  // biome-ignore lint/suspicious/noAssignInExpressions: This is a caching pattern.
  await (cachedInitWrapper ??= (async () => {
    // TODO: keep this as a `.wasm` file (instead of embedding it in JS) as soon as `esbuild` supports it out of the box.
    // - https://github.com/evanw/esbuild/issues/795
    // - https://github.com/evanw/esbuild/issues/2866
    const wasmUint8Array = (
      await import("../../.temp/rust-wasm/twsearch_wasm_bg.wasm")
    ).default as unknown as Uint8Array;
    await init(wasmUint8Array.buffer as BufferSource);
  })());
}

export async function wasmRandomScrambleForEvent(
  eventId: string,
): Promise<Alg> {
  await initWrapper();
  return new Alg(rawWasmRandomScrambleForEvent(eventId));
}

export async function wasmTwsearch(
  kpuzzleDefinition: KPuzzleDefinition,
  searchPattern: KPattern,
  options?: { minDepth?: number },
): Promise<Alg> {
  await initWrapper();
  return new Alg(
    rawWasmTwsearch(
      JSON.stringify(kpuzzleDefinition),
      // biome-ignore lint/complexity/useLiteralKeys: JSON field access
      JSON.stringify(searchPattern.toJSON()["patternData"]),
      JSON.stringify(options),
    ),
  );
}
