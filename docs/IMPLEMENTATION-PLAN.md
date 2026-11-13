# beggar — implementation plan

**Status: planned 2026-11-06; slice 1 built 2026-11-09, slices 2 to 5 on 2026-11-10 — all five built.** Its
specification is `spec/` at version 0.6.0; the workspace is laid out by role since 2026-11-13, the crate in
`crates/core/`, the ladder's scenario tiers in `validation/ladder/`, a package of their own, the cluster simulation in
`validation/core-validation/`, laid out as faction's, the memory tier in `validation/memory/`, the specification's
independent model in `crates/spec-model/` since 2026-11-13, and the coverage gate stands at 100% over every tier and
over the conformance tier alone, the model's lines with the crate's. The decisions are the ADRs in [`ADRs/`](ADRs/README.md) and the
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

### 1. Open a recovery and see it — built 2026-11-09

- **Adds:** `Beggar<R>` and its receipts, numbered from 1 on every call, created with an optional bound on open
  sessions and an observer told of every call, `NoOpObserver` for a caller with no use for one
  ([`P1-ADR-EveryCallIsObserved`](ADRs/P1-ADR-EveryCallIsObserved.md), added 2026-11-09 with the specification's
  0.2.0); `StartRecovery` with all its rows — `NoChunks`, `SelfAddressed`, `NotAParty`, `RegistryFull`, and
  `RecoveryStarted` under a new name minted after the instance id; and `ShowRegistry`, the checkpoint, the bound in
  it.
- **Lays down the specification** in `spec/`, at the repository root and outside the crate: the JSON Schema, the
  decision table `matrix.json`, `SPEC.md`, and the vectors, every u64 a string of decimal digits
  ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).
  A vector is an initial state, steps of a command and its expected receipt, and a final state, read through
  `ShowRegistry`: the specification checks state the way a caller would.
- **Lays down the ladder's first four tiers:** unit, in `core/tests/`; conformance, whose runner in `ladder/` -- a
  package of the ladder's scenario tiers, where stern's `paired-test-file` stands down -- maps the JSON to
  beggar's types through a model of its own — so the crate never needs `serde` — and checks every vector against
  the schema and every row of the matrix against the vectors; exhaustive matrix; and property, with the naive
  model. And the coverage gate in stage 2, over every tier and over the conformance tier alone.
- **Lays down both origins:** an instance created fresh or from a checkpoint, the checkpoint's round trip proved
  — `ShowRegistry`, restore, `ShowRegistry` — so vectors can start from the state they need.
- **Proves:** names unique across instances and within one, and receipt numbers one more on every call, refused
  calls included. The dummy goes.
- **A caller can then:** open recoveries under names unique across the cluster, be refused when a command is
  wrong, and log the checkpoint.

### 2. Send it to completion — built 2026-11-10

The serving end: the side that holds the chunks opens the session and sends them
([`P1-ADR-TheSenderOpensAndStatesTheCount`](ADRs/P1-ADR-TheSenderOpensAndStatesTheCount.md)).

- **Adds:** `SendRecoveryPartial`, a run of chunks
  ([`P1-ADR-APartialIsARunOfChunks`](ADRs/P1-ADR-APartialIsARunOfChunks.md)), with all its rows — an empty run,
  not started, misattributed, wrong side, out of range, advanced (with `held` and `chunks`), completed, and a run
  already present, unaffected — and the chunk set, kept as ranges, runs merged in any order. Completion evicts.
  The specification goes to 0.3.0, a record's `received` renamed `chunk_set`.
- **Proves:** the invariants on the chunks recorded; completion exactly when every distinct chunk has been
  recorded; no record after it. The memory tier arrives: its own test target -- `memory/`, a package of its own --
  a counting allocator, a positive control, then 100 recoveries in sequence, in order and interleaved. So does the
  replay tier: the property tier's logs written in the specification's JSON and replayed, a dropped line detected,
  the replay resynchronised from a checkpoint. The rows each command meets before anything changes move to types
  of their own, `StartRows` and `PartialRows`, when iceberg4rust finds them too much private work for `beggar.rs`.
- **A caller can then:** account a whole answer on the node that sends it — IBFT's answer to a request for blocks,
  a Raft leader's push — duplicates, strays and garbage included.

### 3. The receiving end — built 2026-11-10

- **Adds:** `AdoptRecovery` with its rows, sharing `StartRecovery`'s checks of a well-formed session, then not
  open, open with the same parameters, open with different ones; and `ReceiveRecoveryPartial` with all its rows.
  The specification, at 0.4.0, adds a row the design's matrix lacked: an adopt of a name this instance minted is
  refused (`OwnSession`), or a later start would mint it again and two open sessions share it. `RecoveryConflicted`
  carries the open session's terms. The registry's state and operations become a type of their own, `Registry`.
- **Proves:** a host test of two registries joining both ends of a transfer, the sender starting it and the
  receiver adopting it under pull and push alike: B answers A — start on B, adopt on A, sends on B, receives on A,
  both complete. The cluster simulation tier arrives, laid out as faction's `core-validation`: a scenario harness
  for the two-ended scenarios, and a cluster of three to five registries whose reports to one another a seeded
  faulty transport loses, duplicates, delays, reorders, cuts and withholds, with restarts, every cross-node
  invariant checked after every delivery. The replay tier takes its logs; the memory tier goes two-ended.
- **A caller can then:** account both ends of a transfer, pulled or pushed, for IBFT or for Raft.

### 4. Abandon it — built 2026-11-10

- **Adds:** `CancelRecovery<R>`, generic over the caller's enum: cancelled and evicted when open, unaffected when
  not. A cancel wins its race with the final partial. The reason's type runs through the vocabulary —
  `Command<R>`, `Output<R>`, `Receipt<R>`, `Observer<R>` — and the vectors carry an enumeration of their own:
  `Unservable`, `Superseded`, `PeerFailed`, `Stalled { held }`. The specification goes to 0.5.0.
- **Proves:** a late duplicate adopt reopening a finished session, as the ADR states; the memory test covers
  cancelled sessions. The cluster simulation gains a cancellation policy: a receiver cancels a session that stalls
  past its patience and tells the sender, which cancels the pages it still holds; at the end every node cancels
  what it still holds, and nothing is left open anywhere.
- **A caller can then:** act on its cancellation policy.

### 5. Look inside, and keep what finished — built 2026-11-10

- **Adds:** `ListRecoveries`, `ShowRecovery` and `ListMissingChunks`; and retention, recorded in two columns —
  `Default` and deployed — from its first line, with no default: every instance states its own. A finished
  session is kept as it stood, with the number of the receipt that finished it; past the retention the earliest
  finished goes. A name may be open and retained at once — the queries read the open one, else the last finished.
  `ListMissingChunks` answers maximal runs. The specification goes to 0.6.0.
- **Proves:** every vector again at the top of u64, every output but a query's alike; random sequences and
  clusters, faults and restarts included, likewise at 0 and at the top. At quiescence each receiver lacks exactly
  the chunks that never reached it, and the cluster's last word asks each node what it lists. Logs replay through
  checkpoints keeping finished sessions; at a retention the memory held stops growing once it is full.
- **A caller can then:** know which chunks to resend, and keep finished sessions on a host for analysis.

### The matrix, slice by slice

| row of [`DESIGN.md`](DESIGN.md) §4 | slice |
|---|---|
| `NoChunks`, `SelfAddressed`, `NotAParty`, `RegistryFull` | 1 for a start, 3 for an adopt |
| `StartRecovery` started | 1 |
| `AdoptRecovery`: not open, same, different | 3 |
| a partial: empty run, not open, misattributed, wrong side, out of range, advanced, completed, already present | 2 for a send, 3 for a receive |
| `CancelRecovery`: open, not open | 4 |
| `ShowRegistry` | 1 |
| `ListRecoveries`, `ShowRecovery`, `ListMissingChunks`, and not started for a session neither open nor retained | 5 |

## Decided

- **The entry point is `Beggar<R>`**, `R` the caller's cancel-reason enum.
- **Beggar numbers every call:** `process(&mut self, command) -> Receipt<R>`, the receipt carrying the output and
  its number. No ordinal comes from the caller
  ([`P1-ADR-TheLogIsTheRecord`](ADRs/P1-ADR-TheLogIsTheRecord.md)).
- **The sending end first:** slice 2 sends, because a session opens on its sender
  ([`P1-ADR-TheSenderOpensAndStatesTheCount`](ADRs/P1-ADR-TheSenderOpensAndStatesTheCount.md)) and IBFT's control
  layer already exists to integrate against; Raft's recovery has yet to be split into a control layer and beggar.
- **The chunks received are a type of beggar's own, `ChunkSet`**, showing behaviour, never its fields:
  `count`, `ids`, and `missing` — what it lacks, as the maximal runs of consecutive ids. A set has exactly one set
  of maximal runs, so they are a property of the set and not of a representation, and they keep a checkpoint
  small: a session holding chunks 1 to 1,000,000 is one run. `ListMissingChunks` answers in runs; the vectors'
  chunk sets still give ids. Its shape is laid down with the checkpoint in slice 1, and holds its first chunk in
  slice 2. `contains` and `runs`, planned here at first, are not built: nothing the specification asks reads them.
- **Vectors hold one instance each.** Two-ended transfers are host tests of two registries, from slice 3.

## After the slices

Integration has plans of its own: beggar beneath IBFT's control layer, in `etheram-ibft`, and Raft's recovery
split into a control layer with beggar beneath it, in `etheram-raft`. **Raft first** — decided 2026-11-10: slices
4 and 5, then Raft, while IBFT's side is busy with work of its own.

**Whether the control layer moves here is decided after integration, not before** — 2026-11-10, closing for now
[`DESIGN.md`](DESIGN.md) §12's open question. `faction` has a `protocol` crate, its `protocol-validation` and
`system-tests`, because it owns its wire; beggar owns none, and `P2-ADR-TestingLadder` already leaves its system
tier to its consumers. Its counterpart to `faction`'s protocol layer is the control layer above it, of which there
is one today, IBFT's, and will be two once Raft's recovery is split. If the two prove one consensus-unaware layer in
two configurations, it arrives here as `protocol/`, with `protocol-validation/` and `system-tests/` as `faction`'s;
if not, each stays with its consumer. One example is not enough to draw the abstraction from.
