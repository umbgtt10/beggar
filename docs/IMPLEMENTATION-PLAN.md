# beggar — implementation plan

**Status: planned 2026-11-06; nothing built.** The decisions are the ADRs in [`ADRs/`](ADRs/README.md) and the
how is [`DESIGN.md`](DESIGN.md); this is the order of building it.

## Vertical slices

Each slice is a working registry, end to end, and each augments the one before it: never a layer built alone —
the data structure, then the vocabulary, then the tests.

**A slice brings a command with all of its rows.** `process` is total
([`P1-ADR-OneEntryPointOneOutput`](ADRs/P1-ADR-OneEntryPointOneOutput.md)): every command has one output in every
situation, so a command cannot arrive with its happy path and leave its refusals for later. Slices are cut by
command, and each is total over the commands it has.

### Every slice

- **Is vertical:** the matrix rows it adds, their JSON vectors written first, the code, the property tests for the
  invariants it touches, the memory test wherever a record is made or removed, and the documents saying what is
  built.
- **Is test-first:** vectors and tests are seen failing before the code that passes them.
- **Ends usable and releasable:** both gates green, and the crate documentation and the README saying what the
  crate does so far. When to publish is the owner's call.
- **Grows what is there:** the vectors, the random-sequence property test and the memory test take each new
  command; nothing is rebuilt.
- **Lands in every tier that applies** ([`P2-ADR-TestingLadder`](ADRs/P2-ADR-TestingLadder.md)), and keeps
  coverage whole: no uncovered line, region or function, over all tiers and over the vectors alone
  ([`P1-ADR-NoLineTheSpecificationDoesNotAskFor`](ADRs/P1-ADR-NoLineTheSpecificationDoesNotAskFor.md)).
- **Grows the specification first:** its rows in `spec/matrix.json`, its types in the schema, its meaning in
  `spec/SPEC.md`, its vectors — then the code.

## The slices

### 1. Open a recovery and see it

- **Adds:** `Beggar<R>` and its receipts, numbered from 1 on every call; `StartRecovery` with all its rows —
  `NoChunks`, `SelfAddressed`, `NotAParty`, and `RecoveryStarted` under a new name minted after the instance id;
  and `ShowRegistry`, the checkpoint.
- **Lays down the specification** in `spec/`, at the repository root and outside the crate: the JSON Schema, the
  decision table `matrix.json`, `SPEC.md`, and the vectors, every u64 a string of decimal digits
  ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).
  A vector is an initial state, steps of a command and its expected receipt, and a final state, read through
  `ShowRegistry`: the specification checks state the way a caller would.
- **Lays down the ladder's first four tiers:** unit; conformance, whose runner in `core/tests/` maps the JSON to
  beggar's types through a model of its own — so the crate never needs `serde` — and checks every vector against
  the schema and every row of the matrix against the vectors; exhaustive matrix; and property, with the naive
  model. And the coverage gate in stage 2, over every tier and over the conformance tier alone.
- **Lays down both origins:** an instance created fresh or from a checkpoint, the checkpoint's round trip proved
  — `ShowRegistry`, restore, `ShowRegistry` — so vectors can start from the state they need.
- **Proves:** names unique across instances and within one, and receipt numbers one more on every call, refused
  calls included. The dummy goes.
- **A caller can then:** open recoveries under names unique across the cluster, be refused when a command is
  wrong, and log the checkpoint.

### 2. Receive it to completion

Pull: the node behind opens the session and receives.

- **Adds:** `ReceiveRecoveryPartial` with all its rows — not started, misattributed, wrong side, out of range,
  advanced (with received and expected), completed, and a duplicate, unaffected — and the received set, kept as
  ranges, in any order. Completion evicts.
- **Proves:** the invariants on the chunks received; completion exactly when every distinct chunk has arrived;
  no record after it. The memory tier arrives: its own test target, a counting allocator, a positive control, then
  100 recoveries in sequence, in order and interleaved. So does the replay tier: the property tier's logs written
  in the specification's JSON and replayed, a dropped line detected, the replay resynchronised from a checkpoint.
- **A caller can then:** account a whole recovery on the node that asked for it — IBFT's request for blocks —
  duplicates, strays and garbage included.

### 3. The serving end

- **Adds:** `AdoptRecovery` with its rows, sharing `StartRecovery`'s checks of a well-formed session, then not
  open, open with the same parameters, open with different ones; and `SendRecoveryPartial` with all its rows.
- **Proves:** a host test of two registries joining both ends of a transfer. Pull: B serves A — start on A, adopt
  on B, sends on B, receives on A, both complete. Push: the sender starts it and the receiver adopts. The
  simulation tier arrives: several registries over a seeded fake network that loses, duplicates and reorders, and
  the replay tier takes its logs.
- **A caller can then:** account both ends of a transfer, pulled or pushed, for IBFT or for Raft.

### 4. Abandon it

- **Adds:** `CancelRecovery<R>`, generic over the caller's enum: cancelled and evicted when open, unaffected when
  not. A cancel wins its race with the final partial.
- **Proves:** a late duplicate adopt reopening a finished session, as the ADR states; the memory test covers
  cancelled sessions.
- **A caller can then:** act on its cancellation policy.

### 5. Look inside, and keep what finished

- **Adds:** `ListRecoveries`, `ShowRecovery` and `ListMissingChunks`; and retention, recorded in two columns —
  `Default` and deployed — from its first line.
- **Proves:** the vectors run at retention 0 and at a high retention, and every output but a query's matches.
- **A caller can then:** know which chunks to resend, and keep finished sessions on a host for analysis.

### The matrix, slice by slice

| row of [`DESIGN.md`](DESIGN.md) §4 | slice |
|---|---|
| `NoChunks`, `SelfAddressed`, `NotAParty` | 1 for a start, 3 for an adopt |
| `StartRecovery` started | 1 |
| `AdoptRecovery`: not open, same, different | 3 |
| a partial: not open, misattributed, wrong side, out of range, advanced, completed, duplicate | 2 for a receive, 3 for a send |
| `CancelRecovery`: open, not open | 4 |
| `ShowRegistry` | 1 |
| `ListRecoveries`, `ShowRecovery`, `ListMissingChunks`, and not started for a session neither open nor retained | 5 |

## Decided

- **The entry point is `Beggar<R>`**, `R` the caller's cancel-reason enum.
- **Beggar numbers every call:** `process(&mut self, command) -> Receipt<R>`, the receipt carrying the output and
  its number. No ordinal comes from the caller
  ([`P1-ADR-TheLogIsTheRecord`](ADRs/P1-ADR-TheLogIsTheRecord.md)).
- **Pull first:** slice 2 receives, because IBFT's control layer already exists to integrate against; Raft's
  catch-up has yet to be split into a control layer and beggar.
- **The chunks received are a type of beggar's own, `ReceivedChunks`**, showing behaviour, never its fields:
  `count`, `contains`, `ids`, and `runs` — the maximal runs of consecutive ids. A set has exactly one set of
  maximal runs, so they are a property of the set and not of a representation, and they keep a checkpoint small:
  a session holding chunks 1 to 1,000,000 is one run. `ListMissingChunks` answers in runs too. The vectors still
  give ids. Its shape is laid down with the checkpoint in slice 1, and holds its first chunk in slice 2.
- **Vectors hold one instance each.** Two-ended transfers are host tests of two registries, from slice 3.

## After the slices

Integration has plans of its own: beggar beneath IBFT's control layer, in `etheram-ibft`, and Raft's catch-up
split into a control layer with beggar beneath it, in `etheram-raft`.
