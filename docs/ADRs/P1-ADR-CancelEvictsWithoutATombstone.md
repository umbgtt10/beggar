# P1-ADR-CancelEvictsWithoutATombstone

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

A session has to be abandonable, for four independent reasons: it was opened
for a node that turns out unservable; the node fell further behind and needs a
larger session; the serving peer was banned, refused or failed midway; and any
timeout policy needs something to act through. Beggar owns no clock, so it
cannot time a session out itself. And after a session ends, late traffic for
it is certain: the transport retries undelivered datagrams indefinitely and
delivers a partition's oldest at the heal.

## Decision

`CancelRecovery` evicts the session and leaves no tombstone, a cancel wins once
accepted, and ending abandoned sessions is the caller's stated cancellation
policy.

## Forcing constraints / Evidence

**Beggar cannot time anything out.** It owns no clock
([P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md)), so
`CancelRecovery` closes the mechanism gap and not the policy gap. A cancel
nobody decides to send leaves sessions accumulating exactly as no cancel
command would.

**A canceller knows what beggar does not** — that is why it cancels. Arrival
order is not a contract, so a final partial racing a cancel must not override
it.

## Rejected alternatives

**A tombstone for every finished session**, to tell "cancelled" from "never
existed". Rejected: tombstones accumulate as retained records do, and buy a
*why* beggar has no use for. Beggar can say a session is absent, not why.

**The highest count adopted from each peer instance**, to refuse a late
duplicate adopt. Rejected: it would also refuse a genuine session that arrived
out of order.

**A completion emitted after an accepted cancel.** Rejected: transport timing
would override the canceller.

**Timeouts inside beggar.** Rejected: they need a clock, and their length is a
policy.

## Consequences

A partial after a cancel cannot be told from one for a session that never
existed: `RecoveryNotStarted` covers both. It is ordinary traffic on a healthy
node — a duplicate final chunk after eviction produces it — so it is never
alerted on, and a count of it is paired with a witness, or it reads as
catastrophic loss when it is a capacity signature.

A late duplicate `AdoptRecovery` reopens a finished session whose chunks never
come. Beggar cannot tell it from a new session; it is an abandoned session like
any other, and the caller's cancellation policy ends it.

A cancel is legal in four situations, none an error: the session unknown (it
may already be evicted), already completed, already cancelled, or racing its
final partial. A cancel is local; telling the other end is the caller's.

The cancel's reason is a value of the caller's enum — with or without a
payload, never a string. Its variants are why recoveries end, which beggar
does not know; so beggar is generic over it, stores it, emits it and never
reads it. It is the only place a peer's unwillingness survives in the ledger:
"the peer said no" and "the peer said nothing" are otherwise the same absence.

`RecoveryAdvanced` carries the chunks received and expected, so a transfer
stalled at 97 of 100 is visible to the policy that has to act on it.

## Enforcement

The record has no tombstone state; the invariant that no record survives a
terminal output; the matrix's rows for a cancel and for a partial whose
session is not open.

## Related

[P1-ADR-RetentionGovernsQueriesNeverEmission](P1-ADR-RetentionGovernsQueriesNeverEmission.md)
— retention keeps finished records for queries only, which is not a tombstone.
