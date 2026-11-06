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

Every call is answered with a receipt — its output and a number the instance
counts itself, one more than the last — and logged as `(number, command,
output)`; the state is rebuilt by replaying the receipts, resynchronised from a
checkpoint the caller asks for, and a gap in the numbers proves a lost line.

## Forcing constraints / Evidence

**The state is derivable.** `process` is total and deterministic
([P1-ADR-OneEntryPointOneOutput](P1-ADR-OneEntryPointOneOutput.md)), so the
state after a call follows from the state before it and the command. Commands
and outputs are small and bounded; a replay needs nothing else.

**A lossy log looks complete without numbers.** A replay built from a capture
with lines missing omits events silently and stays self-consistent. With a
number on every receipt, a gap is detectable, the replay can report
*reconstructed X of Y*, and it knows when to resynchronise.

**Beggar can count its calls itself.** A number is a count, not a time: it is
state, like the count names are minted from, so it needs no clock, is
deterministic and is restored with the checkpoint. And only a number beggar
counts is one more on every call by construction, so a gap proves a lost line
whatever the caller does.

**The sender is the witness.** `SendRecoveryPartial` is fed on the serving
node, and the serving node is not the severed one. So the existence, size,
peer and progress of every transfer are reconstructible from the nodes that
can still speak; what the silence costs is the receiver's timings and its
local view.

## Rejected alternatives

**The full state on every call.** Rejected: on a board it competes with the
lines it exists to be correlated with, and the state is derivable anyway.

**An ordinal the caller supplies with every command.** Rejected. A gap would
prove loss only where the caller numbered beggar's commands one by one;
requiring exactly one more would let a single skipped number make the registry
refuse everything after it, and requiring only a larger one would leave gaps
unprovable. It adds an input, a refusal and a row of the matrix. What it caught
that nothing now does — a caller handing in the same command twice under the
same ordinal — is a caller's bug, not traffic, and both calls are in the log
under consecutive numbers.

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

`process` returns a `Receipt<R>`, carrying the `number` and the `output`. Every
call gets one, a refused one included. An instance's first receipt is number 1;
the instance id — node and incarnation — tells two instances' receipts apart,
so a log is ordered, instance by instance, by its numbers.

Placing beggar's receipts among the caller's own events is the caller's: it
logs each receipt within its own sequenced log.

The checkpoint is a query, the whole registry: every open and retained record,
the count names are minted from, and the next receipt's number. The caller
asks for it as often as it chooses and logs it like any other receipt.

The registry keeps membership, not order. Which chunks arrived is the record's;
when is the log's, answered by the receipt numbers, and an ordered structure in
memory would duplicate it.

Both partials are kept: the send, on the serving node, is the witness; the
receive is the local view of the node that may not be heard.

## Enforcement

The receipt is what `process` returns, and the caller logs it. The property
tests check that every call's number is one more than the last, refused calls
included; the conformance vectors give every expected receipt.

## Related

[P1-ADR-RetentionGovernsQueriesNeverEmission](P1-ADR-RetentionGovernsQueriesNeverEmission.md)
— why a terminal output is logged whatever is retained.
