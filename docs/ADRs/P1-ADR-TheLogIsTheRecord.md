# P1-ADR-TheLogIsTheRecord

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

Beggar's decisions have to be reconstructible after the fact, from a log
beggar assumes nothing about: a line may be lost, and the node whose record
matters most may be the one least able to keep it. And a full-state dump per
call would cost its caller more than the lines it is meant to be read against.

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

**Both ends are witnesses.** Every transfer is accounted at both of its ends —
`SendRecoveryPartial` on the sender, `ReceiveRecoveryPartial` on the receiver —
so its existence, size, peers and progress survive the loss of either end's
log; what a lost log costs is that end's timings and its local view.

## Rejected alternatives

**The full state on every call.** Rejected: it costs more than the lines it
exists to be correlated with, and the state is derivable anyway.

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

**Instrumenting one end only.** Rejected: a transfer would then survive only
as long as that end's log.

**Relying on the log's delivery.** Rejected: how the caller carries its log is
the caller's, and beggar assumes nothing about it. A lost line is detected, not
prevented.

## Consequences

`process` returns a `Receipt<R>`, carrying the `number` and the `output`. Every
call gets one, a refused one included. An instance's first receipt is number 1;
the instance id — node and incarnation — tells two instances' receipts apart,
so a log is ordered, instance by instance, by its numbers.

Placing beggar's receipts among the caller's own events is the caller's: it
logs each receipt within its own sequenced log.

The checkpoint is a query, the whole registry: the instance id and the
retention, every open and retained record, the count names are minted from,
and the next receipt's number. The caller asks for it as often as it chooses
and logs it like any other receipt.

An instance is created from one of two origins: a fresh instance id with its
retention, or a checkpoint. Restoring a checkpoint is how a replay
resynchronises after a gap, and how a vector starts from the state it needs.
It is never a live restart: a checkpoint's name count may be behind the count
the instance had reached, so a restored instance in service would mint names
again. A node that restarts is a new instance, under a new incarnation, and
starts empty.

The registry keeps membership, not order. Which chunks arrived is the record's;
when is the log's, answered by the receipt numbers, and an ordered structure in
memory would duplicate it.

Both partials are kept: each is its end's record of what it saw.

## Enforcement

The receipt is what `process` returns, and the caller logs it. The property
tests check that every call's number is one more than the last, refused calls
included; the conformance vectors give every expected receipt; the replay tier
rebuilds every receipt from a log, detects a dropped line and resynchronises
from a checkpoint ([P2-ADR-TestingLadder](P2-ADR-TestingLadder.md)).

## Related

[P1-ADR-RetentionGovernsQueriesNeverEmission](P1-ADR-RetentionGovernsQueriesNeverEmission.md)
— why a terminal output is logged whatever is retained.
