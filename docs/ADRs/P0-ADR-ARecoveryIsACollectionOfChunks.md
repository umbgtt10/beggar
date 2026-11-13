# P0-ADR-ARecoveryIsACollectionOfChunks

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P0 (axiom)

## Context

Beggar serves recovery in two consensus protocols whose recoveries differ in
everything a protocol cares about. IBFT's node behind asks for blocks, and the
node it asks answers with up to eight at a time. Raft's leader pushes a lagging follower the log entries it lacks,
and has no fixed end: entries keep arriving, and a rejection steps it back.
Either may send a snapshot in pieces. A bookkeeper that knew any of this would
be rewritten for the next protocol, the next transfer mode, the next way of
cutting a transfer up.

## Decision

A recovery is just a collection of chunks: beggar knows sessions, chunk
counts, chunk ids and node ids, and never what a chunk holds, how it moves, in
what order, under what policy or which protocol — and no bits or bytes.

## Forcing constraints / Evidence

The vocabulary is the enforcement. There is no type in which a block, an
entry, a height or a wire format could be expressed, so protocol independence
holds by construction rather than by discipline. And the shape is the one
thing the recoveries share: an answer of up to eight blocks, a follower's
missing entries and a snapshot in a hundred pieces are each a session of chunks.

## Rejected alternatives

**Fitting beggar to the transfers that already have a fixed end** — snapshots,
block requests — and leaving log recovery out. Rejected: whether a transfer
has a fixed end is how its caller frames it. A log recovery becomes a
collection once the caller decides what one recovery covers.

**A bitmap for the chunks received.** Rejected: a byte-level representation in
a crate that deals in none, and its size follows the chunk count. The received
set is chunk ids, kept as ranges, so its size follows the gaps.

**A small chunk count** (u16, sized for a block request). Rejected: a caller
may make one log entry one chunk. `chunks` and chunk ids are u64.

## Consequences

Turning a stream into a collection is the caller's work. Raft's control layer
decides what one recovery covers — what was missing when it opened, with
entries appended later left to ordinary replication — and a range found wrong
is a cancel, its reason saying why, and a new session.

Push and pull look the same: in both, the side that holds the chunks opens the
session and states their count, and under pull the side behind only asks
([P1-ADR-TheSenderOpensAndStatesTheCount](P1-ADR-TheSenderOpensAndStatesTheCount.md));
the same registry runs at both ends, and nothing in the commands says which it
was.

What a chunk holds is the caller's to choose — a block, an entry, a batch, a
slice of a snapshot — and so is the chunk size, which, with the caller's
cancellation policy, bounds how many gaps a session can hold.

## Enforcement

The crate's types: no command or output carries a payload, a byte, a slice or
a protocol type. The conformance vectors give a session's chunks as ids.

## Related

[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) — the caller
that frames a transfer as chunks is the control layer that ADR puts above
beggar.
