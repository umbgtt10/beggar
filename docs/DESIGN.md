# beggar — design

**Status: design settled 2026-09-30. No implementation.** This document supersedes every earlier description
of beggar, including the framing in `README.md` — see §11 for what changed and why. **Amended 2026-10-12** with
where beggar sits among the layers of the integration — §12. Nothing settled here changed. **Amended 2026-11-06**:
a recovery is just a collection of chunks — logs or snapshots, IBFT or Raft, pushed or pulled — §13; and with it
the received set holds ranges of chunk ids rather than a bitmap, and `chunks` is a u64 — §8.

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

**Initiating a recovery is begging for help.** Not a metaphor for the name — a statement about what the asking
node can know, and the whole design follows from it.

A node that has fallen behind faces three irreducible uncertainties:

| | |
|---|---|
| **Is the peer ahead?** | It may or may not know. A status may have arrived; it may not have |
| **Will the peer serve?** | It may or may not know. Willingness is discovered only by asking, and silence and refusal look identical from outside |
| **Ask one peer, or several at once?** | It may or may not be willing to fan out. That is a choice, not a fact about the world |

**These are negative requirements — things the design refuses to assume** — and each is a place where beggar
deliberately holds *less* knowledge than the recovery it replaces:

| uncertainty | as-built recovery | beggar |
|---|---|---|
| peer ahead? | `InFlightSlot` distinguishes `targeted` from `blind`, and `P1-ADR-KnowingItIsBehindIsEnoughToAsk` ratifies asking without evidence | **cannot know** — there are no heights. The ignorance is structural, not tolerated |
| peer willing? | refusal is distinct from timeout and excludes a peer at that height | **no refusal command exists.** An unwilling peer simply never produces a partial |
| one or many? | a **single** in-flight slot, ever | many concurrent sessions; beggar neither requires nor prevents fan-out |

### Why this makes it a registry rather than a state machine

**You cannot build a state machine over uncertainties you refuse to resolve.** A machine would need a label
distinguishing *waiting on a willing peer* from *waiting on an unwilling one* — and that distinction is
unknowable from inside, because the two produce identical observations. Any such label would be a guess encoded
as structure.

A registry does not need it. It records what actually arrived and says nothing about what a silence means. So
the epistemic humility is not a gloss on the design; **it is the reason the design is accounting**, and §3's
argument about state labels is this argument in mechanical form.

### Two consequences

**`CancelRecovery`'s `reason` carries more weight than it appears to.** Because beggar cannot express refusal,
*"the peer said no"* and *"the peer said nothing"* collapse into the same absence — and they warrant different
responses, since only one of them should exclude that peer next time. The tag is the **only** place willingness
survives in the ledger. That makes it load-bearing rather than a convenience.

**The stance is about not assuming, not about not acting.** "Begging" reads as passive, and one of the as-built
open issues is precisely *"reactive only, never probes"*. If the stance were taken to endorse passivity, that
defect would quietly become a feature. It does not: **a beggar may ask repeatedly, and ask several peers at
once — what it may not do is presume the answer.** The design permits the first and refuses the second, and the
two must not be conflated.

**The stance names the node in need, not the one that opens a session.** Under pull the node behind asks and
opens it; under push the node ahead opens it, and the node behind never asked. Beggar cannot tell the two apart
and does not need to (§13).

## 1. What beggar is

**A pure accountant driving a registry of ongoing operations**, with a configurable retention policy, full
queryability, full forensic observability, and zero technology.

It deals in **no bytes, blocks, arrays or wire types**. It knows session identities, chunk counts and node
identities. The payload moves elsewhere; beggar records that it moved. Nor bits: which chunks have arrived is a set
of chunk ids, never a bitmap (§8).

**Why the name.** Recovery is a supplicant. A node that fell behind has no leverage over the peers that did
not: it asks for what it missed, the peer **may refuse**, it may be offered something other than what it asked
for, and if ignored its only recourse is to ask again. That asymmetry is the mechanism.

**Sibling to `faction`, and the difference is load-bearing.** Faction *is* a state machine — membership has
genuine states a node moves between, and `Bootstrapped` is terminal. Beggar has no states at all. That decides
how each is tested: **an FSM needs transition coverage, a registry needs invariants.** Neither should borrow
the other's strategy.

## 2. Properties

| # | property |
|---|---|
| 1 | **consuming-protocol independent** — beggar does not know whether it serves IBFT or Raft |
| 2 | **not intertwined** with faction or any other subsystem |
| 3 | **fully observable** |
| 4 | **technology agnostic** — `no_std` + `alloc`, zero `unsafe` |
| 5 | **one single entry point**: `(output) = process(command, <internal state>)` |
| 6 | **query without side effect**: `(output) = process(command:query, <internal state>)` |

**Property 1 is true by construction, not by discipline.** There is no vocabulary in which a protocol-specific
type could be expressed.

**Beggar has zero injected dependencies — settled 2026-09-30.** An earlier draft made **admission** a trait
boundary, on the reasoning that whether an entry is valid and who may serve it does not generalise. That was
inherited from a broader conception in which beggar fetched and validated. It does not apply: **beggar never
sees an entry.** The caller validates, and only then reports that a chunk arrived; a rejected entry is simply
never reported. Nothing is left to inject — no clock (property 4 forbids it), no transport, and emission is
*returned* rather than pushed to a sink.

That is a result rather than a gap. A type with no injection points is trivially implementable in any language
and needs no stubs to test, which is what makes a cross-language conformance suite practical.

**Property 2 has an API consequence.** Beggar cannot ask whether it is a member and cannot be told the
validator set changed. The peer set is **supplied per call** and never cached, which costs nothing while
membership is static and is what lets beggar survive faction gaining join/leave without either learning about
the other.

**Property 4 is what collapses the entrances.** A core that owns no timer and no socket cannot have five call
sites; "the retry timeout expired" becomes a command rather than an entrance.

## 3. A registry, not a state machine

**The test that settles it.** In a state machine, the response to a command depends on a *state label* that
cannot be read off the data — the same command with the same arguments yields different results in different
states. Here it never does. Every output is a pure function of the command and the session record. No label does
any work, so there is no machine: there is a keyed collection with validated operations. `StartRecovery`
inserts, `CancelRecovery` removes, a partial updates one set member.

**Three consequences.**

**`admissible commands` does not exist.** An earlier design returned the set of legal commands alongside the
output. That only makes sense relative to a state, and keeping it would have meant inventing a state concept to
give the field something to say. If a sender needs to know what to resend, that is a **query returning the
missing chunk ids** — data from the record, not a transition list.

**Both terminal outputs evict.** Retaining a completed record accumulates exactly as a height-keyed exclusion
map once did in `etheram-ibft`: 292 orphaned entries at ~110 bytes, fatal by fragmentation long before size. With
both evicting, the registry's size is bounded by *concurrent* transfers, not by transfers ever attempted.

**Correctness is invariants, not paths.** `received ⊆ 1..=chunks`, `|received| ≤ chunks`, no record survives a
terminal output, and the command-to-output mapping is total. Exhaustively property-testable over small `chunks`.

## 4. Vocabulary

Commands are **imperative**, outputs **past participle** — so "what I was asked to do" and "what happened"
cannot be confused at a call site.

| command | side | carries |
|---|---|---|
| `StartRecovery` | — | `session_id`, `chunks`, `sender`, `receiver`. `sender` defaults to this node's id when set at instantiation |
| `SendRecoveryPartial` | sending | `session_id`, `chunk_id`, `sender`, `receiver` |
| `ReceiveRecoveryPartial` | receiving | `session_id`, `chunk_id`, `sender`, `receiver` |
| `CancelRecovery` | either | `session_id`, `reason` — an **opaque caller-defined tag**, recorded and emitted, never interpreted |

| output | meaning |
|---|---|
| `RecoveryStarted` | a session was opened |
| `RecoveryAdvanced` | a non-redundant partial was added |
| `RecoveryCompleted` | the partial just added filled the set |
| `RecoveryCancelled` | cancelled by the caller |
| `RecoveryUnaffected` | nothing changed |
| `RecoveryNotStarted` | a partial arrived for a session that is not open |
| `RecoveryConflicted` | a start names an open session with different parameters |
| `RecoveryPartialRejected` | the `chunk_id` is outside `1..=chunks` |

**`chunks` is mode-agnostic, and that is the point.** Under `etheram-ibft`'s deployed profile a block request
carries 8; a chunked snapshot would carry of the order of 100; a Raft leader catching a follower up carries
whatever its control layer chose. Beggar does not know which it is accounting for, which is property 1 paying for
itself (§13). It is a u64, so a caller may make one log entry one chunk (§8).

**`sender` and `receiver` on the partials are redundant, and kept anyway — the rule is "nothing redundant,
except where the redundancy is the only witness".** `StartRecovery` binds both to the `session_id`, which is the
key, so on an *open* session they are derivable. But a partial arriving for a session that is **not** open has no
record to derive them from, and without them the ledger cannot attribute that rejection to a peer — a hole
exactly where a misbehaving or buggy peer would appear. One case, principled, and written down rather than
quietly violated.

**Why `CancelRecovery` carries a reason.** Beggar cannot know *why* a session was abandoned — "superseded by a
larger one" and "the peer has pruned past what I need" are height-and-retention facts it has no vocabulary for.
Without the tag those two are indistinguishable in the forensic record, and they call for completely different
responses. Beggar records and emits it and never reads it, so protocol independence is intact.

**One registry, two roles, many sessions.** The same type runs on both ends; the role follows from which
commands arrive and from `sender`/`receiver` against this node's id. A node receiving from one peer while
serving another holds two sessions.

### The complete matrix — eight outputs, ten situations

| command | session state | output |
|---|---|---|
| `StartRecovery` | unknown | `RecoveryStarted` |
| `StartRecovery` | open, same `chunks` | `RecoveryUnaffected` |
| `StartRecovery` | open, **different** `chunks` | `RecoveryConflicted` |
| partial | not open | `RecoveryNotStarted` |
| partial | open, new chunk, set not full | `RecoveryAdvanced` |
| partial | open, new chunk, **set now full** | `RecoveryCompleted` |
| partial | open, chunk already present | `RecoveryUnaffected` |
| partial | `chunk_id` out of range | `RecoveryPartialRejected` |
| `CancelRecovery` | open | `RecoveryCancelled` |
| `CancelRecovery` | unknown | `RecoveryUnaffected` |

**`RecoveryUnaffected` is reused three times deliberately, and the rule generalises: an output describes what
happened to the *session*, never which command arrived** — the command is already in the logged tuple. No tag
exists merely to tell two commands apart. Apply this when the next command is added.

**Terminals are `RecoveryCompleted` and `RecoveryCancelled`, and only those.** `RecoveryConflicted` terminates
nothing: the open session survives and the *new* start is refused.

## 5. Semantics

**Completion is set cardinality, not last index.** Chunks arrive in any order; anything keyed on
`chunk_id == chunks` is wrong on the first reorder.

**The received set is a set, not a counter.** The transport retries undelivered datagrams indefinitely and
delivers a partition's oldest at the heal, so a duplicate partial is *expected*. A counter would complete a
session early and report a transfer that never finished.

**`RecoveryNotStarted` is ordinary traffic, not an error.** A duplicate final chunk arriving after eviction
produces it, and duplicates are guaranteed. It will appear routinely on a healthy node. **Do not alert on it**,
and pair any count of it with a witness — otherwise it becomes a number that looks like catastrophic loss and
is a capacity signature.

**An out-of-range `chunk_id` is rejected, not absorbed.** Folding it into `RecoveryUnaffected` would let a
hostile or buggy peer's garbage read as a benign duplicate.

**`RecoveryAdvanced` should carry `received` and `expected`.** Not for completion, which has its own tag, but
because a transfer stalling at 97 of 100 is otherwise invisible — and that is exactly what a cancellation policy
must act on.

### Cancellation

Needed for four independent reasons: a session opened for a node that turns out unservable must be abandonable;
a node falling further behind needs a larger session and the old one must die; the serving peer may be banned,
refuse, or fail mid-session; and any timeout policy needs something to act *through*.

**Cancel is local. Propagation is outside.** Beggar does no wire, so cancelling on one side does not tell the
other.

**Cancel evicts; it does not tombstone.** A partial after a cancel is therefore indistinguishable from one for a
session that never existed, and `RecoveryNotStarted` covers both. The cost is accepted: beggar can say a session
is absent, not *why*. Do not add a tombstone state to recover the distinction.

**Legal in four situations, none an error:** unknown session (normal — it may already be evicted), already
completed, already cancelled, and racing the final partial.

**The race needs a rule, because arrival order is not a contract.** **Cancel wins once accepted, and no
completion is emitted afterwards for that `session_id`.** A canceller has information beggar does not — that is
why it is cancelling — so letting transport timing override it is the wrong default.

**Beggar cannot time anything out**, because property 4 forbids owning a clock. `CancelRecovery` closes the
*mechanism* gap and not the *policy* gap: **whoever drives beggar owns a stated cancellation policy.** A cancel
command nobody decides to send leaves sessions accumulating exactly as no cancel command would. The second half
is the half that gets forgotten.

## 6. Retention

**How many completed-or-cancelled sessions beggar keeps is configuration.** On the IBFT cluster it is **0**:
evicted immediately — **and the terminal output is still emitted.**

**The invariant that must not be optimised away: retention governs what can be *queried*, never what is
*emitted*.** The tempting shortcut is to skip emitting a terminal nobody will retain; that destroys the forensic
trail in precisely the configuration the cluster runs.

One knob, not two mechanisms — which is what lets the same code serve both cases property 4 promises: retention
high on a host for tests and analysis, 0 on a board.

**Record it in two columns from the first line of code** — `Default` and deployed. `etheram-ibft`'s constants
table has two columns because a one-column version led four downstream records to quote a library default as
deployed, and a published mechanism to rest on a poll interval no board ever ran. A retention default other than
0 with the cluster at 0 is the identical trap.

**Query axes.** Open sessions from the API; terminated sessions from the capture. "All session ids queryable"
is two distinct questions — *which are open* and *which ever existed* — and they should not be conflated.

## 7. Observability and forensics

**`process` takes `&mut self` and returns the output tag only — settled 2026-09-30.** It does not return the new
state, and the reason is worth stating because it decides what gets logged: **if the function is total and
deterministic, the posterior state is derivable from the prior state and the command.** A replay does not need
it.

**So the emission per call is `(ordinal, command, output)` — not the full internal state.** An earlier draft had
every call emit its whole state, which is what made observability expensive: a multi-field dump on a board with a
small transmit ring competes with the very lines it exists to be correlated against. Commands and outputs are
small and bounded; state is reconstructed by replaying them.

**Plus a periodic full-state checkpoint, and its purpose is precise.** Not debugging — **resynchronisation.**
Replay from an initial state only works while the command sequence is complete, and on a severed node it will not
be. A checkpoint lets replay recover after a gap instead of diverging from it forever, which bounds the damage a
gap does rather than letting one loss invalidate everything after it. Cadence is a configuration value.

**A caller-supplied monotonic ordinal on every command is required, not optional.** It is the only thing that
makes the forensic claim checkable: without it a replay built from a lossy capture silently omits events and
looks self-consistent. With it, gaps are **detectable** — which is also what makes the checkpoint above usable,
since you must know a gap happened to know you need to resynchronise. It yields a completeness metric that can
fail: *reconstructed X of Y events*. A replay that cannot say how much it is missing is not evidence.

**Beggar owns no clock, so it cannot supply the ordinal itself.** It comes in with the command.

**The in-memory record therefore carries membership, not order.** The received set answers *which* chunks arrived;
*when* is a log question, answered by the ordinals. An ordered structure in the registry would duplicate what the
log already holds.

### The guarantee is majority-side, and the vocabulary is why that works

A partitioned node is close to unobservable: its log path is severed along with its data path. Measured on the
reference hardware — unsevered nodes lose 0.05% of a periodic probe; the severed node loses 2.8% to 84.6% across
a run window, and **inside the partition the loss is effectively total**, what survives being the transmit ring's
content at the moment of heal.

**The asymmetry is structural: the node whose record matters most is the one that cannot speak.**

**But `SendRecoveryPartial` is fed on the serving node, which is not severed.** So every chunk of a session has
an unsevered witness, and the existence, size, peer and progress of a transfer are reconstructible from the
majority alone. What the silence costs is receive *timings* and the local view — not the fact of the transfer.
**That is the argument for keeping both `Send` and `Receive`** rather than instrumenting only the receiver and
putting the whole record on the one node that cannot speak.

**The loss is accepted as a property of the platform**, not deferred as a defect. A reliable transport would
recover the 0.05% and none of the rest, because a severed port is a *connectivity* failure, not a reliability
one — there is no path to retry on. It would also reintroduce head-of-line blocking on the log path, which has
already cost this family of systems a stall that read as a timing problem. And a larger transmit ring does not
close it either: eight times the buffer buys single-digit percentages of a multi-minute partition, because the
binding constraint is partition duration.

## 8. Memory

**No bits or bytes in the record — amended 2026-11-06.** The first sizing kept the received set as a bitmap, a bit
a chunk, sized for a block request's 8. A bitmap is a byte-level representation in a crate that deals in none, and
it grows with the chunk count: a recovery is any number of chunks (§13), and over a u64 count a bitmap cannot
exist. The received set is a set of **ranges of chunk ids** — `3..=7`, `9..=12` — merged as chunks arrive.

| field | holds |
|---|---|
| `session_id` | u64 |
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

The first two turned out to be one decision: answering the return shape made the ordered-ledger question
disappear, because once commands and outputs are the emitted record, order is already in the log and duplicating
it in memory buys nothing.

The last was an error rather than a question, carried forward from a conception in which beggar fetched and
validated. It is the largest single simplification in this document.

### What remains genuinely open

- **The checkpoint cadence** — a configuration value, but nobody has proposed a default.
- **Whether `chunks` may be zero.** A zero-chunk session would complete on creation. Probably forbidden, but the
  matrix in §4 does not currently say.
- **Whether `session_id` uniqueness is beggar's to enforce** or the caller's to guarantee. `RecoveryConflicted`
  handles a *collision with different parameters*, but not a caller that reuses an id after eviction — which is
  indistinguishable from a fresh session, and may well be fine.

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

**One README point survives intact**, and it is kept in both files: **a monotonic counter is what makes a gap
prove loss** — absence of a log line otherwise proving nothing.

The admission row was itself a correction. An earlier revision of this document listed admission as one of *two*
surviving points, which was wrong for the same reason the control layer was: it belonged to a beggar that fetched
and validated.

## 12. Where beggar sits: three layers — decided 2026-10-11

**The extraction in `etheram-ibft` split recovery along the line §11 draws, and found a layer on each side of
it.** What it measured as consensus-unaware is not beggar: it detects lag, chooses a peer from the statuses it has
seen, retries on timeout and excludes a peer that refuses — the control layer §11 puts out of scope. Folding it
into beggar would reverse four settled decisions: no heights (§0, §1); a registry and not a state machine (§3 —
the control layer has genuine states: an episode open or closed, a request targeted or blind); no refusal command
(§0); and the control layer out of scope (§11). The reading the owner decided is the one under which nothing here
is reversed:

| layer | knows | where |
|---|---|---|
| **beggar** — accounting | sessions, chunks, partials, node ids | this repository; designed, not built |
| **control** — consensus-unaware, chain-aware | heights, peer statuses, episodes, the request in flight, retries, refusals | `etheram-ibft`, `recovery/control/`, package `recovery-control` |
| **IBFT** | wire, certificates, validator membership and schedule, the execution payload, consensus's reactions, snapshot | `etheram-ibft`, `recovery/ibft/` and the node |

**Beggar takes over one thing from the control layer: the bookkeeping of the request in flight.** A block request
is a session of `chunks` blocks — 8 under the deployed profile, as §4 says. The control layer keeps everything
beggar refuses to know — which heights a request covers, which peer is ahead, why a request was abandoned, which
travels as `CancelRecovery`'s opaque `reason` — and it is the driver that, per §5, owns the cancellation policy.

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

**Beggar supports logs and snapshots, for IBFT and for Raft. As a bookkeeper it does not know what is being
transferred, how, in what sequence, with what policy or under which protocol. A recovery is just a collection of
chunks.** The owner's rule; §4's mode-agnostic `chunks` said it for IBFT's two modes, and this says it for
everything.

- **Logs and snapshots alike.** IBFT's request for 8 blocks, a Raft follower's missing log entries, a snapshot sent
  in pieces: each is a session of `chunks`. What a chunk holds — a block, an entry, a batch of entries, a slice of
  a snapshot — is the caller's choice and never reaches beggar.
- **Both protocols.** IBFT's control layer and Raft's (§12) drive the same registry through the same entry point.
- **Pushed or pulled.** Under pull the node behind opens the session, as IBFT asks for blocks; under push the node
  ahead does, as a Raft leader sends a lagging follower what it lacks. §4's one registry with two roles covers
  both, and nothing in the commands says which it was.
- **A stream becomes a collection above beggar.** A log catch-up has no fixed end: entries keep arriving, and Raft
  steps back on a rejection. The control layer decides what one recovery covers — what was missing when it
  opened, with entries appended later left to ordinary replication — and a range found wrong is a
  `CancelRecovery`, its `reason` saying why, and a new session. Beggar sees a fixed count either way.
- **No bits or bytes, and a wider count.** The received set is ranges of chunk ids, never a bitmap, and `chunks`
  is a u64 (§8).
