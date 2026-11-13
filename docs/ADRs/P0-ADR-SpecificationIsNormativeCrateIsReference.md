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

Beggar is specified as a standard component, in a versioned `spec/` written
first and by hand — a JSON Schema data model, a JSON decision table, normative
prose and JSON conformance vectors — and this crate is one implementation of
it: where the two disagree, the specification is right and the crate has a
bug.

## Forcing constraints / Evidence

It is practical here for reasons specific to the design. No wire types means
no encoding to specify; no clock means no temporal logic; a registry rather
than a state machine means the normative core is a data model and one total
function over a decision table. And with nothing injected that answers — its
one dependency, an observer, is told and answers nothing — an implementation
runs the vectors with nothing to stub.

Nothing of beggar is built yet, so the specification can come first. A crate
that already existed would have to export its vectors from its own tests and
be checked against itself.

## Rejected alternatives

**The Rust crate as the definition**, its documentation and tests standing for
a specification. Rejected: tests in one language are not consumable by
another, and prose without vectors is not checkable.

**Vectors exported from the crate's tests.** Rejected: the crate becomes the
source the specification is drawn from, and a refactor redefines what
conforming means.

**`serde` derives on the crate's types, their JSON standing as the model.**
Rejected: it ties the format to one implementation's names and nesting, and
gives the crate a dependency.

**Vectors that state an implementation's representation** — the ranges this
crate keeps chunk ids in. Rejected: the specification fixes behaviour, not
representation, so a vector gives a session's chunks received as a set of
chunk ids.

**u64 as JSON numbers.** Rejected: many JSON parsers read numbers as 64-bit
floats, exact only to 2⁵³, and an incarnation drawn from a nanosecond
timestamp is already past it.

## Consequences

The specification has four parts, versioned together:

| part | form | says |
|---|---|---|
| the data model | JSON Schema | every command, receipt, output, rejection and the checkpoint: fields and types |
| the matrix | `matrix.json`, a decision table | the rows in check order, each a named situation and its output, each with an id |
| the semantics | `SPEC.md`, normative prose | what JSON cannot say: each situation's meaning, totality, determinism, what a maximal run is |
| conformance | `vectors/*.json` | `(initial state, commands) → (receipts, final state)`, each vector naming the rows it exercises |

Every u64 is written as a string of decimal digits. An initial state is empty
or a checkpoint; a final state is the checkpoint the vector expects.

Behaviour is in the specification before it is code: a change to the matrix is
a change to the specification first. The receipt and the checkpoint are part of
it, so any implementation's log can be replayed by any tool.

The crate never reads the specification. Its conformance tests map the JSON to
its types through a model of their own. The specification's model, in
`crates/spec-model/`, is a second implementation, in Rust and independent of the
crate, and the vectors are checked against it before the crate runs them
([P1-ADR-IndependentModelOfTheSpecification](P1-ADR-IndependentModelOfTheSpecification.md));
one in another language passing the same vectors would prove the specification
language-independent, and may come later.

The cancel reason, the caller's enum, appears in the vectors as an enum of
their own. The memory test is not a vector: it measures this implementation's
allocations, and no other implementation's.

## Enforcement

The conformance tests run every vector; every vector validates against the
schema; every row of `matrix.json` is exercised by at least one vector. They
arrive with the first slice, before its first function.

## Related

[P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) — what makes the
suite stub-free.
[P1-ADR-NoLineTheSpecificationDoesNotAskFor](P1-ADR-NoLineTheSpecificationDoesNotAskFor.md)
— coverage measured against the vectors alone. `faction`'s ADR of the same
name — the same decision for the sibling crate, drawn from its existing code.
