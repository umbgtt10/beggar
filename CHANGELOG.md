# Changelog

The crate's versions. The specification in [`spec/`](spec/SPEC.md) is versioned on its own; each release names the
version it conforms to.

## [Unreleased]

## [0.6.0] — 2026-11-16

### Published on crates.io

A minor release: what aligns beggar with faction and cartel and changes its public surface. Conforms to the
specification's version 0.7.0, which answers every command as 0.6.1 does and leaves an absent value out where 0.6.1
wrote it `null`. The crate's types are unchanged but for the row modules' paths, now under `beggar::rows::`, and
`#[must_use]` on `Beggar::new` and `Beggar::process`: a caller that discards a receipt binds it.

#### Added
- The conformance tier checks that no public name of the crate and no name of the schema holds a word the
  vocabulary leaves out, as cartel's does: cartel's list, widened by what recovery moves unseen -- `snapshot`,
  `payload`, `bytes` -- and `quorum` (`docs/DESIGN.md` §4).

#### Changed
- The row objects move into `rows/`, the README's examples run as doctests, and `Beggar::new` and `Beggar::process`
  are `#[must_use]`, as the siblings' are. `beggar::opening_rows` and its two siblings are now under
  `beggar::rows::`.

#### Changed
- The specification is 0.7.0: it answers every command as 0.6.1 does, and leaves an absent value out where 0.6.1
  wrote it `null` -- `bound` where there is no bound, `finished` for a session still open -- as faction's and
  cartel's encodings do. The 68 vectors that held one are rewritten, once and on purpose, and `spec/released.json`
  records each vector's bytes at the version that released them; the specification records the break beside 0.3.0's.
  The crate's types are unchanged: `bound` and `finished` were and are `Option<u64>`.

## [0.5.3] — 2026-11-16

### Published on crates.io

A patch release: nothing the crate does changes. Conforms to the specification's version 0.6.1.

#### Changed
- Edition 2024, as the three bookkeepers now are; `rust-version` stays 1.85. rustfmt's 2024 style reflows the
  source. A crate's edition is its own, so no caller is touched.

## [0.5.2] — 2026-11-16

### Published on crates.io

A patch release: nothing the crate does changes. Conforms to the specification's version 0.6.1, which answers every
command as 0.6.0 does and keeps its 79 vectors as released.

#### Changed
- The README and the crate's front page follow the outline the three bookkeepers now share -- what it does, why,
  how it works, a quick start, validation, project status, design principles, non-goals, gates, roadmap, workspace,
  siblings -- the quick start run as a doctest on the front page. The agent rules carry the boundary rule, the
  specification first, additive releases and the orthogonality rules faction's state, and `P0-ADR-NoStdZeroUnsafe`
  is written down as faction's and cartel's are; the three ADR indexes share one table of the common principles.
- The specification is 0.6.1: it answers every command as 0.6.0 does, and keeps a release's vectors as released.
  The growth rule says a member a later version adds is omitted at the value that keeps the earlier behaviour;
  `spec/released.json` lists all 79 vectors by the version that released them and the SHA-256 of their bytes, and a
  test fails on one removed or changed. Planting one changed byte in a released vector is caught. The one break
  before the rule, 0.3.0's rename of the checkpoint's `received` to `chunk_set`, is recorded in the specification.
- The workspace is laid out by role, as faction's is: the crate in `crates/core/`, the packages that check it in
  `validation/` -- `ladder/`, `core-validation/`, `memory/` -- beside `spec/` and `xtask/`
  (`P2-ADR-WorkspaceLaidOutByRole`). No package is renamed, and nothing the crate does changes.

#### Added
- The specification's independent model, `beggar-spec-model` in `crates/spec-model/`, never published: a second
  implementation of the specification, written from the prose and the decision table, `no_std` and free of unsafe
  code, depending on nothing of the crate (`P1-ADR-IndependentModelOfTheSpecification`). Every vector is checked
  against it, and the crate against it over random sequences. Nothing the crate does changes.

#### Removed
- The ladder's naive registry, the reference model the exhaustive matrix and property tiers checked the crate
  against: both now check it against the specification's model, every receipt and checkpoint compared whole in the
  specification's encoding. Nothing the crate does changes.

## [0.5.1] — 2026-11-13

### Published on crates.io

#### Changed
- The crate's description and README say what 0.5.0 built: no longer under construction, the queries no longer to
  come. No change to the code.

## [0.5.0] — 2026-11-10

### Slice 5 — look inside, and keep what finished

The queries and retention, conformant to the specification's version 0.6.0. All five slices are built.

#### Added
- `ListRecoveries`, answering `RecoveriesListed { sessions }`: the names of the open sessions, in canonical order.
- `ShowRecovery { session_id }`, answering `RecoveryShown { record, finished }` — the open session of the name,
  `finished` none, else the retained one that finished last, with the number of the receipt that finished it — or
  `RecoveryNotStarted`.
- `ListMissingChunks { session_id }`, answering `MissingChunksListed { missing }`: what the same session lacks of
  `1..=chunks`, as maximal runs — or `RecoveryNotStarted`.
- Retention: a finished session, completed or cancelled, kept as it stood as a `RetainedRecord { record, finished }`,
  the checkpoint's retained records in the order of their names and, under one name, of their finishing; past the
  retention the one that finished earliest goes. It changes what a query answers and nothing else.
- `QueryRows`, beside `OpeningRows` and `PartialRows`; `ChunkSet::missing`; `Registry::query_rows`.
- The specification's 0.6.0: the three commands, their four rows, the run's JSON form `{"first", "last"}`, retained
  records with `finished`, the conformance clause that every vector answers every step but a query's alike with its
  retention raised to the top of u64, and 14 new vectors, 79 in all.
- The exhaustive matrix over every query after every history of up to four commands, at four retentions and from
  an instance already keeping two sessions (28,086 situations); random queries in the property tier, the retained
  records checked exactly, every command but a query alike at retention 0 and at the top, and one name finished
  again and again; the cluster simulation at any retention, each receiver's `ListMissingChunks` checked at
  quiescence, the policy's last word asking each node `ListRecoveries`; replay through checkpoints keeping
  finished sessions; and the memory tier at a retention, holding no more after a hundred recoveries than once full.

#### Changed
- `Checkpoint::retained` holds `RetainedRecord`s, not `SessionRecord`s.
- Retention has no default; `Fresh` already required it, and now the documents say so, in two columns.

## [0.4.0] — 2026-11-10

### Slice 4 — abandon it

Cancelling, conformant to the specification's version 0.5.0.

#### Added
- `CancelRecovery { session_id, reason }`: an open session cancelled and evicted, answering
  `RecoveryCancelled(reason)` with the reason given; otherwise `RecoveryUnaffected`. A cancel is local and by name,
  at either end; it wins once accepted — a partial after it finds no session open — and is legal, never an error,
  for a session never opened here, completed, cancelled, or racing its final partial.
- `Registry::evict`, beside open and account.
- The specification's 0.5.0: the command, its two rows, the vectors' own reason enumeration (`Unservable`,
  `Superseded`, `PeerFailed`, `Stalled { held }`) and 11 new vectors, 65 in all.
- The cluster simulation's cancellation policy: a receiver cancels a session no run has reached for as many steps as
  its patience, and tells the sender with a new event, `Abandon`, over the same faulty transport; the sender cancels
  the pages it still holds, or is unaffected if it sent them all. `cancel_abandoned` has every node cancel what it
  still holds, and a new invariant, `settled`, requires nothing left open anywhere after it.
- The exhaustive matrix over cancels after every history of up to three commands (11,700 situations); random
  cancels in the property tier, with an accepted cancel winning over every later well-formed partial; replay of
  logs carrying cancels and their reasons; and the memory tier over recoveries cancelled midway, at one end and at
  both.

#### Changed
- `Command`, `Output`, `Receipt` and `Observer` are generic over the caller's reason type, `R`, as `Beggar<R>`
  already was; `Beggar` needs `R: Clone`, to record the reason in the receipt and still report the command.
- `OpeningRows::adoption`, `PartialRows::attribution` and `Registry::account` take `R` as a parameter of their own.

## [0.3.0] — 2026-11-10

### Slice 3 — the receiving end

Both ends of a transfer, conformant to the specification's version 0.4.0.

#### Added
- `AdoptRecovery`: the other side takes on the session its peer started, under the peer's name and count,
  minting nothing. Its rows: the start's form rows (`NoChunks`, `SelfAddressed`, `NotAParty`); `OwnSession`;
  `RecoveryUnaffected` for the same adopt repeated; `RecoveryConflicted { chunks, sender, receiver }`, carrying the
  open session's terms; `RegistryFull`; and `RecoveryStarted`. An adopt after its session completed reopens it.
- `ReceiveRecoveryPartial`, with every partial row, the wrong side mirrored: a receive where this node is the
  session's sender.
- `RecoveryConflicted` and `Rejection::OwnSession`. `OwnSession` closes a gap in the design's matrix: adopting a
  name this instance minted would let a later start mint it again, and two open sessions share one name.
- `Registry`, the registry's state and its operations — open in name order, mint, account, count a call, the
  checkpoint — as a type of its own; `OpeningRows`, the rows a start or an adopt meets before a session opens;
  `Side`, which end a partial is reported on.
- The specification's 0.4.0: the two commands, their rows and 15 new vectors, 54 in all.
- `core-validation/`, the cluster simulation tier, laid out as `faction`'s: `ScenarioHarness` for the two-ended
  scenarios — pull and push, out of order, a short answer, fan-out, a node on both sides — and `ClusterSimulation`
  of three to five nodes over a seeded `FaultyTransport` (loss, duplication, delay, reordering, partitions,
  asymmetric and selective cuts, healing) with restarts. `ClusterInvariants`, each check shown firing on a crafted
  violation, holds after every delivery: receipts as the chunks call for, records on their transfer's terms, held
  by its two ends alone and in name order, receivers holding only what was delivered, names unique and minted only
  by starts, reopens only by late copies; abandonment exact at quiescence, and liveness where nothing was lost.
- The exhaustive matrix over every adopt (3,780 situations) and every partial at both ends (41,760); the property
  tier's random sequences adopting and receiving; the replay tier over the cluster's logs; and the memory tier
  two-ended — five instances, one transfer, nothing per session after 100.

#### Changed
- `StartRows` is `OpeningRows`, its start refusal `start_refusal`; `PartialRows::attribution` takes a `Side`.
- `Beggar` keeps numbering, dispatch and the observer; the state and what commands do to it are `Registry`'s.

## [0.2.0] — 2026-11-10

### Slice 2 — send it to completion

The sending end of a transfer, conformant to the specification's version 0.3.0.

#### Added
- `SendRecoveryPartial`, reporting a run of chunks sent, `first..=last`, with every row of the decision table:
  `EmptyRun { first, last }`, `RecoveryNotStarted`, `Misattributed { sender, receiver }` carrying the session's
  own ends, `WrongSide`, `ChunkOutOfRange { first, last, chunks }`, `RecoveryAdvanced { held, chunks }`,
  `RecoveryCompleted` and `RecoveryUnaffected`. A run out of range is refused whole; a run partly held adds what is
  new; completion is the chunk set full, never a run's last reaching the chunk count, and evicts the session
  without a trace, freeing its place under the bound.
- `ChunkSet`, a session's chunks accounted at this end: maximal runs of ids, sorted, merged in any order, so its
  size follows its gaps and not its chunks; `add`, `count` and `ids`.
- `StartRows` and `PartialRows`, the rows a start and a partial meet before anything changes, as types of their own.
- The specification's 0.3.0: the command, its rows and 23 new vectors, 39 in all.
- The testing ladder's replay and memory tiers. Replay: logs written in the specification's JSON as the observer
  is told them, replayed with every receipt rebuilt, a dropped line detected and the replay resynchronised at the
  next checkpoint. Memory: `memory/`, a package of its own with a counting allocator, measuring 100 recoveries in
  order and interleaved — no call holding more than the state it leaves, and nothing left per session.
- The exhaustive matrix over every partial in 12,690 small situations, and the property tier's random sequences
  sending partials too.

#### Changed
- `ReceivedChunks` is `ChunkSet`, and a record's `received` its `chunk_set`, as the specification renames them: on
  the sending end the set holds chunks sent, and the new name is true at both ends.

## [0.1.0] — 2026-11-09

### Slice 1 — open a recovery and see it

The first version with an implementation, conformant to the specification's version 0.2.0.

#### Added
- `Beggar<R>`, the registry, and its one entry point, `process(command)`. Every call is answered with a `Receipt`:
  its output, and a number the instance counts itself — 1 for the first call, one more for every call after it, a
  refused command and a query included.
- An instance is created from an `Origin`: `Fresh` — the instance id, a node and an incarnation; a retention; and
  an optional bound on open sessions — or a `Checkpoint`, restored as given.
- `StartRecovery`, with every row of the decision table: `NoChunks`, `SelfAddressed`, `NotAParty`,
  `RegistryFull { capacity }`, and `RecoveryStarted`, carrying the new session's name — the instance id and a
  count, so no two sessions anywhere share one.
- `ShowRegistry`, answering the checkpoint: the whole registry, as it stands after the call.
- `Observer`, injected through the constructor and told of every call — its command and its receipt — exactly
  once; and `NoOpObserver`, for a caller with no use for one, as `faction` provides its own.
- The specification, in `spec/`: the prose, a JSON Schema, the decision table and 16 conformance vectors. Its
  0.1.0 specified slice 1; its 0.2.0, the same day, adds the observer and changes nothing else.
- The testing ladder's first four tiers: unit, in `core/tests/`; and, in `ladder/`, a package of their own,
  conformance, the exhaustive matrix — 333 situations, each from a fresh instance and from a checkpoint, against a
  naive model — and property, random sequences against the same model.
- A coverage gate in stage 2: `cargo llvm-cov`, failing on any uncovered line, region or function of the crate,
  over every tier and over the conformance tier alone.
- The design, in `docs/`: four P0 ADRs, ten P1s and the testing ladder's P2, and `DESIGN.md`; and the house's
  quality gates, stage 1 and stage 2, in `xtask/`.

#### Changed
- A registry, not a state machine: a recovery is a collection of chunks, and beggar accounts for which have moved.
  The conception 0.0.1 described — asking a peer, retrying, falling back to a snapshot — belongs to the control
  layer above it.
- `P0-ADR-AccountsNeverDecides`, amended by `P1-ADR-EveryCallIsObserved`: beggar has one injected dependency, the
  observer, which answers nothing.
- `rust-version` 1.85, inherited from `faction`, is measured: the crate builds, and its unit tier passes, on
  1.85.0.

## [0.0.1] — 2026-09-27

Relicensed from Apache-2.0 to MIT, and `faction`'s conventions adopted. Reserves the name; no implementation.

## [0.0.0] — 2026-09-26

Reserves the name; no implementation.
