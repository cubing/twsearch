import { concat, SHA256, validate } from "./helpers";

export async function derive(
  parentDerivationSeed: ArrayBuffer /* 32 bytes */,
  salt: ArrayBuffer /* any length */,
): Promise<ArrayBuffer /* 32 bytes */> {
  validate(parentDerivationSeed, salt);
  const parentLevel = new DataView(parentDerivationSeed).getUint8(1);

  const hashed_salt = await SHA256(salt);
  const intermediate = new Uint8Array(
    await SHA256(await concat(parentDerivationSeed, hashed_salt)),
  );
  intermediate[0] = 0x67;
  intermediate[1] = parentLevel === 0xff ? parentLevel : parentLevel + 1;

  return intermediate.buffer;
}
