# P1-ADR-IndependentModelOfTheSpecification

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P1 (derived from `P0-ADR-SpecificationIsNormativeCrateIsReference`)
- **Phase:** cross-cutting

## Context
Every vector's expected receipts are written by hand, and a hand-written receipt
can be wrong. The crate as the judge of its own vectors is the alternative
[`P0-ADR-SpecificationIsNormativeCrateIsReference`](P0-ADR-SpecificationIsNormativeCrateIsReference.md)
rejects. Until now the second implementation was the ladder's naive registry:
deliberately naive and shaped otherwise than the crate, but built on the crate's
own types -- its commands, outputs, records and checkpoint -- and walking a run
chunk by chunk, so for small chunk counts only. It judged the crate in the
exhaustive matrix and property tiers, and the vectors never.

## Decision
The specification has a second implementation in the repository: the package
`beggar-spec-model` in `crates/spec-model/`, written from the prose and the
decision table and independent of the crate, `no_std` and free of unsafe code,
gated like the library -- as faction's `faction-spec-model` is. The vectors are
checked against it before the crate runs them, and the crate is checked against
it over command sequences no vector holds.

## Forcing constraints / Evidence
**Independent of the crate, or worthless.** A model that shares the crate's types
or its shape shares its mistakes. So the model depends on nothing of `beggar` —
`cargo deny` refuses `beggar` to every crate but those written for it, and was
seen to refuse it to the model, as a dependency and as a dev-dependency — and
takes and answers commands in the specification's own encoding. The two meet
only in the ladder, where the crate's commands and receipts are written in that
encoding.

**The specification's shape, not the crate's.** A command is answered by the
first row of the decision table that applies, in the table's order (§3). The
model reads the rows from `spec/matrix.json` -- their ids, commands, outputs and
rejections -- and owns only what the prose gives each: the situation in which it
applies (eighteen of them, from "always" to "the run holds a chunk not yet in
the chunk set, and the chunk set is then full"), read from the command, from
the sessions the registry holds or from the open session the command names; the
change it makes (six); what its output carries; and the checkpoint (§10). The
crate is a registry whose commands are checked by rows of its own; the model is
a table read and interpreted. Records hold the data — the command, the
instance's state, a session, a row, a rule — and objects the behaviour: the
readers of the encoding, the table, the rule book, the lookup, the view. A chunk
set is kept as its maximal runs (§1), so a vector at the top of u64 is answered
as the rest are, without walking a chunk.

**What it found.** Every vector — 79 of them, 253 steps, 8 from a checkpoint —
answered as written, each receipt and the final state; every step but a query's
answered alike with the retention raised to the top of u64; each vector deciding
by exactly the rows it names; every row of the matrix decided by some vector;
and the model's rule book holding exactly the matrix's rows. The vectors and the
model agreed on the first run: no vector needed correcting. Then, over random
sequences of up to 60 commands on random instances — node 1 to 3, any
incarnation, a retention up to 8, no bound or one up to 4 — the crate answering
every command as the model does, every receipt compared whole in the
specification's encoding.

## Rejected alternatives
**The naive registry as the judge.** Rejected: built on the crate's types, a
mistake in a shared type is in both, and the two agree on it; and walking a run
chunk by chunk, it cannot answer a vector at the top of u64, so it could never
judge the vectors.

**The crate as the judge of its own vectors.** Rejected by
`P0-ADR-SpecificationIsNormativeCrateIsReference`: a vector the crate answers is
checked against nothing but the crate.

**The decision table typed into the model.** Rejected: the table is normative
data. Read, it is the table the specification states; the model's own knowledge
is what the table cannot say, and its rule book is checked to know exactly the
table's rows.

## Consequences
The model is independent of the crate's code and types, not of its author: a
misreading of the prose by one hand can be made twice. The vectors are the
check on both.

A change to the specification changes the model with it. A row the matrix gains
needs its situation and its change in the rule book before the conformance tier
passes; a command, an output or a rejection the vocabulary gains is spelled
there; and the new vectors run against the model before the crate runs them.

The model is one instance and has no observer: what the observer is told is the
crate's to show, in its own conformance tier. It reads the matrix, so it cannot
disagree with it about a row's commands, output or rejection — only about the
situations, the changes and what an output carries; the matrix itself is checked
against the vectors.

The property tier compares the crate with this model. The exhaustive matrix
tier still compares it with the naive registry, until that tier moves onto the
model and the naive registry is retired.

## Enforcement
`cargo deny`'s bans in `deny.toml`. The model's conformance tier,
`validation/ladder/tests/conformance/spec_model_tests.rs`, and the crate against
the model, `validation/ladder/tests/property/beggar_proptest_tests.rs`. Stage 2
holds the model to every gate the library is held to — stern4rust, dry4rust,
crap4rust, twin4rust, iceberg4rust, and coverage over every tier and over the
conformance tier alone (`P1-ADR-NoLineTheSpecificationDoesNotAskFor`); stage 1
builds it for a target with no `std`; `#![forbid(unsafe_code)]`; and every
commit message carries its grip and braintax.

## Related
faction's `P1-ADR-IndependentModelOfTheSpecification` — the model this one
follows.
