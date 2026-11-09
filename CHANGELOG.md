# Changelog

The crate's versions. The specification in [`spec/`](spec/SPEC.md) is versioned on its own; each release names the
version it conforms to.

## [0.1.0] — 2026-11-09

### Slice 1 — open a recovery and see it

The first version with an implementation, conformant to the specification's version 0.2.0.

#### Added
- `Beggar<R>`, the registry, and its one entry point, `process(command)`. Every call is answered with a `Receipt`:
  its output, and a number the instance counts itself — 1 for the first call, one more for every call after it, a
  refused command and a query included.
- An instance is created from an `Origin`: `Fresh` — the instance id, a node and an incarnation; a retention; and
  an optional bound on open sessions — or a `Checkpoint`, restored as given.
- `StartRecovery`, with every row of the decision table: `NoChunks`, `SelfAddressed`, `NotAParty`,
  `RegistryFull { capacity }`, and `RecoveryStarted`, carrying the new session's name — the instance id and a
  count, so no two sessions anywhere share one.
- `ShowRegistry`, answering the checkpoint: the whole registry, as it stands after the call.
- `Observer`, injected through the constructor and told of every call — its command and its receipt — exactly
  once; and `NoOpObserver`, for a caller with no use for one, as `faction` provides its own.
- The specification, in `spec/`: the prose, a JSON Schema, the decision table and 16 conformance vectors. Its
  0.1.0 specified slice 1; its 0.2.0, the same day, adds the observer and changes nothing else.
- The testing ladder's first four tiers: unit, in `core/tests/`; and, in `ladder/`, a package of their own,
  conformance, the exhaustive matrix — 333 situations, each from a fresh instance and from a checkpoint, against a
  naive model — and property, random sequences against the same model.
- A coverage gate in stage 2: `cargo llvm-cov`, failing on any uncovered line, region or function of the crate,
  over every tier and over the conformance tier alone.
- The design, in `docs/`: four P0 ADRs, ten P1s and the testing ladder's P2, and `DESIGN.md`; and the house's
  quality gates, stage 1 and stage 2, in `xtask/`.

#### Changed
- A registry, not a state machine: a recovery is a collection of chunks, and beggar accounts for which have moved.
  The conception 0.0.1 described — asking a peer, retrying, falling back to a snapshot — belongs to the control
  layer above it.
- `P0-ADR-AccountsNeverDecides`, amended by `P1-ADR-EveryCallIsObserved`: beggar has one injected dependency, the
  observer, which answers nothing.
- `rust-version` 1.85, inherited from `faction`, is measured: the crate builds, and its unit tier passes, on
  1.85.0.

## [0.0.1] — 2026-09-27

Relicensed from Apache-2.0 to MIT, and `faction`'s conventions adopted. Reserves the name; no implementation.

## [0.0.0] — 2026-09-26

Reserves the name; no implementation.
