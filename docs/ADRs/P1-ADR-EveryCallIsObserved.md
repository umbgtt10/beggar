# P1-ADR-EveryCallIsObserved

- **Status:** Accepted
- **Date:** 2026-11-09
- **Priority:** P1 (derived)

## Context

The log is the record ([P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md)):
every call is answered with a receipt and logged as `(number, command,
output)`. With the receipt only returned, the log holds every line only if
every call site in every caller remembers to write what it was handed, and a
call site that forgets leaves a gap the numbers prove and nothing fills.
`faction` meets the same hole with an observer injected at construction and
told of every command on every path, its `P2-ADR-TotalObservability`.

## Decision

Beggar is constructed with an `Observer`, through its one constructor, and
reports every call to it exactly once, by the time `process` returns: the
command and its receipt, the line the log holds. The receipt is still
returned. The observer is told, and answers nothing.

## Forcing constraints / Evidence

**Reporting at the one exit makes it total by construction.** Every command is
answered by a receipt ([P1-ADR-OneEntryPointOneOutput](P1-ADR-OneEntryPointOneOutput.md)),
so one call where the receipt is made reports every command — a query and a
refusal included — and a command added later is reported without anyone
remembering to.

**One method, not faction's three.** Faction's observer has a method for each
of its exits: a transition, a query, a rejection. Beggar has one exit and one
form of answer, so its observer has one method, and the output says which
kind of call it was.

**Answering nothing keeps beggar the accountant.** An observer that returned
something would be a dependency beggar decides with. This one returns
nothing, so every output stays a function of the command and the state
([P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md)), and a replay
rebuilds it without one.

**Told by reference.** The observer is lent the command and the receipt: one
that logs formats them and keeps nothing, one that records clones what it
keeps, and no checkpoint is copied for an observer that discards it.

## Rejected alternatives

**The receipt only returned, and the caller logging it** — the stance
[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) first took.
Rejected: the record then rests on every call site.

**A method per kind of output.** Rejected: beggar has one exit, and a method
per output would be a second matrix to keep in step with the first.

**Observing the state.** Rejected for the reason the log does not hold it: it
is derivable ([P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md)).

## Consequences

[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) is amended: the
observer is beggar's one injected dependency, and nothing else is injected.

The observer is a `Box<dyn Observer>`, as faction's is, and `NoOpObserver` is
provided for a caller with no use for one, as faction provides its own.

The observer is in the specification, from 0.2.0: every implementation reports
every call. The conformance suite runs each vector twice — with an observer
that does nothing, checking the receipts, and with one that records, checking
it was told exactly what was answered. Neither stubs anything: an observer
that answers nothing has nothing to fake.

## Enforcement

`process` calls the observer once, after the output is decided and before the
receipt is returned. The unit tier reports a refusal, a start, a full registry
and a query; the conformance tier checks every vector's calls as reported; the
property tier, every call of random sequences. The coverage gate over the
conformance tier alone reaches the call and `NoOpObserver`.

## Related

[P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md) — what the observer is
told is the line the log holds.
