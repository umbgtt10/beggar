# P1-ADR-OneEntryPointOneOutput

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

A bookkeeper reached through several entrances, one that throws on bad input,
or one whose queries sit beside its commands, has behaviour that cannot be
enumerated, replayed or written down as one function. IBFT's recovery, as
built, had five ways into issuing a request.

## Decision

Beggar has one entry point, `process(command)`, which takes `&mut self` and
returns exactly one output for every command — a query and a refused command
included — on a numbered receipt, and never an exception.

## Forcing constraints / Evidence

Totality is what the two other promises rest on. The matrix is the
specification only if every command in every situation has a row
([P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md));
the log is the record only if every call leaves a line
([P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md)). A call that throws,
or returns nothing, is one neither can hold.

## Rejected alternatives

**Errors as exceptions or a `Result`.** Rejected: a refused call drops out of
the record, or the function stops being total.

**Returning the new state.** Rejected: the function is total and
deterministic, so the state after a call follows from the state before it and
the command; returning it would hand the log what a replay already rebuilds.

**Queries as methods of their own.** Rejected: they would escape the
numbering and the log, and the checkpoint is a query that has to be in the
log.

**An output per kind of refusal.** Rejected: it widens the matrix for no gain.
One `RecoveryRejected` carries a specific `Rejection`, a payload where the log
needs one, as the cancel carries its reason.

## Consequences

Every refusal is `RecoveryRejected` carrying a `Rejection`: no chunks, a
session that is self-addressed or does not involve this node, an adopt of a
session this instance minted itself, a full registry, an empty run, a partial misattributed, on the wrong side or out of range. Each is specific, and
carries what the log needs to tell a degenerate caller from a malicious peer:
a misattributed partial carries the session's own ends.

The matrix is checked in a fixed order — whether the command is well formed,
then the session, then the partial's attribution, then the effect — and the
first row that applies is the output.

An output describes what happened to the session, never which command arrived:
the command is already in the logged tuple. No output exists merely to tell
two commands apart, which is why `RecoveryUnaffected` serves three rows.

The four queries — the open sessions, one session's record, its missing chunk
ids, the whole registry — are commands: each gets a receipt and is logged.

## Enforcement

The matrix in `../DESIGN.md` §4, one row for every command in every situation;
the conformance vectors and an exhaustive property test over small `chunks`.

## Related

[P0-ADR-ARegistryNotAStateMachine](P0-ADR-ARegistryNotAStateMachine.md) —
why there is no set of admissible commands to return beside the output.
