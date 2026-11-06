# P0-ADR-SpecificationIsNormativeCrateIsReference

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P0 (axiom)

## Context

Beggar's consumers run on boards and on hosts, inside more than one protocol,
and may one day be written in more than one language. A definition that lives
only in Rust is one implementation's behaviour, its bugs included, and a
second implementation has nothing to be checked against.

## Decision

The specification is normative and this crate is one conforming
implementation: a data model, one total function, and a committed,
machine-readable conformance suite in JSON — vectors of `(initial state,
command sequence) → (expected receipts, final state)` — that this
crate's tests and any other implementation consume.

## Forcing constraints / Evidence

It is practical here for reasons specific to the design. No wire types means
no encoding to specify; no clock means no temporal logic; a registry rather
than a state machine means the normative core is a data model and one total
function — the matrix in `../DESIGN.md` §4. And with zero injected
dependencies, an implementation runs the vectors with nothing to stub.

## Rejected alternatives

**The Rust crate as the definition**, its documentation and tests standing for
a specification. Rejected: tests in one language are not consumable by
another, and prose without vectors is not checkable.

**Vectors that state an implementation's representation** — the ranges this
crate keeps chunk ids in. Rejected: the specification fixes behaviour, not
representation, so a vector gives a session's chunks received as a set of
chunk ids.

## Consequences

Behaviour is a vector before it is code: a change to the matrix is a change to
the vectors first. The vectors' JSON is a public format and is versioned.
The cancel reason, the caller's enum, appears in the vectors as an enum of
their own.

The memory test is not a vector. It measures this implementation's
allocations, and no other implementation's.

## Enforcement

The crate's conformance tests read the committed vectors. Neither exists yet:
they arrive with the implementation, before its first function.

## Related

[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) — what makes the
suite stub-free. `faction`'s `P0-ADR-SpecificationIsNormativeCrateIsReference`
— the same decision for the sibling crate.
