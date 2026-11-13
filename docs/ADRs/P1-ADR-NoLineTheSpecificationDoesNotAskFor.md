# P1-ADR-NoLineTheSpecificationDoesNotAskFor

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P1 (derived)

## Context

Coverage is usually a target approached over time, because infrastructure —
sockets, files, clocks, processes — makes some lines hard to reach from a
test. Beggar has none: every line is reachable from `process`. A shortfall in
coverage here is therefore never a hard-to-test line; it is untested
behaviour, dead code, or behaviour the specification does not ask for.

## Decision

Beggar has 100% coverage from its first function — no uncovered line, region
or function, under all the test tiers together and under the conformance
vectors alone.

## Forcing constraints / Evidence

**Nothing is out of reach.** With nothing injected but an observer that
answers nothing ([P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md))
and one entry point, there is no line a test cannot drive.

**The vectors alone are the stronger measure.** The specification is normative
([P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md)):
a line the vectors never reach is behaviour the specification does not state —
unspecified behaviour, or dead code. Full coverage from the vectors alone means
every line exists because the specification asks for it.

## Rejected alternatives

**A percentage threshold.** Rejected: a rounded percentage hides the one line
that matters. The gate counts uncovered items, and allows none.

**Branch or MC/DC coverage.** Not rejected, unavailable: both are unstable,
nightly only, and the compiler is pinned to a stable release. Region coverage,
which counts every arm of a `match` and an `if`, is the finest stable measure.

**Excluding code from measurement.** Rejected: there is nothing to exclude, and
an exclusion is where an untested line would hide.

## Consequences

A type derives only the traits something uses, because derived code is
measured too, and a derive nobody calls is an uncovered function.

A new line arrives with the vector that reaches it. A line no vector can reach
is either removed, or the specification grows a vector — and a row, if the
line is behaviour the matrix does not yet state.

Coverage says every line ran, not that its effect is asserted. The house's
teeth checks — break the code, see a test fail — answer the second; mutation
testing (`cargo-mutants`) would automate them, once installing it is allowed.

## Enforcement

A coverage gate in stage 2, built with the first slice: `cargo llvm-cov` with
`--fail-uncovered-lines 0`, `--fail-uncovered-regions 0` and
`--fail-uncovered-functions 0`, run twice — over every tier, and over the
conformance tests alone. Since 2026-11-13 it measures the specification's model
too ([P1-ADR-IndependentModelOfTheSpecification](P1-ADR-IndependentModelOfTheSpecification.md)):
a line of it the vectors never reach is one the prose does not give it.

## Related

[P2-ADR-TestingLadder](P2-ADR-TestingLadder.md) — the tiers measured together.
