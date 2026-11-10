# beggar

## Meaning

`beggar` is a `no_std`, protocol-independent recovery registry: one of the four
bookkeepers, mutually unaware, that IBFT and Raft orchestrate.

A recovery is just a collection of chunks -- logs or snapshots, for IBFT or for
Raft, pushed or pulled. Beggar does not know what is transferred, how, in what
order, under what policy or which protocol. No bits or bytes in beggar.

`docs/ADRs/` holds the foundations and `docs/DESIGN.md` the how; both are
authoritative. Read them before changing anything.

## Quality Gates

### Mandatory after every change

Run:

`just stage1`
`just stage2`

If either gate is not green, the work is not complete.

Stage 1 is formatting, clippy over every target, the `no_std` check and tests.
The `no_std` check builds for `thumbv7em-none-eabihf`, a target with no std at
all, which `rust-toolchain.toml` installs.

Stage 2 is `cargo xtask stage2`, ported from etheram-core's: stern4rust, then
dry4rust against `dry4rust-baseline.json`, crap4rust, the coverage gate twice,
twin4rust, iceberg4rust and cargo-deny, in that order, for the reasons
etheram-core records. stern runs over `beggar`, `beggar-core-validation`,
`beggar-ladder`, `beggar-memory` **and** `xtask`; the rest over `beggar`.

The tests climb a ladder of seven tiers — `docs/ADRs/P2-ADR-TestingLadder.md`.
The coverage gate, `cargo llvm-cov` over `beggar` and `beggar-ladder`, fails on
any line, region or function of beggar left uncovered: once over every tier, and
once over the conformance tier alone, its tests filtered to `conformance::` —
the vectors must reach every line by themselves, or the crate holds something
the specification does not ask for
(`docs/ADRs/P1-ADR-NoLineTheSpecificationDoesNotAskFor.md`). Derived impls are
not measured, and a hand-written one is a line like any other: `ChunkSet`
derives `Default`, which clippy asks for beside its `new`, rather than writing
it. The specification in `spec/` is normative; the crate is one implementation
of it.

## Layout

A workspace: `core/` holds the crate, `ladder/` the testing ladder's scenario
tiers, `core-validation/` its cluster simulation, `memory/` its memory tier, and
`xtask/` runs the gates. A crate at the repository root would put
`xtask/tests/**` inside its package directory, and stern would read them as the
crate's.

`core/tests/` is the unit tier, one test file per source file, under every rule.
The tiers that are scenarios rather than mirrors of a file -- conformance, the
exhaustive matrix, property, replay -- live in `ladder/`, a package with no code
of its own (`publish = false`), where stern's `paired-test-file` stands down:
it pairs `tests/a/b_tests.rs` with `src/a/b.rs` by path, and those tiers are
named for what they prove. The cluster simulation is `core-validation/`, laid
out as faction's: its harness -- `ScenarioHarness`, `ClusterSimulation` over a
seeded `FaultyTransport`, and `ClusterInvariants`, each check shown firing on
a crafted violation -- is its `src/`, `no_std`, and its tests, named for what
they prove, skip `paired-test-file` too. The memory tier is
`memory/`, a package of its own because a `#[global_allocator]` is per binary
and a tests folder has one door: the counting allocator and the run it measures
are its `src/`, mirrored by its tests under every rule -- and the workspace's
one `unsafe`, the allocator's, since beggar forbids it. The specification,
`spec/`, sits at the repository root, read by the ladder and never by the
crate.

## Strict Rules

### Test-driven

Tests are written first and seen failing before any implementation.

### Test preservation

**NEVER remove a test without explicitly asking the user first.**

### Assertion integrity

**NEVER relax a test assertion without explicitly asking the user first.**

### No re-export shims

Every `use` points directly to the module where the symbol is defined.

## User coding standards

- one struct per file
- do not use fully qualified paths; use `use` imports instead
- no unnecessary comments in code
- no `&mut` input parameters; prefer return values (`&mut self` is allowed in traits and impl blocks)
- only use `pub mod` in `mod.rs` and `lib.rs`
- one test file per source file, named `<source file name>_tests.rs`
- in `all_tests.rs`, reference test files one by one without `#[path = ...]`
- AAA (`Arrange`, `Act`, `Assert`) structure in tests; `// Arrange & Act` or `// Act & Assert` where two merge
- the header in `docs/header.txt` on every Rust source file
- **test naming**: `<method>_<description>_<outcome>` where `<method>` is the function called in the Act section
