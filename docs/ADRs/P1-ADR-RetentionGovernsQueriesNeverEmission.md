# P1-ADR-RetentionGovernsQueriesNeverEmission

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

A host running tests or an analysis wants finished sessions kept, to query
them; a board cannot afford them. A record kept past its session's end is
memory that accumulates: a height-keyed exclusion map in `etheram-ibft` once
held 292 orphaned entries at about 110 bytes each, fatal by fragmentation long
before size. Two shortcuts tempt: skip emitting a terminal nobody will keep,
and let a kept record change what a command answers.

## Decision

Retention — how many finished sessions beggar keeps — changes what a query can
answer and nothing else: every terminal output is emitted whatever is kept,
and to every command but a query a retained session is not open.

## Forcing constraints / Evidence

**Skipping a terminal destroys the trail where it matters.** The cluster runs
at retention 0, so a shortcut that emits only what is kept would erase every
completion and cancellation from exactly the configuration that needs them.

**Outputs that depended on retention would make the host lie about the
board.** Tests run on a host at a high retention; the board runs at 0. If a
retained record answered commands, the host's tests would say nothing about
the board.

## Rejected alternatives

**Emitting only what is retained.** Rejected, for the first reason above.

**Retained sessions answering commands** — an adopt finding a retained,
completed session, say. Rejected, for the second.

**Two mechanisms, one for hosts and one for boards.** Rejected: one knob lets
the same code serve both, high on a host and 0 on a board.

## Consequences

Both terminal outputs evict at retention 0, so the registry's size is bounded
by concurrent transfers, not by transfers ever attempted.

Retention is recorded in two columns from the first line of code — its
`Default` and its deployed value. A one-column constants table once led four
downstream records to quote a library default as deployed; a retention
default other than 0, with the cluster at 0, is the same trap. Since
2026-11-10 it has no default at all: every instance states its own, and the
first column reads none.

A finished session is kept as it stood, with the number of the receipt that
finished it, and past the retention the one that finished earliest goes;
whether it completed is its chunk set, full or not, and a cancel's reason
stays in the log.

Which sessions are open is the API's to answer; which ever existed is the
capture's. They are two questions, and neither stands in for the other.

## Enforcement

In the matrix only `ShowRecovery` and `ListMissingChunks` read a retained
record. Every conformance vector runs again with its retention raised to the
top of u64 (the specification's §11); the property and cluster simulation
tiers run random sequences and clusters at 0 and at the top; and every output
but a query's must match.

## Related

[P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md) — the record that
skipping a terminal would break.
