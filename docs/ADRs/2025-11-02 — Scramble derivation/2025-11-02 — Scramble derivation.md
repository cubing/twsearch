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

Fo example, a typical root derivation seed for a competition would look like this:

```
67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67
```

This seed should be generated using a commitment scheme that:

- ensures confidentiality until the scrambles are used
- creates a way to verify the root seed against the scrambles used at the competition, and
- allows a way for the WST to perform a multi-party ceremony to recover the seed in case of an emergency.

TODO: specify this commitment scheme either here or in a separate spec.

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
| 1 | Event ID | `"333"`, `"333mbf"`, `"unofficial"-guildford` |
| 2 | Round ID (per [WCIF](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#round)) | `"r1"` |
| 3 | Group ID (part of a [WCIF activity code](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#ActivityCode)) | `"g1"` |
| 4 | Attempt ID (generalization of part of a [WCIF activity code](https://github.com/thewca/wcif/blob/d8321491178738a62c62f7c4f9ae7cd3f340a4ea/specification.md#ActivityCode)) | `"a1"`, `"e1"`, `"a1e1"` |
| 5 | Subevent ID | `"333"` |
| 6 | Subevent scramble salt | `"sub1"`, `"sub1e1"` |
| 7 | Scramble filtering candidate salt | `"candidate1"` |

Note that:

- All ID and salt counters start with `1` (not `0`), matching the WCIF.
- It is possible to generate an extra scramble for a round that is not bound to a specific scramble (`"e1"`), or one that is for a specific attempt (`"a1e1"`). The former allows scrambles to be generated and used (e.g. printed) the same way as they were with TNoodle, and the latter allows for innovation in scramble accountability and recording mechanisms.
  - It is additionally possible to generate extra scrambles within an attempt, e.g. `"sub1e1"` for 3×3×3 Multi-Blind.
- Some events (`"333"`, `"pyram"`, `"unofficial-tetraminx"`) produce a single scramble alg. We will call these "monoscramble" events. (Note that such a scramble will usually correspond to a single "physical" puzzle, but this may not be true in general.) For these events:
  - The subevent ID (level 5) is the event ID.
  - The subevent scramble salt (level 6) is always `"sub1"`. (Note that **this level of the hierarchy is not skipped**, even though it is a fixed salt for the event. To makes it easier to implement correctly.)
- Some events (3×3×3 Multi-Blind, Mini Guildford) contains multiple subevents or multiple subscrambles for a single subevent. In this case:
  - The scramble specification for each such event must define a list of subevents.
  - Each subevent ID (level 5) must be a monoscramble event ID.
  - The number of scrambles for each subevent may be fixed for the event or unspecified. (If it is unspecified, it is expected to be part of the result.)
- To generate a scramble, level-7 "candidate seeds" are derived from the level-6 seed until one is accepted:
  - For `candidateSalt` taking the values `candidate1`, `candidate2`, `candidate3` etc. :
    - Compute `candidateSeed = derive(level6Seed, candidateSalt)`.
    - Use `candidateSeed` to deterministically generate a pseudorandom scramble pattern (e.g. 3×3×3) or scramble alg (e.g. Megaminx). This will be an event-tailored implementation whose selection is based on fairness and practicality (TODO: link to a separate specification for this).
    - Check if the generated pattern/alg passes a scramble filtering check (TODO: link to a separate specification for this).
    - If it passes, use this pattern/alg. Else, continue to the next candidate.

See [`scrambleSpecifications.ts`](scrambleSpecifications.ts) for a rough example of how event information can be stored in an implementation-agnostic way.

The seed at a given hierarchy path is calculated by performing a derivation with the salt in each hierarchy recursively. For example, here is a calculation of the derivation seed for the first round of 3×3×3 Speed Solving using the example root seed from above:

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

const competitonSeed = fromHex("67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67");
const roundSeed = await deriveHierarchy(competitonSeed, [
  fromASCII("333"),
  fromASCII("r1"),
]);

expectEqual(roundSeed, fromHex("6702bc57ffee6c047ce99e9d8daf63eebee585eb385403d9ce17ccb864abf84b"));
```

Associated file: [`deriveHierarchy.ts`](./deriveHierarchy.ts)
