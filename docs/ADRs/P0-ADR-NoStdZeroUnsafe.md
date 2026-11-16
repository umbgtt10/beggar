# P0-ADR-NoStdZeroUnsafe

- **Status:** Accepted
- **Date:** 2026-11-16
- **Priority:** P0 (axiom)

## Context

Beggar runs beneath the recovery of every deployment its protocols have, from a microcontroller to processes in a
cluster: ideally the same source, the same crate.

## Decision

`#![no_std]` with `alloc`, and `#![forbid(unsafe_code)]`.

## Forcing constraints / Evidence

The target set makes `std` unavailable on one end. The security posture makes `unsafe` undesirable throughout: a
registry whose correctness rests on ownership can be reviewed by inspection.

## Rejected alternatives

**`std`.** Rejected: it excludes bare metal.

**`unsafe` for speed.** Rejected: accounting for chunks needs none, and one block forfeits the guarantee.

**`#![deny(unsafe_code)]`.** Rejected: `deny` can be lifted locally with an `#[allow]`; `forbid` cannot.

## Consequences

- State is `alloc`'s vectors, kept in canonical order: no hash map, whose order is not deterministic.
- No threads.
- No `std`-only types anywhere.
- A session's size follows the gaps in what has arrived, not its chunk count, and an optional bound caps the
  sessions open ([P1-ADR-AnOptionalBoundOnOpenSessions](P1-ADR-AnOptionalBoundOnOpenSessions.md)).

## Enforcement

- `#![no_std]` and `#![forbid(unsafe_code)]` at the crate root are enforced by the compiler.
- Stage 1 builds the crate and the specification's model for a target with no `std`.

## Related

faction's and cartel's ADRs of the same name.
