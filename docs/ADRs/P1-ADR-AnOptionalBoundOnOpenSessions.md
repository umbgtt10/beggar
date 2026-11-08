# P1-ADR-AnOptionalBoundOnOpenSessions

- **Status:** Accepted
- **Date:** 2026-11-08
- **Priority:** P1 (derived)

## Context

Every open session holds a record until a terminal output evicts it. A caller
whose peers open sessions faster than they end — a buggy peer, a hostile one,
or a cancellation policy that is missing — grows the registry without limit,
and on a host with little memory it is the node that holds what the others
need whose memory runs out.

## Decision

An instance is created with an optional bound on its open sessions — none, no
bound — and a start, or an adopt of a session not open, that would exceed it is
refused with `RecoveryRejected(RegistryFull { capacity })`, changing nothing.

## Forcing constraints / Evidence

**A number fixed at construction is configuration, not policy.** Beggar does
not choose it, change it, or decide which session matters more; it counts its
open sessions against the caller's number and refuses past it, as it keeps the
caller's number of finished sessions for retention.

**A refusal belongs in the record.** A caller could bound the registry by
policing its own calls, but a refused open would then leave no line. Refused
by beggar, it is a receipt like any other, and the side that asked hears it
as the protocol's error.

## Rejected alternatives

**No bound at all.** Rejected: the caller's only defence would be to keep its
own count beside beggar's, and a refusal would be invisible in the record.

**Evicting the oldest session to make room.** Rejected: choosing which session
to give up is a policy, and it ends a transfer nobody cancelled.

**A bound on chunks or on the ranges a session holds.** Rejected: the caller's
chunk size and its cancellation policy bound those.

## Consequences

The bound counts open sessions only: a retained session answers queries and
nothing else, and retention bounds it.

An adopt of a session already open opens nothing, so its rows come first and a
full registry still answers it, unaffected or conflicted. The bound's row
follows them.

The bound is part of the instance: given at construction with the instance id
and the retention, carried in the checkpoint, changed by no command. A full
registry opens again as soon as a terminal output evicts a session.

Carrying the refusal to the side that asked is the protocol's.

## Enforcement

The invariant that the open sessions never outnumber the bound; the matrix's
rows for a start and an adopt at the bound; the exhaustive matrix over small
bounds, none included; the conformance vectors at a bound.

## Related

[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) — beggar holds
no policy; the bound is the caller's number.
