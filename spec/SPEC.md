# beggar — the specification

**Version 0.2.0.** The four parts of this specification are versioned together: this prose, the data model in
[`schema.json`](schema.json), the decision table in [`matrix.json`](matrix.json) and the conformance vectors in
[`vectors/`](vectors/). It is normative; the `beggar` crate is one implementation of it, and where the two disagree
the specification is right ([`P0-ADR-SpecificationIsNormativeCrateIsReference`](../docs/ADRs/P0-ADR-SpecificationIsNormativeCrateIsReference.md)).

**What 0.2.0 covers:** an instance, created fresh or from a checkpoint, and the observer it reports every call to;
`StartRecovery` with every row it has; and `ShowRegistry`, the checkpoint. The other commands arrive in later
versions, each with all of its rows. 0.2.0 is 0.1.0 with the observer: the matrix, the schema and the vectors are
unchanged but for the version.

## 1. Conventions

**Every u64 is a string of decimal digits**, with no sign, no leading zero and no exponent, its value within
`0..=18446744073709551615`. Many JSON parsers read numbers as 64-bit floats, exact only to 2⁵³; an incarnation drawn
from a nanosecond timestamp is already past it.

**Every value of an enumeration is an object with exactly one member**, named for the value: `{"ShowRegistry": {}}`,
`{"RecoveryRejected": {"RegistryFull": {"capacity": "2"}}}`. A value that carries nothing carries the empty object.

**Names are the vocabulary's.** Commands are imperative and outputs past participle, so what a caller asked for
and what happened cannot be confused.

**Canonical order.** Wherever an answer lists records, they come in ascending order of their names, compared as
`(node, incarnation, count)` numerically; a list of chunk ids comes in ascending order. An answer has exactly one
form, so two implementations' logs of the same calls are identical.

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

**The retention** is how many finished sessions the instance keeps for its queries. In 0.1.0 no session finishes,
so the retained records are always none; the retention is carried and shown.

**The bound** is how many sessions may be open at once, or none for no bound. It counts open sessions only, and a
bound of 0 is a bound like any other: every start is refused.

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

**A refusal changes nothing** but the receipt number: no record is made, no name is minted.

## 4. Names

**A session's name is the instance id that named it and a count**: node, incarnation, count. The count of an
instance's first session is 1, and every session it opens takes the next. No two sessions of one instance share a
count, and no two instances share an instance id, so no two sessions anywhere share a name.

The **name count** is how many names the instance has minted; the next session it opens is named with the name count
plus one.

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
`receiver` as given, and no chunk received.

## 6. `ShowRegistry`, the checkpoint

**Changes nothing** but the receipt number. **Answers `RegistryShown`**, carrying the checkpoint: the instance id,
the retention and the bound; every open session's record and every retained one's; the name count; and the next
receipt's number.

**The checkpoint describes the state after its own call:** its next receipt number is one more than the number of
the receipt that carries it. An instance restored from it carries on from the next call.

**A session's record** is its name, `chunks`, `sender`, `receiver`, and the chunks received: a set of chunk ids,
given in ascending order, each within `1..=chunks`. In 0.1.0 it is always empty.

## 7. Conformance

**A vector is an initial state, steps, and a final state.** The initial state is an origin: fresh, or a checkpoint.
Each step is a command and the receipt it must be answered with. The final state is the checkpoint a
`ShowRegistry` issued after the last step must answer — its own receipt numbered one more than the last step's, so
its next receipt number is one more again. A vector states nothing of an implementation's representation: it
checks state the way a caller would.

**Each vector names the rows of the matrix it exercises.** Every vector is valid against the schema's `vector`; the
matrix is valid against its `matrix`; and every row of the matrix is exercised by at least one vector.

**An implementation conforms** when, created from each vector's origin with an observer that does nothing and given
each step's command in turn, it answers each with the step's receipt, and the closing `ShowRegistry` with the final
state; and when, created with an observer that keeps what it is told, it has reported each of those calls with the
receipt it answered, in order, and nothing else.
