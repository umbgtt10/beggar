# beggar — design

**Status: design settled; no implementation.** The decisions are nine ADRs in [`ADRs/`](ADRs/README.md) — four
P0 foundations and five P1s derived from them — and say why. This document says how: the vocabulary and its
matrix (§4), the semantics (§5), memory (§8), technology (§9), and where beggar sits among the layers above it
(§12). Settled 2026-09-30, amended 2026-10-12 and 2026-11-06; the decisions moved to the ADRs on 2026-11-06, and
the sections they came from keep their numbers, because other repositories cite them, and say where each went.

**Sequencing.** Beggar takes over one part of the block-recovery path in `etheram-ibft` — the bookkeeping of
the request in flight — beneath the control layer the extraction carved out of it (§12). The path's as-built
behaviour, known issues and extraction plan are `etheram-ibft/docs/RECOVERY-AS-BUILT.md`.

**Beggar itself can be built and validated with no dependency on that work and without touching firmware**, so
nothing here waits on it.

**The integration is sequenced step-by-step behind that extraction — decided 2026-09-30.** A single-change
integration was argued and rejected, and the reason is not beggar's correctness: three-tier validation settles
that independently. It is that the extraction produces two things nothing else does — **the line-by-line reading
of what recovery actually does**, which the as-built document explicitly is not, and which the layers above beggar
must reproduce — most of it the consensus-unaware control layer, not the IBFT side (§12); and **a seam proven
with known-good logic behind it**, so that beggar afterwards changes only one side. Without it, beggar, a new
driver and the rewiring land together and a failed hardware sweep has three candidate causes it cannot separate.

So the order is: the extraction, then beggar behind the seam it created. Beggar's own validation runs in
parallel with all of it.

## 0. The stance, and why it forces the architecture

**Initiating a recovery is begging for help**, and the design refuses to assume what the node behind cannot know:
whether a peer is ahead, whether it will serve, whether to ask one or several. That refusal is why beggar is a
registry — [`P0-ADR-ARegistryNotAStateMachine`](ADRs/P0-ADR-ARegistryNotAStateMachine.md).

Against IBFT's recovery as built, each is a place where beggar deliberately holds *less* knowledge than what it
replaces:

| uncertainty | as-built recovery | beggar |
|---|---|---|
| peer ahead? | `InFlightSlot` distinguishes `targeted` from `blind`, and `P1-ADR-KnowingItIsBehindIsEnoughToAsk` ratifies asking without evidence | **cannot know** — there are no heights. The ignorance is structural, not tolerated |
| peer willing? | refusal is distinct from timeout and excludes a peer at that height | **no refusal command exists.** An unwilling peer simply never produces a partial |
| one or many? | a **single** in-flight slot, ever | many concurrent sessions; beggar neither requires nor prevents fan-out |

## 1. What beggar is

**A pure accountant driving a registry of ongoing transfers**, with a configurable retention policy, full
queryability, full forensic observability, and zero technology. Four ADRs found it:

- [`P0-ADR-ARecoveryIsACollectionOfChunks`](ADRs/P0-ADR-ARecoveryIsACollectionOfChunks.md) — what it knows: sessions, chunk counts, chunk ids and node ids;
- [`P0-ADR-ARegistryNotAStateMachine`](ADRs/P0-ADR-ARegistryNotAStateMachine.md) — what it is;
- [`P0-ADR-AccountsNeverDecides`](ADRs/P0-ADR-AccountsNeverDecides.md) — what it leaves to the caller;
- [`P0-ADR-SpecificationIsNormativeCrateIsReference`](ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md) — what defines it.

Five P1 ADRs derive from them, indexed in [`ADRs/`](ADRs/README.md). What the name means is in the README.

## 2. Properties

| # | property |
|---|---|
| 1 | **consuming-protocol independent** — beggar does not know whether it serves IBFT or Raft |
| 2 | **not intertwined** with faction or any other subsystem |
| 3 | **fully observable** |
| 4 | **technology agnostic** — `no_std` + `alloc`, zero `unsafe` |
| 5 | **one single entry point**: `(output) = process(command, <internal state>)` |
| 6 | **query without side effect**: `(output) = process(command:query, <internal state>)` |

Property 1 is [`P0-ADR-ARecoveryIsACollectionOfChunks`](ADRs/P0-ADR-ARecoveryIsACollectionOfChunks.md)'s. Property 2, and with it zero injected dependencies — no clock, storage, transport, sink or
admission check; **the caller validates, and only then reports**; the peer set supplied per call and never
cached — is [`P0-ADR-AccountsNeverDecides`](ADRs/P0-ADR-AccountsNeverDecides.md)'s. Properties 3, 5 and 6 are this document's: §7 and §4.

## 3. A registry, not a state machine

Why is [`P0-ADR-ARegistryNotAStateMachine`](ADRs/P0-ADR-ARegistryNotAStateMachine.md). What it commits the design to:

`StartRecovery` and `AdoptRecovery` insert, `CancelRecovery` removes, a partial updates one set member.

Both terminal outputs evict — [`P1-ADR-RetentionGovernsQueriesNeverEmission`](ADRs/P1-ADR-RetentionGovernsQueriesNeverEmission.md).

**The invariants.** `received ⊆ 1..=chunks`, `|received| ≤ chunks`, no record survives a terminal output, every
open session has this node as exactly one of its two ends, no two open sessions share a name, and the
command-to-output mapping is total. Exhaustively property-testable over small `chunks`.

## 4. Vocabulary

Commands are **imperative**, outputs **past participle** — so "what I was asked to do" and "what happened"
cannot be confused at a call site.

**An instance is created with three things:** this node's id, an incarnation that never repeats for this node, and
its retention (§6). The first two are the **instance id**, and every session the instance opens is named after it
— [`P1-ADR-TheAccountantNamesTheSession`](ADRs/P1-ADR-TheAccountantNamesTheSession.md). **Every command carries the caller's `ordinal`** (§7), queries included.

| command | side | carries |
|---|---|---|
| `StartRecovery` | the side that opens it | `chunks`, `sender`, `receiver`. No id: beggar names the session and returns the name |
| `AdoptRecovery` | the other side | `session_id`, `chunks`, `sender`, `receiver` — a session the peer started, under the name its beggar gave it, carried here by the caller |
| `SendRecoveryPartial` | sending | `session_id`, `chunk_id`, `sender`, `receiver` |
| `ReceiveRecoveryPartial` | receiving | `session_id`, `chunk_id`, `sender`, `receiver` |
| `CancelRecovery` | either | `session_id`, `reason` — a value of the **caller's enum**, recorded and emitted, never interpreted |

**A `session_id` is the instance id that named the session and a count** — node, incarnation, count: three numbers.

**Queries change nothing** (property 6) and are commands like the rest: each carries an ordinal and is emitted.

| query | answers |
|---|---|
| `ListRecoveries` | the names of the open sessions |
| `ShowRecovery` | one session's record — `chunks`, `sender`, `receiver`, the chunk ids received — open or retained (§6) |
| `ListMissingChunks` | one session's chunk ids not yet received |
| `ShowRegistry` | the whole registry: every open and retained record, the count, the last ordinal — **the checkpoint** (§7) |

| output | meaning |
|---|---|
| `RecoveryStarted` | a session was opened; carries its `session_id` |
| `RecoveryAdvanced` | a non-redundant partial was added; carries `received` and `expected` (§5) |
| `RecoveryCompleted` | the partial just added filled the set |
| `RecoveryCancelled` | cancelled by the caller; carries the `reason` |
| `RecoveryUnaffected` | nothing changed |
| `RecoveryNotStarted` | the command names a session that is not open |
| `RecoveryConflicted` | an adopt names an open session with a different `chunks`, `sender` or `receiver` |
| `RecoveryRejected` | the command was refused and nothing changed; carries a `Rejection` saying why |
| `RecoveriesListed`, `RecoveryShown`, `MissingChunksListed`, `RegistryShown` | a query's answer |

| `Rejection` | refuses |
|---|---|
| `OrdinalNotAfter { last, given }` | any command whose ordinal is not after the last accepted (§7) |
| `NoChunks` | a start or adopt with `chunks` = 0 |
| `SelfAddressed` | a start or adopt whose `sender` and `receiver` are the same node |
| `NotAParty` | a start or adopt in which this node is neither `sender` nor `receiver` |
| `Misattributed { sender, receiver }` | a partial whose ends differ from its session's; carries the session's |
| `WrongSide` | a partial reported on the wrong end: a send on the session's receiver, a receive on its sender |
| `ChunkOutOfRange { chunk_id, chunks }` | a partial whose `chunk_id` is outside `1..=chunks` |

**A refusal is an output, never an exception** — [`P1-ADR-OneEntryPointOneOutput`](ADRs/P1-ADR-OneEntryPointOneOutput.md). `OrdinalNotAfter` is a caller's mistake; `NoChunks`,
`SelfAddressed` and `NotAParty` a degenerate command; `Misattributed`, `WrongSide` and `ChunkOutOfRange` a
degenerate or a malicious one — a peer's, which is why each carries what the log needs to tell which.

**`chunks` is mode-agnostic, and that is the point.** Under `etheram-ibft`'s deployed profile a block request
carries 8; a chunked snapshot would carry of the order of 100; a Raft leader catching a follower up carries
whatever its control layer chose. Beggar does not know which it is accounting for — [`P0-ADR-ARecoveryIsACollectionOfChunks`](ADRs/P0-ADR-ARecoveryIsACollectionOfChunks.md). It is a u64, so a
caller may make one log entry one chunk (§8).

**`sender` and `receiver` on the partials are redundant, and kept anyway — the rule is "nothing redundant,
except where the redundancy is the only witness".** Opening a session binds both to its name, which is the key, so
on an *open* session they are derivable — and checked: a partial whose ends differ is `Misattributed`. But a
partial arriving for a session that is **not** open has no record to derive them from, and without them the ledger
cannot attribute that rejection to a peer — a hole exactly where a misbehaving or buggy peer would appear. One
case, principled, and written down rather than quietly violated.

**Why `CancelRecovery` carries a reason.** Beggar cannot know *why* a session was abandoned — "superseded by a
larger one" and "the peer has pruned past what I need" are height-and-retention facts it has no vocabulary for.
Without the tag those two are indistinguishable in the forensic record, and they call for completely different
responses. Beggar records and emits it and never reads it, so protocol independence is intact. Its type is the
caller's enum, for the same reason, and the only place a peer's unwillingness survives in the ledger — [`P1-ADR-CancelEvictsWithoutATombstone`](ADRs/P1-ADR-CancelEvictsWithoutATombstone.md).

**One registry, two roles, many sessions.** The same type runs on both ends; the role follows from which
commands arrive and from `sender`/`receiver` against this node's id. A node receiving from one peer while
serving another holds two sessions. Neither end is assumed: under push the node that opens a session sends, under
pull it receives (§13), so `sender` has no default.

### The complete matrix

Checked in this order; the first row that applies gives the output.

| command | situation | output |
|---|---|---|
| any | ordinal not after the last accepted | `RecoveryRejected(OrdinalNotAfter)` |
| start, adopt | `chunks` = 0 | `RecoveryRejected(NoChunks)` |
| start, adopt | `sender` = `receiver` | `RecoveryRejected(SelfAddressed)` |
| start, adopt | this node neither `sender` nor `receiver` | `RecoveryRejected(NotAParty)` |
| `StartRecovery` | — | `RecoveryStarted`, under a new name |
| `AdoptRecovery` | not open | `RecoveryStarted` |
| `AdoptRecovery` | open, same `chunks`, `sender` and `receiver` | `RecoveryUnaffected` |
| `AdoptRecovery` | open, any of them different | `RecoveryConflicted` |
| partial | not open | `RecoveryNotStarted` |
| partial | ends differ from the session's | `RecoveryRejected(Misattributed)` |
| partial | reported on the wrong end | `RecoveryRejected(WrongSide)` |
| partial | `chunk_id` outside `1..=chunks` | `RecoveryRejected(ChunkOutOfRange)` |
| partial | new chunk, set not full | `RecoveryAdvanced` |
| partial | new chunk, **set now full** | `RecoveryCompleted` |
| partial | chunk already present | `RecoveryUnaffected` |
| `CancelRecovery` | open | `RecoveryCancelled` |
| `CancelRecovery` | not open | `RecoveryUnaffected` |
| `ShowRecovery`, `ListMissingChunks` | neither open nor retained | `RecoveryNotStarted` |
| query | otherwise | its answer |

**`RecoveryUnaffected` is reused three times deliberately, and `RecoveryNotStarted` twice; the rule generalises:
an output describes what happened to the *session*, never which command arrived** — the command is already in the
logged tuple. No tag exists merely to tell two commands apart. Apply this when the next command is added.

**Terminals are `RecoveryCompleted` and `RecoveryCancelled`, and only those.** `RecoveryConflicted` and
`RecoveryRejected` terminate nothing: an open session survives, and the *new* command is refused.

## 5. Semantics

**Completion is set cardinality, not last index.** Chunks arrive in any order; anything keyed on
`chunk_id == chunks` is wrong on the first reorder.

**The received set is a set, not a counter.** The transport retries undelivered datagrams indefinitely and
delivers a partition's oldest at the heal, so a duplicate partial is *expected*. A counter would complete a
session early and report a transfer that never finished.

**`RecoveryNotStarted` is ordinary traffic, not an error, and a late duplicate `AdoptRecovery` reopens a finished
session** — both consequences of eviction without a tombstone, in [`P1-ADR-CancelEvictsWithoutATombstone`](ADRs/P1-ADR-CancelEvictsWithoutATombstone.md).

**An out-of-range `chunk_id` is rejected, not absorbed** (`ChunkOutOfRange`). Folding it into `RecoveryUnaffected`
would let a hostile or buggy peer's garbage read as a benign duplicate.

**`RecoveryAdvanced` should carry `received` and `expected`.** Not for completion, which has its own tag, but
because a transfer stalling at 97 of 100 is otherwise invisible — and that is exactly what a cancellation policy
must act on.

### Cancellation

A cancel evicts, leaves no tombstone and wins once accepted; it is local, and legal in four situations, none an
error. Beggar cannot time anything out, so ending abandoned sessions is **the caller's stated cancellation
policy** — [`P1-ADR-CancelEvictsWithoutATombstone`](ADRs/P1-ADR-CancelEvictsWithoutATombstone.md).

## 6. Retention

How many finished sessions beggar keeps is configuration — **0** on the IBFT cluster — and it changes what a query
can answer and nothing else: every terminal output is emitted whatever is kept — [`P1-ADR-RetentionGovernsQueriesNeverEmission`](ADRs/P1-ADR-RetentionGovernsQueriesNeverEmission.md).

## 7. Observability and forensics

`process` takes `&mut self` and returns the output only ([`P1-ADR-OneEntryPointOneOutput`](ADRs/P1-ADR-OneEntryPointOneOutput.md)). Every call is emitted as
**`(ordinal, command, output)`**, never the state, which a replay rebuilds; the **checkpoint** is a query,
`ShowRegistry`, asked whenever the caller chooses; the **ordinal** is the caller's and must rise; and the
guarantee is the majority side's, because the sending node witnesses every transfer — [`P1-ADR-TheLogIsTheRecord`](ADRs/P1-ADR-TheLogIsTheRecord.md).

## 8. Memory

**The received set is ranges of chunk ids** — `3..=7`, `9..=12` — merged as chunks arrive; never a bitmap
([`P0-ADR-ARecoveryIsACollectionOfChunks`](ADRs/P0-ADR-ARecoveryIsACollectionOfChunks.md)).

| field | holds |
|---|---|
| `session_id` | three u64s: node, incarnation, count (§14) |
| `sender` | u64 |
| `receiver` | u64 |
| `chunks` | u64 — widened from u16, so a caller may make one log entry one chunk without converting |
| `received` | ranges of chunk ids |

What that costs in bytes is the implementation's to report and the memory test's to measure, never the design's
to promise.

**A session's size follows its gaps, not its chunks.** Chunks that arrive in order — the common case for a log and
for a snapshot alike — hold one range however many there are; each gap holds one more. The worst case is a peer
sending every other chunk: a range a gap, up to `chunks / 2`. Beggar cannot bound that, because it owns no policy;
the caller's chunk size and its cancellation policy (§5) bound it, and the caller states how.

**The container costs more than the record.** One session in a `Vec<Session>` is the record in one allocation; in
a `BTreeMap` it is a full B=6 leaf — 11 keys plus 11 values plus overhead, several times the record. And below ~5
sessions a linear scan also beats a tree lookup.

**Five nodes, one session, one peer: a record on each of the two involved nodes and nothing on the other three** —
`Vec::new()` does not allocate. Five concurrent sessions are one allocation of records, and their ranges.

**Churn is negligible too.** At 3–8 sessions per catch-up and a few hundred catch-ups per long run, that is
~1,900 sessions opened and evicted across ~16 hours — about **0.03 per second**, against a logging path doing ~40
lines per second on the same board.

### The memory test — required in CI

**Shape.** Five nodes, one session, eight chunks, one peer; then **100 recoveries in sequence**, each completed
so that retention 0 evicts it. Chunks arrive in order, so each session holds one range; **a second run interleaves
them** — the odd chunks first — so each session's ranges grow to their worst before it completes. Two assertions,
catching different bugs:

- **peak live bytes never exceed the skeleton plus one session's record and its ranges** — catches transient bloat
  that frees itself before the end
- **live bytes after 100 equal live bytes after the first** — catches per-session residue

**Measure at the allocator, not in the type.** A counting `#[global_allocator]` sees every `Layout`, so it
reports what the allocator saw rather than what the registry believes. A type that reports its own size is the
failure mode this project has already paid for: a census that existed, was tested, and was never called.

**Why "equal to the skeleton" is the wrong assertion.** Removing entries from a `Vec` does not release its
buffer, so after 100 sequential sessions the registry legitimately holds `len = 0` with capacity retained — and a
literal comparison against an empty skeleton fails on *correct* code. Retaining bounded capacity is desirable;
it is what avoids churn. **After-100-equals-after-1** is independent of capacity policy and sharper against the
real bug class, which is per-session accumulation.

**A positive control is mandatory.** A mis-wired allocator hook never moves the counter and reports a perfect
no-leak result from a broken instrument. Assert first that **one** session's allocation is observed as non-zero,
then that 100 do not grow.

**Two CI mechanics decide whether it works at all.** A `#[global_allocator]` is per-binary, so counts would
include every other test in the target, and parallel execution would interleave their allocations — flaky, and
flaky in the direction of **passing**. It needs its **own test target**.

## 9. Technology

**`no_std` + `alloc`.** `alloc` is what makes a session-keyed map practical instead of a hand-rolled free list.

**Zero `unsafe`, gateable rather than reviewable:** `#![forbid(unsafe_code)]` at the crate root makes it a
compile error. A registry over identifiers and sets has no motive for it.

**A compile-time fixed-capacity registry was considered and rejected.** It would bind an embedded constraint
into a core that property 4 requires to be platform-neutral — worse than the problem it solves, and §8 shows
the problem is not there.

## 10. Closed questions, and what remains

**All five open questions were closed on 2026-09-30, because a specification cannot have any.** Recorded here
with where each landed, so the reasoning is not lost in the sections that now state the conclusions flatly.

| question | closed as | where |
|---|---|---|
| return shape of `process` | `&mut self`, returns the output tag only — posterior state is derivable, so nothing else is needed | §7 |
| ordered ledger or membership | **membership** in the registry; order lives in the log, via the ordinals | §7 |
| where "unservable" lives | **not in beggar** — it is a height-and-retention fact. `CancelRecovery` gains an opaque caller-defined `reason` instead | §4, §5 |
| `sender`/`receiver` redundancy | **kept**, under the rule *"nothing redundant, except where the redundancy is the only witness"* | §4 |
| shape of the injected admission policy | **deleted, not closed.** There is no admission policy and no trait boundary — beggar never sees an entry, so there is nothing to admit. Zero injected dependencies | §2 |
| width of `chunks` — closed 2026-11-06 | **u64.** u16 was sized for a block request; a recovery is any number of chunks | §8, §13 |
| the checkpoint cadence — closed 2026-11-06 | **the caller's.** The checkpoint is a query, asked whenever the caller chooses | §7, §14 |
| whether `chunks` may be zero — closed 2026-11-06 | **no:** `RecoveryRejected(NoChunks)` | §4 |
| whether `session_id` uniqueness is beggar's — closed 2026-11-06 | **beggar's:** it names every session, after an instance id that never repeats | §14 |

### What remains genuinely open

Nothing in the design, as of 2026-11-06: the three questions left here were closed that day, in the table above.

## 11. What this supersedes in the README, the crate's documentation and its manifest

All three predated this design. The README was rewritten to agree with it; the crate's documentation
(`src/lib.rs`) and its manifest's description and keywords still said "state machine", "admissible commands" and
"episodes" until 2026-11-06, when they were brought into line. crates.io shows 0.0.1's description until a release
carries the new one. What the old texts said is listed so nobody reintroduces it:

| the old texts said | now |
|---|---|
| "recovery **state machine**" | a **registry**; there are no states — §3 |
| "the set of **admissible commands** returned on every result" | **dropped** — it presupposes states — §3 |
| transitions emitted as `(from, event, to, epoch)` | `(output) = process(command, state)` plus a caller-supplied ordinal — §7 |
| a control layer that detects lag, chooses a peer, retries on timeout and falls back to a snapshot | **out of scope.** Beggar accounts; it does not detect, choose or time. It has no heights and no clock — §1, §2. That layer exists, above beggar — §12 |
| "admission is the trait boundary" | **also superseded.** Beggar never sees an entry, so there is nothing to admit: the caller validates and only then reports. **Zero injected dependencies** — §2 |

## 12. Where beggar sits: three layers — decided 2026-10-11

**The extraction in `etheram-ibft` split recovery along the line [`P0-ADR-AccountsNeverDecides`](ADRs/P0-ADR-AccountsNeverDecides.md) draws, and found a layer on each side of
it.** What it measured as consensus-unaware is not beggar but the control layer that ADR puts above it:

| layer | knows | where |
|---|---|---|
| **beggar** — accounting | sessions, chunks, partials, node ids | this repository; designed, not built |
| **control** — consensus-unaware, chain-aware | heights, peer statuses, episodes, the request in flight, retries, refusals | `etheram-ibft`, `recovery/control/`, package `recovery-control` |
| **IBFT** | wire, certificates, validator membership and schedule, the execution payload, consensus's reactions, snapshot | `etheram-ibft`, `recovery/ibft/` and the node |

**Beggar takes over one thing from the control layer: the bookkeeping of the request in flight.** A block request
is a session of `chunks` blocks — 8 under the deployed profile, as §4 says. The control layer keeps everything
beggar refuses to know — which heights a request covers, which peer is ahead, why a request was abandoned, which
travels as `CancelRecovery`'s `reason`, the caller's enum — and it is the driver that, per §5, owns the
cancellation policy.

**The dependency runs one way.** The control layer will depend on beggar; beggar depends on nothing, and property 1
holds by construction as before. The control layer's own independence is mechanical too, at a different line: its
manifest names `etheram-core` and nothing else of the ecosystem, and a check in `etheram-ibft`'s first gate refuses
consensus vocabulary in its source, comments included. It may know a chain; it may not know a consensus protocol.

**Three of this document's conclusions were adopted above it, unchanged in substance:**

- **the emission shape** (§7): `(ordinal, command, output)` plus a periodic checkpoint becomes `(ordinal, event,
  facts, outcome)` there, because the control layer's input includes the facts its caller supplies. Where it is
  emitted is still open in `etheram-ibft`;
- **"the caller validates, and only then reports"** (§2): the IBFT side validates a response and reports a verdict,
  so no block and no certificate enters the control layer;
- **the peer set supplied per call and never cached** (§2): the control layer borrows it from its caller on every
  call.

**Raft has the same three layers — 2026-11-06, §13.** Its control layer is the catch-up bookkeeping behind
`etheram-raft`'s `CatchUpBookkeeping` port, whose default steps back on a rejection: log indices, `nextIndex` and
`matchIndex`, the follower to send to next. Beggar sits beneath it as beneath IBFT's, and takes over the same one
thing: the bookkeeping of what is in flight.

**Status, 2026-10-12.** The control layer exists on `etheram-ibft`'s branch `extract-recovery-crate`, as `(outcome)
= process(event, facts)` behind one trait. Beggar does not exist yet, so nothing depends on it. **Not decided:**
whether the control layer ever moves into this repository. If it does, it arrives as a second crate that depends
on beggar, never the reverse.

**Status, 2026-11-06.** The extraction merged to `etheram-ibft`'s main in `0963d1d` on 2026-10-17, so the control
layer is on main, in `recovery/control/`. Beggar still does not exist. Nor does Raft's split: its catch-up
bookkeeping is one layer today, and beggar's arrival there is what would split it.

## 13. A recovery is a collection of chunks — decided 2026-11-06

[`P0-ADR-ARecoveryIsACollectionOfChunks`](ADRs/P0-ADR-ARecoveryIsACollectionOfChunks.md): logs or snapshots, IBFT or Raft, pushed or pulled. What a chunk holds, in what order chunks move, under what
policy and which protocol are the caller's, and a stream becomes a collection above beggar.

## 14. Names, refusals and queries — decided 2026-11-06

Each decision of the day is in its ADR:

- beggar names the session, after an instance id whose incarnation is the caller's — [`P1-ADR-TheAccountantNamesTheSession`](ADRs/P1-ADR-TheAccountantNamesTheSession.md);
- one entry point, one output for every command; refusals are outputs, queries are commands — [`P1-ADR-OneEntryPointOneOutput`](ADRs/P1-ADR-OneEntryPointOneOutput.md);
- the checkpoint is a query, and the ordinal must be after the last accepted — [`P1-ADR-TheLogIsTheRecord`](ADRs/P1-ADR-TheLogIsTheRecord.md);
- the cancel reason is the caller's enum, and a late duplicate adopt is ended by the caller's policy — [`P1-ADR-CancelEvictsWithoutATombstone`](ADRs/P1-ADR-CancelEvictsWithoutATombstone.md);
- no caps — [`P0-ADR-AccountsNeverDecides`](ADRs/P0-ADR-AccountsNeverDecides.md);
- JSON conformance vectors, a session's chunks given as a set of ids — [`P0-ADR-SpecificationIsNormativeCrateIsReference`](ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md).

What stays here is §4's vocabulary and matrix, in which `sender` has no default.
