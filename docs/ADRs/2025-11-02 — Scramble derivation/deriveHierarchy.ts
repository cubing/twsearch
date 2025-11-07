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

const competitonSeed = fromHex(
  "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
);
const roundSeed = await deriveHierarchy(competitonSeed, [
  fromASCII("EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF"), // server salt
  fromASCII("333"),
  fromASCII("r1"),
]);

expectEqual(
  roundSeed,
  fromHex("6703928015c02a7a58cf784afb2b44264a1d19a7eb505eb1e868ca0d6f511671"),
);
