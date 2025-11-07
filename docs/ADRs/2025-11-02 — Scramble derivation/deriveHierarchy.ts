/**
 * Run using `bun` (https://bun.sh/):
 *
 *     bun run deriveHierarchy.ts
 *
 */

import { derive } from "./derive";
import { expectEqual, fromASCII, fromHex } from "./helpers";

export async function deriveHierarchy(
  parentDerivationSeed: ArrayBuffer,
  saltHierarchy: ArrayBuffer[],
): Promise<ArrayBuffer> {
  let derivationSeed = parentDerivationSeed;
  for (const salt of saltHierarchy) {
    derivationSeed = await derive(derivationSeed, salt);
  }
  return derivationSeed;
}

const rootSeed = fromHex(
  "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
);
const roundSeed = await deriveHierarchy(rootSeed, [
  fromASCII("EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF"), // auditor salt
  fromASCII("scrambles"),
  fromASCII("333"),
  fromASCII("r1"),
  fromASCII("g1"),
  fromASCII("a1"),
  fromASCII("333"),
  fromASCII("sub1"),
  fromASCII("candidate1"),
]);

expectEqual(
  roundSeed,
  fromHex("67090777cf85e259361b2035023b0fbbbb478cc38c5d174926509ee82ec0431b"),
);
