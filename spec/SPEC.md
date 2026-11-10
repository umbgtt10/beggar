# beggar — the specification

**Version 0.6.0.** The four parts of this specification are versioned together: this prose, the data model in
[`schema.json`](schema.json), the decision table in [`matrix.json`](matrix.json) and the conformance vectors in
[`vectors/`](vectors/). It is normative; the `beggar` crate is one implementation of it, and where the two disagree
the specification is right ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](../docs/ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).

**What 0.6.0 covers:** beggar whole — an instance, created fresh or from a checkpoint, and the observer it reports
every call to; both ends of a transfer — `StartRecovery` and `AdoptRecovery`, `SendRecoveryPartial` and
`ReceiveRecoveryPartial` — and `CancelRecovery`; the queries `ListRecoveries`, `ShowRecovery`, `ListMissingChunks`
and `ShowRegistry`, the checkpoint; each with every row it has; and retention, finished sessions kept for the
queries. 0.6.0 is 0.5.0 with the three other queries and retention.

## 1. Conventions

**Every u64 is a string of decimal digits**, with no sign, no leading zero and no exponent, its value within
`0..=18446744073709551615`. Many JSON parsers read numbers as 64-bit floats, exact only to 2⁵³; an incarnation drawn
from a nanosecond timestamp is already past it.

**Every value of an enumeration is an object with exactly one member**, named for the value: `{"ShowRegistry": {}}`,
`{"RecoveryRejected": {"RegistryFull": {"capacity": "2"}}}`. A value that carries nothing carries the empty object.

**Names are the vocabulary's.** Commands are imperative and outputs past participle, so what a caller asked for
and what happened cannot be confused.

**A run is `first..=last`:** the chunk ids from `first` to `last`, both included, as a partial carries them and
as an answer lists them, `{"first": "3", "last": "7"}`. A single chunk is a run of one, `n..=n`. A run whose `last`
is below its `first` is empty, and malformed.

**A set of chunk ids listed as runs is listed as its maximal runs:** the fewest runs that hold exactly its ids, in
ascending order — chunks 1, 2, 3, 5, 6 and 9 are `1..=3`, `5..=6`, `9..=9`. No two of them overlap or touch: each
starts at least two past the `last` of the one before. A set has exactly one list of maximal runs, so they are a
property of the set and not of how an implementation keeps it; the empty set is the empty list.

**Canonical order.** Wherever an answer lists records or names, they come in ascending order of the names, compared
as `(node, incarnation, count)` numerically, and retained records under one name in ascending order of the receipt
numbers that finished them; a chunk set comes as its chunk ids in ascending order. An answer has
exactly one form, so two implementations' logs of the same calls are identical.

## 2. An instance

**An instance is created from one of two origins**, and with an observer (§3).

- **Fresh:** the node's id, an incarnation, a retention and a bound on open sessions, or none. The node's id and
  the incarnation are the **instance id**. The caller guarantees that an incarnation never repeats for its node
  ([`P1-ADR-TheAccountantNamesTheSession`](../docs/ADRs/P1-ADR-TheAccountantNamesTheSession.md)). A fresh instance
  holds no session, has minted no name, and its first receipt is number 1.
- **A checkpoint:** an instance restored from the checkpoint is in exactly the state the checkpoint describes —
  the same instance id, retention and bound, the same records, the same name count and the same next receipt
  number. A checkpoint is restored as given: one that no `ShowRegistry` answered is outside this specification.
  Restoring is how a replay resynchronises and how a vector starts from the state it needs; it is never a live
  restart, since a checkpoint's name count may be behind the one the instance had reached
  ([`P1-ADR-TheLogIsTheRecord`](../docs/ADRs/P1-ADR-TheLogIsTheRecord.md)).

**The retention** is how many finished sessions the instance keeps for its queries. A session finishes when it
completes (§7) or is cancelled (§8), and is evicted: it is no longer open. Where the retention is above 0 its record
is kept, retained, as it stood when it finished, with the number of the receipt that finished it; and when the
retained records then outnumber the retention, the one that finished earliest is let go. A retention of 0 keeps
none. To every command but a query a retained session is not open, so what is kept changes what a query answers
and nothing else ([`P1-ADR-RetentionGovernsQueriesNeverEmission`](../docs/ADRs/P1-ADR-RetentionGovernsQueriesNeverEmission.md)).
The retention has no default: every instance states its own.

**The bound** is how many sessions may be open at once, or none for no bound. It counts open sessions only, and a
bound of 0 is a bound like any other: every start, and every adopt of a session not open, is refused.

## 3. One entry point, one output

**Every call is a command, and every command is answered with a receipt**: an output and a number. The number is
one more than the instance's previous receipt's, the first being 1, whatever the output — a refused command and a
query are numbered like the rest ([`P1-ADR-OneEntryPointOneOutput`](../docs/ADRs/P1-ADR-OneEntryPointOneOutput.md)).

**The answer is total and deterministic.** Every command has exactly one output in every state, given by the first
row of [`matrix.json`](matrix.json) that applies, in the matrix's order. The same state and the same command give
the same receipt.

**Every call is reported to the instance's observer**, given when the instance is created: the command and its
receipt, exactly once, by the time the receipt is returned — a refused command and a query like the rest. The
observer is told and answers nothing, so it cannot change an output; an instance with no use for one is given an
observer that does nothing ([`P1-ADR-EveryCallIsObserved`](../docs/ADRs/P1-ADR-EveryCallIsObserved.md)).

**A refusal changes nothing** but the receipt number: no record is made or changed, and no name is minted. Nor do
`RecoveryNotStarted` and `RecoveryUnaffected`: an output describes what happened to a session, and those two say
that nothing did. **Nor does a query** (§9, §10): it answers, and changes nothing but the receipt number.

## 4. Names

**A session's name is the instance id that named it and a count**: node, incarnation, count. The count of an
instance's first session is 1, and every session it starts takes the next. No two sessions of one instance share a
count, and no two instances share an instance id, so no two sessions anywhere share a name.

The **name count** is how many names the instance has minted; the next session it starts is named with the name
count plus one. A session's eviction does not take its name back: the name count never goes down.

**An adopted session keeps its name**: the one its peer's instance minted when it started the session. Adopting
mints nothing, and an instance never adopts a name it minted itself (§6), so no two of its open sessions ever share
a name, however they were opened.

## 5. `StartRecovery`

**Carries:** `chunks`, how many chunks the session holds; `sender`, the node that holds them; `receiver`, the node
they go to. No name: beggar names the session and returns the name. The side that holds the chunks opens the
session, since only it knows their count ([`P1-ADR-TheSenderOpensAndStatesTheCount`](../docs/ADRs/P1-ADR-TheSenderOpensAndStatesTheCount.md));
beggar does not check which side starts, so `sender` has no default.

**Checked in this order:**

| row | situation | output |
|---|---|---|
| `no-chunks` | `chunks` is 0 | `RecoveryRejected(NoChunks)` |
| `self-addressed` | `sender` is `receiver` | `RecoveryRejected(SelfAddressed)` |
| `not-a-party` | this instance's node is neither `sender` nor `receiver` | `RecoveryRejected(NotAParty)` |
| `start-registry-full` | the open sessions number the bound | `RecoveryRejected(RegistryFull { capacity })`, the bound |
| `start-started` | otherwise | `RecoveryStarted`, carrying the new session's name |

**Started:** the instance mints the next name, and opens a session under it: its `chunks`, `sender` and
`receiver` as given, and an empty chunk set.

## 6. `AdoptRecovery`

**Carries:** `session_id`, the name the session was started under; `chunks`, `sender` and `receiver`, as its starter
stated them. The other side of a transfer adopts the session its peer started, under that name and with that count,
which reach it with what the peer sends ([`P1-ADR-TheSenderOpensAndStatesTheCount`](../docs/ADRs/P1-ADR-TheSenderOpensAndStatesTheCount.md)).

**Checked in this order:**

| row | situation | output |
|---|---|---|
| `no-chunks` | `chunks` is 0 | `RecoveryRejected(NoChunks)` |
| `self-addressed` | `sender` is `receiver` | `RecoveryRejected(SelfAddressed)` |
| `not-a-party` | this instance's node is neither `sender` nor `receiver` | `RecoveryRejected(NotAParty)` |
| `own-session` | `session_id` was minted by this instance: its node and incarnation are this instance's | `RecoveryRejected(OwnSession)` |
| `adopt-unaffected` | a session named `session_id` is open, with the same `chunks`, `sender` and `receiver` | `RecoveryUnaffected` |
| `adopt-conflicted` | a session named `session_id` is open, with a different `chunks`, `sender` or `receiver` | `RecoveryConflicted { chunks, sender, receiver }`, the open session's |
| `adopt-registry-full` | no session named `session_id` is open, and the open sessions number the bound | `RecoveryRejected(RegistryFull { capacity })`, the bound |
| `adopt-started` | otherwise | `RecoveryStarted`, carrying `session_id` |

**Started:** the instance opens a session under `session_id`, with its `chunks`, `sender` and `receiver` as given
and an empty chunk set. It mints nothing: the name count is unchanged.

**Its own session.** A session is adopted from the peer that started it. A name this instance minted is one it
started, or one it will mint: adopting it would let a later start mint the same name again, and two open sessions
share it. So it is refused.

**Unaffected and conflicted.** The same adopt reported twice leaves the open session as it was. One that disagrees
with the open session on its count or its ends changes nothing, and answers with what the open session holds, so the
log can tell a repeated report from a contradicting one ([`P1-ADR-OneEntryPointOneOutput`](../docs/ADRs/P1-ADR-OneEntryPointOneOutput.md)).
`RecoveryConflicted` terminates nothing: the open session stays open.

**A late duplicate.** An adopt whose session already completed or was cancelled here finds none open, and opens it
again: an evicted session leaves no trace to refuse it by. The session reopened never completes unless its chunks come again;
it is an abandoned session like any other, and the caller's cancellation policy ends it
([`P1-ADR-CancelEvictsWithoutATombstone`](../docs/ADRs/P1-ADR-CancelEvictsWithoutATombstone.md)).

## 7. `SendRecoveryPartial` and `ReceiveRecoveryPartial`

**Carry:** `session_id`, the session's name; `first` and `last`, the run of chunks, which the caller moved and
validated as one unit ([`P1-ADR-APartialIsARunOfChunks`](../docs/ADRs/P1-ADR-APartialIsARunOfChunks.md)); and
`sender` and `receiver`, the session's ends as the caller has them. A send is reported on the session's sender once
the run is sent; a receive on its receiver once the run has arrived and been validated.

**Checked in this order, both of them:**

| row | situation | output |
|---|---|---|
| `empty-run` | `last` is below `first` | `RecoveryRejected(EmptyRun { first, last })`, the run's |
| `partial-not-started` | no open session has the name `session_id` | `RecoveryNotStarted` |
| `misattributed` | `sender` or `receiver` differs from the session's | `RecoveryRejected(Misattributed { sender, receiver })`, the session's |
| `wrong-side` | reported on the wrong end: a send where this instance's node is the session's receiver, a receive where it is the session's sender | `RecoveryRejected(WrongSide)` |
| `chunk-out-of-range` | the run reaches outside `1..=chunks`: `first` is 0, or `last` is above the session's `chunks` | `RecoveryRejected(ChunkOutOfRange { first, last, chunks })`, the run's and the session's `chunks` |
| `partial-advanced` | the run holds a chunk not yet in the chunk set, and the chunk set is not then full | `RecoveryAdvanced { held, chunks }` |
| `partial-completed` | the run holds a chunk not yet in the chunk set, and the chunk set is then full | `RecoveryCompleted` |
| `partial-unaffected` | otherwise: every chunk of the run is already in the chunk set | `RecoveryUnaffected` |

**The chunk set** is the chunk ids the session has accounted at this end — on the session's sender, the chunks it
has sent; on its receiver, the chunks it has received. A run adds those of its chunks the chunk set does not hold yet. A chunk is held once however often it is
reported, and runs may arrive in any order: overlapping, repeated, or leaving gaps.

**The wrong side.** Every open session has this instance's node as exactly one of its ends. A send belongs on the
session's sender and a receive on its receiver, so a send reported where the node is the receiver, or a receive
where it is the sender, is on the wrong end.

**A run out of range is refused whole**, even where part of it lies within `1..=chunks`: the caller presented it as
one unit, and nothing of it is added.

**Advanced** carries `held`, how many chunks the chunk set holds once the run is added, and `chunks`, the
session's.

**Completed:** the chunk set is full when it holds every chunk of `1..=chunks` — when it holds `chunks` of them, and never
merely because a run's `last` reached `chunks`. The session is evicted: it is no longer open, and to every command
but a query it leaves no trace — its record is kept only if the retention keeps it (§2) — so a later partial naming
it answers `RecoveryNotStarted`
([`P1-ADR-CancelEvictsWithoutATombstone`](../docs/ADRs/P1-ADR-CancelEvictsWithoutATombstone.md)). Its place under
the bound is free again; its name stays minted.

## 8. `CancelRecovery`

**Carries:** `session_id`, the session's name; and `reason`, why the caller abandons it — a value of the caller's
own enumeration, with or without a payload, never a string. Beggar records the reason in the receipt and never
reads it: why a session ends is the caller's to know and the log's to keep
([`P1-ADR-CancelEvictsWithoutATombstone`](../docs/ADRs/P1-ADR-CancelEvictsWithoutATombstone.md)).

**Checked in this order:**

| row | situation | output |
|---|---|---|
| `cancel-cancelled` | a session named `session_id` is open | `RecoveryCancelled(reason)`, the reason given |
| `cancel-unaffected` | otherwise | `RecoveryUnaffected` |

**Cancelled:** the session is evicted, as a completed one is: no longer open, no trace left to any command but a
query, its record kept only if the retention keeps it (§2), its place under the bound free again and its name still
minted. A cancel is local — telling the other end is the
caller's — and carries no ends to check: it names a session, at whichever end the instance holds it.

**A cancel wins once accepted.** A partial that arrives after it, the session's final one included, finds none
open and answers `RecoveryNotStarted`: no completion follows an accepted cancel. A partial that arrived before it
stands: a session the partial completed is no longer open, and the cancel is unaffected.

**None is an error.** A cancel of a session never opened here, already completed, already cancelled, or racing its
final partial each has its row; an abandoned session ends only by the caller's cancellation policy, since beggar
times nothing.

**The reason in the vectors** is an enumeration of their own, standing for any caller's: `Unservable`,
`Superseded`, `PeerFailed`, and `Stalled { held }`, which carries a count. Its values are the four reasons a
session is abandoned that the ADR names; an implementation maps them to its own reason type.

## 9. `ListRecoveries`, `ShowRecovery` and `ListMissingChunks`

**`ListRecoveries`** carries nothing, and answers `RecoveriesListed { sessions }`: the names of the open sessions,
in canonical order. A retained session is not open, and is not listed.

**`ShowRecovery`** and **`ListMissingChunks`** each carry `session_id`, and read the session of that name — the open
one if there is one, otherwise the retained one that finished last. Checked in this order:

| row | situation | output |
|---|---|---|
| `query-not-started` | no session named `session_id` is open or retained | `RecoveryNotStarted` |
| `recovery-shown` | otherwise, for `ShowRecovery` | `RecoveryShown { record, finished }` |
| `missing-chunks-listed` | otherwise, for `ListMissingChunks` | `MissingChunksListed { missing }` |
| `recoveries-listed` | always, for `ListRecoveries` | `RecoveriesListed { sessions }` |

**`RecoveryShown`** carries the session's record and `finished`: null for an open session, and for a retained one
the number of the receipt that finished it. A retained record's chunk set is full exactly when it completed.

**`MissingChunksListed`** carries `missing`: the chunk ids of `1..=chunks` the session's chunk set does not hold,
as their maximal runs (§1). A session that completed is missing none; one that has received nothing is missing one
run, `1..=chunks`, however many chunks it holds.

## 10. `ShowRegistry`, the checkpoint

**Changes nothing** but the receipt number. **Answers `RegistryShown`**, carrying the checkpoint: the instance id,
the retention and the bound; every open session's record, and every retained one's with the number of the receipt
that finished it; the name count; and the next receipt's number.

**The checkpoint describes the state after its own call:** its next receipt number is one more than the number of
the receipt that carries it. An instance restored from it carries on from the next call.

**A session's record** is its name, `chunks`, `sender`, `receiver`, and its chunk set: chunk ids, each within
`1..=chunks`, in ascending order. The records come in the canonical order of their names (§1), started and adopted
alike, the open and the retained each listed apart. A name may be both open and retained — a late duplicate adopt
reopening a finished session — and retained more than once. An open session's chunk set is never full: the run that fills it completes the
session and evicts it.

## 11. Conformance

**A vector is an initial state, steps, and a final state.** The initial state is an origin: fresh, or a checkpoint.
Each step is a command and the receipt it must be answered with. The final state is the checkpoint a
`ShowRegistry` issued after the last step must answer — its own receipt numbered one more than the last step's, so
its next receipt number is one more again. A vector states nothing of an implementation's representation: it
checks state the way a caller would.

**Each vector names the rows of the matrix it exercises.** Every vector is valid against the schema's `vector`; the
matrix is valid against its `matrix`; and every row of the matrix is exercised by at least one vector.

**An implementation conforms** when, created from each vector's origin with an observer that does nothing and given
each step's command in turn, it answers each with the step's receipt, and the closing `ShowRegistry` with the final
state; when, created with an observer that keeps what it is told, it has reported each of those calls with the
receipt it answered, in order, and nothing else; and when, created from the origin with its retention raised to
`18446744073709551615`, it answers every step but a query's with the step's receipt — what is kept changes what a
query answers and nothing else.
