# P1-ADR-TheAccountantNamesTheSession

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

A transfer is accounted twice: the sender's registry records what it sent,
the receiver's what it received, and the record of a transfer joins across
the two only if both name the session the same. Whoever names it decides
whether that name can repeat — and a repeated name is a session that cannot
be told from another.

## Decision

Beggar names every session: `StartRecovery` mints a name — the instance id
and a count — and `RecoveryStarted` returns it, and the peer's registry opens
the same session under it with `AdoptRecovery`.

## Forcing constraints / Evidence

**Uniqueness by construction, without coordination.** An instance id is the
node's id and an incarnation that never repeats for that node; no two
instances share one, and no instance repeats a count. So no two sessions in
the cluster share a name, and an adopted name never collides with one this
instance minted.

**A restart is where it would break.** The count starts again at every start,
while a peer may still hold the previous instance's sessions. Were node 7 to
come back under the same instance id, its first new session would share a
name with an old one; an adopt of it would find the old session open with the
same parameters, answer `RecoveryUnaffected`, and count the new transfer's
chunks onto the old one's — a completion before the chunks arrived.

## Rejected alternatives

**Names chosen by the caller.** Rejected: uniqueness becomes a discipline, and
a name reused after its session was evicted is indistinguishable from a fresh
one.

**The node's id alone as the instance id.** Rejected: it repeats at every
restart, which is the failure above.

**Beggar keeping its count across restarts.** Rejected: that needs storage,
and beggar owns none ([P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md)).

## Consequences

A name is three numbers: node, incarnation, count. The caller carries it with
every message of the transfer, as it carries everything else on the wire.

The incarnation is the caller's, supplied when the instance is created, and
the caller guarantees one thing: it never repeats for that node. How is the
caller's too:

| incarnation | needs | fits |
|---|---|---|
| the start's wall-clock timestamp | a real-time clock | hosts, pods |
| a stored count, incremented at every start | storage | a node of a protocol that persists — Raft stores its term and vote already |
| a random number drawn at start | a hardware random-number generator | a board with neither |

A timestamp is not safe on a board without a real-time clock: its time
restarts at zero, so every boot draws the same one. An ordered incarnation — a
count or a timestamp — also tells a peer that every session of an older one is
dead, which the caller's cleanup can use; a random one cannot.

`StartRecovery` cannot meet an open session, so it has one row. `AdoptRecovery`
can, and takes the three: not open, open with the same parameters, open with
different ones.

## Enforcement

`StartRecovery` carries no `session_id`. The invariant that no two open
sessions share a name, checked by the property tests and the conformance
vectors.

## Related

[P1-ADR-CancelEvictsWithoutATombstone](P1-ADR-CancelEvictsWithoutATombstone.md)
— why a late duplicate of an adopt can reopen a finished session.
