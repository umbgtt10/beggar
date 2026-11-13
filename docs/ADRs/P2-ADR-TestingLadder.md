# P2-ADR-TestingLadder

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P2 (structural)

## Context

One style of test catches one class of failure and misses the others. The
vectors prove the specification's samples; they say nothing about a command
sequence nobody wrote down. A property test explores sequences; it cannot
enumerate every situation. Neither drives two registries whose reports to one
another are lost, duplicated, reordered or withheld. Without named tiers, a command lands with
one kind of test and skips the rest.

## Decision

Beggar's correctness rests on a fixed ladder of seven tiers, each with an
obligation of its own, all run by the gates: unit, conformance, exhaustive
matrix, property, cluster simulation, replay and memory.

## Forcing constraints / Evidence

Each tier is the only one that catches its class of failure:

| tier | proves | catches |
|---|---|---|
| 1. unit | each type alone, one test file per source file, the crate's and the specification's model's | a type wrong in isolation |
| 2. conformance | every vector, run against the crate and against the specification's model, every vector valid against the schema, every row of the matrix exercised — and, by the model, decided | a departure from the specification, or a vector that departs from it |
| 3. exhaustive matrix | every command in every situation over small bounds — `chunks` up to 3, every run within, across and beyond them — against the naive model | a situation no vector names |
| 4. property | the invariants after every call, receipts one more on every call, the crate against the specification's model over random sequences | a sequence nobody wrote down |
| 5. cluster simulation | several registries whose reports to one another a seeded harness loses, duplicates, reorders and withholds — pull, push, cancels, restarts under a new incarnation | what happens between two ends |
| 6. replay | the property and cluster simulation tiers' logs written in the specification's JSON and replayed into a fresh instance: every receipt matches, a dropped line is detected, the replay resynchronises from a checkpoint | a log that cannot rebuild the state |
| 7. memory | a counting allocator in a test target of its own: a positive control, then peak and residue over 100 recoveries | growth no other tier sees |

## Rejected alternatives

**The vectors alone.** Rejected: they are samples, however complete their
coverage of the matrix's rows.

**A system tier**, as `faction` has. Not applicable: beggar has no transport,
no socket and no process. Its system tier is its consumers' — IBFT's and
Raft's system tests, once it is integrated.

**Tiers without names.** Rejected: coverage gaps drift in silently.

## Consequences

A command lands in every tier that applies to it, in the slice that brings it:
a unit test for each new type, its rows in the vectors, its situations in the
exhaustive matrix, its situations and changes in the specification's model
([P1-ADR-IndependentModelOfTheSpecification](P1-ADR-IndependentModelOfTheSpecification.md)), and a cluster simulation scenario and
a replay where it changes what passes between two ends.

The ladder is slower than unit tests alone; tier 3's bounds keep it exhaustive
and fast, and tier 5 is seeded, so a failure replays.

## Enforcement

Stage 1 runs every tier: the unit tier in `crates/core/tests/` and
`crates/spec-model/tests/`, mirrored file by file, and the scenario tiers in `validation/ladder/`, a package of their own (decided
2026-11-09), where stern's `paired-test-file` stands down because they are
named for what they prove, not for a source file. The memory tier needs a
binary of its own, a global allocator being per binary, and is `validation/memory/`, a
package of its own (decided 2026-11-10) whose instrument and run are its source
and mirrored by its tests. The cluster simulation is `validation/core-validation/`, laid
out as faction's (decided 2026-11-10): its harness is its source, its tests are
named for what they prove and skip `paired-test-file` as the ladder's do, and
the cross-node invariants are a type of their own, each check shown firing on a
crafted violation, so a checker that never fires cannot pass for one that
holds. Stage 2's coverage gate measures the unit and
scenario tiers together and the conformance tier alone
([P1-ADR-NoLineTheSpecificationDoesNotAskFor](P1-ADR-NoLineTheSpecificationDoesNotAskFor.md)).

## Related

`faction`'s `P2-ADR-TestingLadder` — the ladder this one follows, tier for tier
where beggar has the same thing to prove.
