# beggar

## Meaning

`beggar` is a `no_std`, protocol-independent recovery registry: one of the four
bookkeepers, mutually unaware, that IBFT and Raft orchestrate.

A recovery is just a collection of chunks -- logs or snapshots, for IBFT or for
Raft, pushed or pulled. Beggar does not know what is transferred, how, in what
order, under what policy or which protocol. No bits or bytes in beggar.

`docs/DESIGN.md` is authoritative; read it before changing anything.

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
dry4rust against `dry4rust-baseline.json`, crap4rust, twin4rust, iceberg4rust
and cargo-deny, in that order, for the reasons etheram-core records. stern runs
over `beggar` **and** `xtask`; the rest over `beggar`.

crap4rust needs a function to score: on a crate with none, llvm-cov finds no
coverage data and the gate fails. Until the implementation begins,
`core/src/dummy.rs` and its test give it one; they go when the first real
function arrives.

## Layout

A workspace: `core/` holds the crate and `xtask/` runs the gates. A crate at the
repository root would put `xtask/tests/**` inside its package directory, and
stern would read them as the crate's.

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
