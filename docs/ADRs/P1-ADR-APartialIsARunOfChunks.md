# P1-ADR-APartialIsARunOfChunks

- **Status:** Accepted
- **Date:** 2026-11-08
- **Priority:** P1 (derived)

## Context

A caller reports chunks in the units it moves and validates them in: a page of
blocks whose certificate covers the whole range, a batch of log entries, a
message holding several pieces of a snapshot. Every call is answered with a
receipt the caller logs. Reported one chunk at a time, a unit of eight is eight
calls, eight receipts and eight lines, at both ends of every transfer — and the
granularity is one the caller never checked.

## Decision

A partial reports a run of consecutive chunk ids, `first..=last`; a single
chunk is a run of one.

## Forcing constraints / Evidence

**The run is what the caller validated.** A page accepted or refused as a unit
is one fact; reporting it chunk by chunk invents an order and a timing inside
it that nobody observed.

**The record already keeps runs.** The chunks received are held as ranges and
shown as maximal runs ([P0-ADR-ARecoveryIsACollectionOfChunks](P0-ADR-ARecoveryIsACollectionOfChunks.md)),
so a run merges in one step, and a transfer of `n` units leaves `n` receipts
however many chunks each holds.

## Rejected alternatives

**One chunk per partial.** Rejected: calls, receipts and log lines multiply by
the run's length, at both ends.

**An arbitrary set of chunk ids per partial.** Rejected: ids that are not
consecutive were not validated as one unit, and each run is its own report; a
set enlarges every command and vector for no case that needs it.

**Accepting the part of a run that falls inside the session.** Rejected: the
caller presented the run as one unit, and a unit that overruns its session is
garbage as a whole.

## Consequences

A run that reaches outside `1..=chunks`, at either end, is refused whole with
`ChunkOutOfRange { first, last, chunks }`, and nothing changes. A run whose
`last` is below its `first` is malformed and refused with `EmptyRun`, before
the session is looked at.

A run partly received already adds the chunks that are new: the output is
`RecoveryAdvanced`, or `RecoveryCompleted` if they fill the set, when any chunk
is new, and `RecoveryUnaffected` when none is. A duplicate is ordinary, whole
or in part.

One receipt per call is unchanged. What a run gives up is a timing for each
chunk inside it, which its caller did not have.

The conformance vectors give a run as `first` and `last`, each a string of
decimal digits, and a session's final chunks as a set of ids, as before.

## Enforcement

The matrix's rows for an empty run, a run out of range and a run partly
present; the exhaustive matrix over every run within, across and beyond small
`chunks`; the property tests' invariants after every run.

## Related

[P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md) — the receipt each call
leaves, which a run makes one per unit.
