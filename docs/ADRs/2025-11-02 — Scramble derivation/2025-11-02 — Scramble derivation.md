# Scramble derivation

- Author: Lucas Garron
- Latest update: 2025-11-06

## Goal

A mechanism that allows generating unpredictable scrambles from a single competition-wide seed, while ensuring that the generated scrambles are tamper-proof. In particular, this must make it possible to:

- Derive scrambles for an entire competition from a root seed.
- Derive a sub-seed that is scoped to useful parts of a competition. For example
  - Derive a sub-seed that can be used to derive all scrambles for the 3x3x3 Blindfolded, but not for any other events.
  - Derive a sub-seed that can be used to derive all scrambles for given round but no other rounds.
  - …
- Generate entire extra attempts or extra scrambles for a given attempt.
- Support known official and popular unofficial events without hacks.
- Add new accountability features and ergonomic conveniences, without giving up on any current scramble accountability requirements.
- Audit scramble program implementations for compliance without advanced solving algorithms.

## Decision

### Derivation seeds

Derivation will involve *32-byte derivation seed* values as follows:

| Byte index | Role                                     |
| ---------- | ---------------------------------------- |
| 0          | Static protocol sentinel (`0x67`)        |
| 1          | Derivation level (`0x00` for root seeds) |
| 2-31       | Pseudo-random bytes                      |

Fo example, a typical root derivation seed would look like this:

```text
67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67
```

The protocol to generate a competition derivation seed is described in a section below.

Given a `parentDerivationSeed` and an ASCII byte string `salt` (of any length), a new derivation seed is calculated per the following algorithm, expressed as a JavaScript reference implementation:

```ts
import { concat, SHA256, validate } from "./helpers";

async function derive(parentDerivationSeed, salt) {
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

```

Associated file: [`derive.ts`](./derive.ts)

### Salt hierarchy

A given scramble pattern is derived from a *salt hierarchy*, which is a list of salts where each salt is an ASCII byte string of arbitrary length. For the initial protocol, the hierarchy is the following list:

| Level | Meaning | Example values |
|-|-|-|
| 1 | Auditor salt (see below) | `"X]LJ…@VKK"` (64 ASCII characters) |
| 2 | Purpose | `"scrambles"` |
| 3 | Event ID | `"333"`, `"333mbf"`, `"unofficial-guildford"` |
| 4 | Round ID (per [WCIF](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#round)) | `"r1"`, `"r2"` |
| 5 | Group ID (part of a [WCIF activity code](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#ActivityCode)) | `"g1"`, `"g2"` |
| 6 | Attempt ID (generalization of part of a [WCIF activity code](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#ActivityCode)) | `"a1"`, `"e1"`, `"a1e1"` |
| 7 | Subevent ID | `"333"`, `"222"` |
| 8 | Subevent scramble salt | `"sub1"`, `"sub1e1"` |
| 9 | Scramble filtering candidate salt | `"candidate1"`, `"candidate2"` |

Note that:

- All ID and salt counters start with `1` (not `0`), matching the WCIF.
- It is possible to generate an extra scramble for a round that is not bound to a specific scramble (`"e1"`), or one that is for a specific attempt (`"a1e1"`). The former allows scrambles to be generated and used (e.g. printed) the same way as they were with TNoodle, and the latter allows for innovation in scramble accountability and recording mechanisms.
  - It is additionally possible to generate extra scrambles within an attempt, e.g. `"sub1e1"` for 3×3×3 Multi-Blind.
- Some events (`"333"`, `"pyram"`, `"unofficial-tetraminx"`) produce a single scramble alg. We will call these "monoscramble" events. (Note that such a scramble will usually correspond to a single "physical" puzzle, but this may not be true in general.) For these events:
  - The subevent ID (level 7) is the event ID.
  - The subevent scramble salt (level 8) is always `"sub1"`.
  - Note that **all 9 levels of the hierarchy are still used**. This makes it easier for implementations ot be simple and correct.
- Some events (3×3×3 Multi-Blind, Mini Guildford) contain multiple subevents or multiple subscrambles for a single subevent. In this case:
  - A scramble specification for each such event must define a list of subevents.
  - Each subevent ID (level 7) must be a monoscramble event ID.
  - The number of scrambles for each subevent may be fixed for the event or unspecified. (If it is unspecified, it is expected to be part of the result.)
- To generate a scramble, level-9 "candidate seeds" are derived from the level-8 seed until one is accepted:
  - For `candidateSalt` taking the values `candidate1`, `candidate2`, `candidate3` etc. :
    - Compute `candidateSeed = derive(level6Seed, candidateSalt)`.
    - Use `candidateSeed` to deterministically generate a pseudorandom scramble pattern (e.g. 3×3×3) or scramble alg (e.g. Megaminx). This will be an event-tailored implementation whose selection is based on fairness and practicality (TODO: link to a separate specification for this).
    - Check if the generated pattern/alg passes a scramble filtering check (TODO: link to a separate specification for this).
    - If it passes, use this pattern/alg. Else, continue to the next candidate.

See [`scrambleSpecifications.ts`](scrambleSpecifications.ts) for a rough example of how event information can be stored in an implementation-agnostic way.

The seed at a given hierarchy path is calculated by performing a derivation with each salt in the hierarchy recursively. For example, here is a calculation of a scramble candidate seed using the example root seed from above:

```ts
import { derive } from "./derive";
import { expectEqual, fromASCII, fromHex } from "./helpers";

export async function deriveHierarchy(parentDerivationSeed, saltHierarchy) {
  let derivationSeed = parentDerivationSeed;
  for (const salt of saltHierarchy) {
    derivationSeed = await derive(derivationSeed, salt);
  }
  return derivationSeed;
}

const rootSeed = fromHex("67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67");
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

expectEqual(roundSeed, fromHex("67090777cf85e259361b2035023b0fbbbb478cc38c5d174926509ee82ec0431b"));
```

Associated file: [`deriveHierarchy.ts`](./deriveHierarchy.ts)

### Generating a competition derivation seed

This is a protocol between two parties:

- The generator (e.g. code on a Delegate's computer)
- The auditor (e.g. a WCA server)

Steps:

1. The generator generates a random `rootSeed` by concatenating `0x6700` and 30 (cryptographically strong) random bytes.
2. The generator computes `generatorCommitmentHash = derive(rootCompetitionSeed, "commitment")`
3. The generator sends `generatorCommitmentHash` to the auditor.
4. The auditor generates a 64-byte `auditorSalt`, where each byte is independently generated by concatenating `0x0100` with a (cryptographically strong) random [nybble](https://en.wikipedia.org/wiki/Nibble#History).
    - Note: this allows the auditor salt to be safely treated as an ASCII string that contains only printable characters (which is easily serialized), while keeping 256 bits of entropy.
    - The auditor may wish to sign and/or publish this salt along with the `generatorCommitmentHash` and a timestamp, to provide a ["not before"](https://en.wikipedia.org/wiki/Public_key_certificate#Common_fields) certification for when the generator gained the ability to derive the `competitionSeed`. That is outside the scope of this specification.
5. The auditor sends `auditorSalt` to the generator.
6. The generator computes `competitionSeed = derive(rootSeed, auditorSalt)`

This has the following properties:

- None of the values sent over the network are secret. In fact, it is possible to make them completely public for greater transparency. (For example, the auditor could publish a transparency log of protocol exchanges.)
- Assuming the auditor generates a random auditor salt:
  - The generator has [negligible advantage](https://en.wikipedia.org/wiki/Advantage_(cryptography)) in predicting or influencing the competition seed, and therefore any of the generated scrambles.
- Assuming the generator has generated `rootSeed` properly:
  - Given only the `generatorCommitmentHash`, even an adversarial auditor has negligible advantage in determining the `rootSeed`, `competitionSeed`, or any seeds/scrambles derived from these.
    - The only control the auditor has is to try to give a "bad" `auditorSalt` value instead of a random one. But this is not enough to influence or predict scrambles with any advantage.
- The generator can reveal the `rootSeed` in the future, at which point the auditor can verify that the computation of the commitment hash and the competition seed (and any derived scrambles) are correct.

Note that it would be possible for the auditor to compute `auditorSalt` earlier and send a commitment of it to the generator before receiving `generatorCommitmentHash`, but this is not necessary for
the threat model. (Note that this would also affect the properties of a transparency log.)

The generator must save and protect the secrecy of `rootSeed`, else they will be unable to prove that they used the correct `competitionSeed` (or any derived scrambles). To this end:

- Implementations may want to store the `rootSeed` in a file that is easy to copy and process, but more difficult to exfiltrate without intentional cooperation of the party in possession of it (e.g. by [shoulder surfing](https://en.wikipedia.org/wiki/Shoulder_surfing_(computer_security)) or accidental copy-pasted text). This can be implemented by storing the root seed across a file that is several in size KB, which is easy to share by direct file copy but not practical to share otherwise. See [`obfuscateRootSeed.ts`](./obfuscateRootSeed.ts) for an example implementation.
- At the end of the protocol, implementations may allow the generator to send [threshold encrypted](https://en.wikipedia.org/wiki/Threshold_cryptosystem) ciphertexts to multiple parties associated with the auditor (e.g. WST members), such that a certain number of these parties can recover the `rootSeed` in an emergency ceremony.
