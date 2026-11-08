# P1-ADR-TheSenderOpensAndStatesTheCount

- **Status:** Accepted
- **Date:** 2026-11-08
- **Priority:** P1 (derived)

## Context

A session's chunk count is fixed when it opens: completion is the set of chunks
received reaching it. Only the side that holds the chunks knows how many it
will send. A node that asks for what it missed does not: the side it asks
decides how much one answer carries, by its own budget, and may hold less than
was asked for; and a node that asks without knowing who is ahead does not even
know what the other side holds.

## Decision

The side that holds the chunks opens the session and states their count; the
other side adopts it, under the name and with the count it was given.

## Forcing constraints / Evidence

**A count fixed by the side that cannot know it is wrong whenever the answer
is shorter.** A session opened for eight that receives six never completes, on
either end; the side that asked would have to cancel it and open another for
what is left, and the side that answered would hold an open session it never
finishes. A multi-part recovery would become a chain of cancellations.

**Push and pull meet in one rule.** Under push the side ahead sends what the
other lacks; under pull the side behind asks, and the side ahead answers. In
both, the side ahead holds the chunks and is the one that knows their count.

## Rejected alternatives

**The side that asks opens the session with an upper bound,** and cancels on a
short answer. Rejected: every short answer becomes a cancellation, which reads
as an abandoned transfer, and the answering side's session never completes.

**A command that changes a session's count while it is open.** Rejected: it
adds a command and its rows, and a session whose expected count moves has no
fixed point at which it is complete.

## Consequences

Under pull, asking is the protocol's: the side behind sends a request that
opens no session, and its request in flight is tracked above beggar, with its
retries and its timeouts, until an answer opens one. The answering side
starts the session with the count it will send; the name and the count travel
with what it sends, and the side behind adopts the session from them.

A short answer is a smaller session, and completes.

Beggar does not check which side opens: a start names both ends, `sender` has no
default, and the rule is its callers'. It is stated here because a session's
count is only right when it is kept.

Asking again before an answer arrives can make the side asked open more than
one session for the same need; each is an open session like any other, ended
by completion or by the caller's cancellation policy.

## Enforcement

The two-ended host tests and the simulation tier open every transfer on its
sender and adopt it on its receiver; the matrix is unchanged by it.

## Related

[P0-ADR-ARecoveryIsACollectionOfChunks](P0-ADR-ARecoveryIsACollectionOfChunks.md)
— the collection whose size this fixes.
