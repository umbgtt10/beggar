# P0-ADR-AccountsNeverDecides

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P0 (axiom)

## Context

Recovery needs decisions — that a node is behind, which peer to ask, when to
retry, when to fall back to a snapshot, when to give up — and accounting —
which transfers are open and which chunks have moved. Kept in one crate, the
accounting inherits everything the decisions need: a clock, a transport,
heights, peer statuses and a policy.

## Decision

Beggar accounts and never decides: it detects nothing, chooses no peer, times
nothing and holds no policy, and it has zero injected dependencies — no clock,
no storage, no transport, no sink and no admission check.

## Forcing constraints / Evidence

**Admission is not a boundary, because beggar never sees an entry.** Whether
an entry is valid and who may serve it look like a trait to inject; they are
the caller's. The caller validates and only then reports that a chunk
arrived; a rejected entry is never reported.

**Owning no timer and no socket collapses the entrances.** IBFT's recovery, as
built, had five separate ways into issuing a request, only one of them the
poll. A core with no timer cannot have five: "the retry timeout expired" is a
command from the caller, not a call site inside beggar.

**What decides has states, and a registry refuses them.** The consensus-unaware
part of IBFT's recovery, once extracted, detects lag, chooses a peer from the
statuses it has seen, retries on a timeout and excludes a peer that refuses —
with an episode open or closed and a request targeted or blind. That is a
state machine over heights, and it sits above beggar, not in it.

## Rejected alternatives

**Folding the control layer into beggar.** Rejected: it brings heights, a
clock, refusal and states — each refused by
[P0-ADR-ARegistryNotAStateMachine](P0-ADR-ARegistryNotAStateMachine.md) or
[P0-ADR-ARecoveryIsACollectionOfChunks](P0-ADR-ARecoveryIsACollectionOfChunks.md).

**An injected admission policy.** Rejected: there is nothing to admit.

**Beggar enforcing limits** — a cap on sessions, a timeout on a stalled one.
Rejected: a limit is a policy, and its right value is the caller's.

## Consequences

Three layers: beggar accounts; a control layer above it — consensus-unaware,
chain-aware — holds heights, peer statuses, retries, refusals and the
cancellation policy; the protocol sits above that. The dependency runs one
way: the control layer depends on beggar, never the reverse.

Every policy is the caller's, and the caller states it — the cancellation
policy above all, because a cancel nobody decides to send leaves sessions
accumulating exactly as no cancel command would.

What would need storage comes in from the caller: an incarnation, when the
instance is created. What needs only a count, beggar keeps: the names it mints
and the numbers on its receipts. Outputs are returned, never pushed to a
sink. The peer set is supplied per call and never cached, so beggar survives
membership changes without learning of them.

With nothing injected, any implementation in any language is tested with
nothing to stub.

## Enforcement

The crate's manifest has no dependencies, and beggar calls out through no
trait. It is generic only over the caller's cancel reason, an enum that
carries no behaviour.

## Related

[P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md)
— zero injected dependencies is what makes a stub-free conformance suite
possible.
