# P1-ADR-TheLogIsTheRecord

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

Beggar's decisions have to be reconstructible after the fact, on a platform
where the node whose record matters most is the one least able to send it. On
the reference hardware, nodes outside a partition lose 0.05% of a periodic
probe; the severed node loses 2.8% to 84.6% across a run window, and inside
the partition effectively all of it — what survives is the transmit ring's
content at the heal. And on a board a full-state dump per call competes with
the very lines it should be read against.

## Decision

Every call is emitted as `(ordinal, command, output)`; the state is rebuilt by
replaying them, resynchronised from a checkpoint the caller asks for, and the
ordinal — the caller's, required to rise — makes every gap detectable.

## Forcing constraints / Evidence

**The state is derivable.** `process` is total and deterministic
([P1-ADR-OneEntryPointOneOutput](P1-ADR-OneEntryPointOneOutput.md)), so the
state after a call follows from the state before it and the command. Commands
and outputs are small and bounded; a replay needs nothing else.

**A lossy log looks complete without an ordinal.** A replay built from a
capture with lines missing omits events silently and stays self-consistent.
With an ordinal on every command, a gap is detectable, the replay can report
*reconstructed X of Y*, and it knows when to resynchronise.

**The sender is the witness.** `SendRecoveryPartial` is fed on the serving
node, and the serving node is not the severed one. So the existence, size,
peer and progress of every transfer are reconstructible from the nodes that
can still speak; what the silence costs is the receiver's timings and its
local view.

## Rejected alternatives

**The full state on every call.** Rejected: on a board it competes with the
lines it exists to be correlated with, and the state is derivable anyway.

**An ordinal that must be exactly one more.** Rejected: one number the caller
skipped would make every later ordinal wrong too, and the registry would
refuse everything after it, for good.

**A checkpoint cadence held by beggar.** Rejected: when to checkpoint is a
policy, and beggar holds none.

**Instrumenting only the receiver.** Rejected: it puts the whole record on the
one node that cannot speak.

**A reliable transport for the log, or a larger transmit ring.** Rejected: a
severed port is a connectivity failure, not a reliability one — a reliable
transport recovers the 0.05% and none of the rest, and brings head-of-line
blocking onto the log path, which has already cost this family of systems a
stall that read as a timing problem. Eight times the ring buys single-digit
percentages of a multi-minute partition, because the binding constraint is the
partition's length. The loss is accepted as a property of the platform.

## Consequences

The ordinal must be after the last accepted. A command whose ordinal is not is
recorded and rejected, and leaves the last where it was; every other command
moves it, whatever its output. A gap proves a lost line only where the caller
numbers beggar's commands one by one — the caller's to do, and to state.

The checkpoint is a query, the whole registry: every open and retained record,
the count, the last ordinal. The caller asks for it as often as it chooses and
logs it like any other output.

The registry keeps membership, not order. Which chunks arrived is the record's;
when is the log's, answered by the ordinals, and an ordered structure in
memory would duplicate it.

Both partials are kept: the send, on the serving node, is the witness; the
receive is the local view of the node that may not be heard.

## Enforcement

The emission is what `process` returns, and the caller logs it. The ordinal
rule is the matrix's first row; the conformance vectors replay command
sequences and compare every output.

## Related

[P1-ADR-RetentionGovernsQueriesNeverEmission](P1-ADR-RetentionGovernsQueriesNeverEmission.md)
— why a terminal output is emitted whatever is retained.
