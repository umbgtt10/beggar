# P2-ADR-WorkspaceLaidOutByRole

- **Status:** Accepted
- **Date:** 2026-11-13
- **Priority:** P2 (structural)
- **Phase:** cross-cutting

## Context
The workspace holds the specification, the library, three packages that check
the library -- the testing ladder's scenario tiers, its cluster simulation and
its memory tier -- and the crate that runs the gates. Each role is held to
different gates: the library to every gate at full strength, the packages that
check it to the house rules and the lints. faction, beggar's sibling, lays its
workspace out by role (its own `P2-ADR-WorkspaceLaidOutByRole`); the tree is
where a reader meets the roles first.

## Decision
The workspace is laid out by role: `spec/` holds the specification, `crates/`
the library, `validation/` every package that checks the library, and `xtask/`
the gates.

| directory | packages |
|---|---|
| `crates/core/` | `beggar`, published |
| `validation/ladder/` | `beggar-ladder` |
| `validation/core-validation/` | `beggar-core-validation` |
| `validation/memory/` | `beggar-memory` |
| `xtask/` | `xtask` |

## Forcing constraints / Evidence
The gates name the library already: CRAP, the coverage gate, the mirrored tests
and the file risk measure `beggar`, and the duplication gate scans its source
alone. The layout states what the gates enforce. `spec/` is normative data,
versioned apart from the crate, read and never built; a package inside it would
make it something else.

## Rejected alternatives
**Every package at the root**, beside the specification, the documents and the
gates' configuration: the roles were stated only in `CLAUDE.md`'s layout.

**Directories named for their packages** (`crates/beggar/`,
`validation/beggar-ladder/`): every path renamed for the one directory, `core/`,
whose name differs from its package's -- and the sibling names its own `core/`.

## Consequences
A crate the library gains goes under `crates/` and joins the library's lists in
`xtask/` -- the gates name packages, and the layout selects nothing on its own.
A package that checks the library goes under `validation/`. Packages reach the
specification and the README two directories up. No package is renamed, and
nothing the crate does changes.

## Enforcement
The workspace manifest's members. The duplication gate scans
`crates/core/src`, `DRY_PATH` in `xtask/src/main.rs`.
