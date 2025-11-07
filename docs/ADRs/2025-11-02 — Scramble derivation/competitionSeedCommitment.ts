import { derive } from "./derive";
import { fromASCII, toHex } from "./helpers";

function randomServerSaltChar(): string {
  const byte = new Uint8Array(1);
  globalThis.crypto.getRandomValues(byte);
  return String.fromCharCode(0b01000000 | (byte[0] & 0b00001111));
}

function randomServerSalt(): string {
  return new Array(64).fill("").map(randomServerSaltChar).join("");
}

class MockAuditor {
  public readonly generatorCommitmentHash: ArrayBuffer;
  public readonly serverSaltASCII: string;
  constructor(commitmentHash: ArrayBuffer) {
    this.generatorCommitmentHash = commitmentHash;
    this.serverSaltASCII = randomServerSalt();
  }
}

const uncommittedRootSeed = new Uint8Array(32);
globalThis.crypto.getRandomValues(uncommittedRootSeed);
uncommittedRootSeed[0] = 0x67;
uncommittedRootSeed[1] = 0x00;

const commitmentHash = await derive(
  uncommittedRootSeed.buffer,
  fromASCII("commitment"),
);
console.log("Commitment hash: ", toHex(commitmentHash));

const server = new MockAuditor(commitmentHash);
console.log("Server salt: ", server.serverSaltASCII);
const competitionSeed = await derive(
  uncommittedRootSeed.buffer,
  fromASCII(server.serverSaltASCII),
);

console.log(`CompetitionSeed: ${new Uint8Array(competitionSeed).toHex()}`);
