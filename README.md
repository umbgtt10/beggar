# beggar

**A `no_std`, zero-`unsafe`, protocol-independent recovery registry.**

Every replicated system has nodes that fall behind: a follower missing log entries, a validator missing blocks, a
node restarted from an old snapshot. Every protocol writes its own recovery -- which entries to ask for, from whom,
which have arrived, what is still missing, when to give up -- tangled with the protocol's rules and its wire types,
so the accounting is tested only through the protocol, and every protocol writes it again. Its bugs -- a chunk
counted twice, a transfer that never ends, a session the two ends disagree about -- surface when a node is already
behind, which is exactly when it matters.

Recovery is a supplicant. A node that fell behind has no leverage over the peers that did not: it asks, the peer
**may refuse**, it may be handed something other than what it asked for, and if ignored its only recourse is to ask
again. That asymmetry is the whole mechanism, and it is the reason for the name.

`beggar` takes the accounting out: a **formally specified, fully tested registry** of transfer sessions that
records what moved and decides nothing.

**All five slices are built** (2026-11-10): an instance, created fresh or from a checkpoint; both ends of a
transfer; cancellation for the caller's own reason; the queries; and retention, conformant to version 0.7.0 of the
specification in [`spec/`](spec/SPEC.md). Version 0.6.0 is on [crates.io](https://crates.io/crates/beggar).

---

## What beggar does

`beggar` keeps the books on transfers:

- which sessions are open, between which two nodes, and how many chunks each holds;
- which runs of chunks have moved, at each end;
- what each session still lacks, as maximal runs;
- which sessions finished, and how.

**A recovery is just a collection of chunks**: a request for blocks, a lagging follower's missing log entries, a
snapshot sent in pieces; under any consensus protocol; pulled by the node behind or pushed by the node ahead. It
deals in **no bytes, blocks, arrays or wire types** -- session identities, chunk counts, chunk ids and node
identities, and nothing else. The payload moves elsewhere; `beggar` records that it moved.

It does exactly this. Nothing more.

No detection that a node is behind. No choice of peer, no retry timer, no fallback to a snapshot, no clock, no
heights. The caller owns all of that. `beggar` owns the books.

---

## Why beggar?

- **Deterministic** — every output is a function of the command and the session record it meets. The same calls
  always give the same receipts.
- **Specified** — a normative, language-independent specification: prose, a JSON Schema, a decision table and 79
  conformance vectors. The crate is one implementation; an independent model of the specification is another.
- **Verifiable** — every command in every situation is checked against the model over small chunk counts, and the
  invariants after every call over random sequences. No untested paths exist.
- **Embeddable** — `no_std + alloc`, zero `unsafe`. Given a bound on open sessions, an open past it is refused, so
  its memory has a ceiling on a microcontroller as on a server.
- **Observable** — every call reaches a trait-based `Observer`, once, with its receipt: an accepted command, a
  refused one, a query.
- **Replayable** — every receipt is numbered by `beggar`, one more than the last. A log rebuilds the registry
  receipt by receipt, and a gap in the numbers proves a line lost.
- **Protocol-independent** — there is no vocabulary in which a protocol's type could be expressed: session ids,
  chunk counts and node ids mean the same thing everywhere. Nothing is injected that answers, so a conformance suite
  runs against any implementation in any language without stubbing anything.

---

## How it works

### A registry, not a state machine

`beggar` has no states: every outcome is a function of the command and the session record, so there is no label
doing any work. That decides how it is tested: invariants and a decision table, not transition coverage.

### Commands and outputs

Nine commands, one entry point, `process(command) -> receipt`:

| Command | Answered with | Meaning |
|---|---|---|
| `StartRecovery { chunks, sender, receiver }` | `RecoveryStarted { session_id }` | Open a session on the end that holds the chunks, and name it |
| `AdoptRecovery { session_id, chunks, sender, receiver }` | `RecoveryStarted`, `RecoveryUnaffected` or `RecoveryConflicted` | Open the same session on the other end, under the name its starter minted |
| `SendRecoveryPartial { session_id, first, last, sender, receiver }` | `RecoveryAdvanced { held, chunks }`, `RecoveryCompleted` or `RecoveryUnaffected` | Record a run of chunks sent |
| `ReceiveRecoveryPartial { session_id, first, last, sender, receiver }` | `RecoveryAdvanced { held, chunks }`, `RecoveryCompleted` or `RecoveryUnaffected` | Record a run of chunks received |
| `CancelRecovery { session_id, reason }` | `RecoveryCancelled(reason)` or `RecoveryUnaffected` | End a session for the caller's own reason |
| `ListRecoveries` | `RecoveriesListed { sessions }` | The open sessions |
| `ShowRecovery { session_id }` | `RecoveryShown { record, finished }` | One session's record, open or retained |
| `ListMissingChunks { session_id }` | `MissingChunksListed { missing }` | The chunks a session still lacks, as maximal runs |
| `ShowRegistry` | `RegistryShown` | The checkpoint: the whole registry |

A command naming a session that is not open is answered `RecoveryNotStarted`, and a malformed or misdirected one is
refused, `RecoveryRejected(rejection)`, with one of nine rejections saying why and carrying what the log needs
beside the command. A refusal changes nothing. `beggar` never panics and never returns an opaque error.

### Both ends of a transfer

The side that holds the chunks opens the session and states their count, since only it knows it. `beggar` mints the
session's name -- the instance id and a count -- and the peer's registry adopts it. Each end then records the runs
of chunks it sends or receives; a single chunk is a run of one. When the set is full the session completes; when the
caller cancels, it ends, carrying the caller's reason, which `beggar` records and never reads.

### Serving always works

The node that answers a request is precisely the one already in sync, so anything that accepted serve requests only
while behind would go deaf exactly when peers need it. In a registry this is automatic rather than a rule to
remember.

### Retention and the bound

**Retention** keeps finished sessions for the queries: as many as the instance's retention, the latest to finish,
each as it stood. It changes what a query answers and nothing else, so a host kept for analysis and a board keeping
none answer every other command alike. An optional **bound** on open sessions refuses an open past it.

| Setting | Default | Deployed |
|---|---|---|
| retention | none — every instance states its own | 0 on the IBFT cluster; each other integration's chosen as it is built; high on a host kept for analysis |
| bound on open sessions | none — no bound | the integration's to choose |

### Crash and restart

A node that restarts is a new instance: created fresh, under a new incarnation, so the names it mints never collide
with those it minted before. An instance can also be restored from a checkpoint -- `ShowRegistry`'s answer -- in
exactly the state it describes: that is how a replay resynchronises and how a vector starts from the state it
needs, never a live restart, since a checkpoint's name count may be behind the one the instance had reached.

---

## Quick start

```toml
[dependencies]
beggar = "0.6"
```

```rust
use beggar::beggar::Beggar;
use beggar::command::Command;
use beggar::fresh::Fresh;
use beggar::instance_id::InstanceId;
use beggar::no_op_observer::NoOpObserver;
use beggar::origin::Origin;
use beggar::output::Output;

// The caller's own reasons for giving a transfer up: beggar records them and
// never reads them.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Reason {
    PeerFailed,
}

fn fresh(node: u64) -> Beggar<Reason> {
    Beggar::new(
        Origin::Fresh(Fresh { instance: InstanceId { node, incarnation: 1 }, retention: 0, bound: None }),
        Box::new(NoOpObserver),
    )
}

// Node 1 holds ten chunks node 2 lacks: it starts the session, and beggar names it.
let mut sender = fresh(1);
let Output::RecoveryStarted { session_id } = sender
    .process(Command::StartRecovery { chunks: 10, sender: 1, receiver: 2 })
    .output
else {
    unreachable!("a fresh registry starts a well-formed session")
};

// Node 2 adopts the session under that name, with the count node 1 stated.
let mut receiver = fresh(2);
let _ = receiver.process(Command::AdoptRecovery { session_id, chunks: 10, sender: 1, receiver: 2 });

// Chunks 1 to 6 move; each end records the run on its own side.
let _ = sender.process(Command::SendRecoveryPartial { session_id, first: 1, last: 6, sender: 1, receiver: 2 });
let receipt =
    receiver.process(Command::ReceiveRecoveryPartial { session_id, first: 1, last: 6, sender: 1, receiver: 2 });
assert_eq!(receipt.output, Output::RecoveryAdvanced { held: 6, chunks: 10 });

// What node 2 still lacks, as maximal runs.
let missing = receiver.process(Command::ListMissingChunks { session_id }).output;
assert_eq!(missing, Output::MissingChunksListed { missing: vec![7..=10] });

// The rest arrives, and the session completes.
let receipt =
    receiver.process(Command::ReceiveRecoveryPartial { session_id, first: 7, last: 10, sender: 1, receiver: 2 });
assert_eq!(receipt.output, Output::RecoveryCompleted);

// Node 1 gives up on its end, for a reason of its own.
let receipt = sender.process(Command::CancelRecovery { session_id, reason: Reason::PeerFailed });
assert_eq!(receipt.output, Output::RecoveryCancelled(Reason::PeerFailed));
```

The caller moves the chunks. `beggar` owns the books.

---

## Validation

| Package | What it tests | Tests |
|---|---|---|
| [`spec/`](./spec/SPEC.md) | The normative specification: a decision table of 25 rows and 79 conformance vectors, each receipt written by hand | — |
| `crates/spec-model/` | The specification's independent model: its decision table read and interpreted, depending on nothing of the crate | 146 |
| `crates/core/` unit tests | Each type and each row in isolation, and the front page's and the README's examples | 98 |
| `validation/ladder/` | Conformance to the specification, of the model and then of the crate; every command in every situation over small chunk counts, against the model; properties over random command sequences, the crate against the model among them; logs replayed, resynchronising from a checkpoint; no word the vocabulary leaves out | 41 |
| `validation/core-validation/` | A seeded cluster simulation: registries whose reports to one another are lost, duplicated, reordered and withheld -- pull, push, cancels, restarts under a new incarnation | 72 |
| `validation/memory/` | A counting allocator: a positive control, then peak and residue over a hundred recoveries | 15 |

---

## Project status

| Metric | Value |
|---|---|
| Core productive LOC | ~670 |
| Specification | [`spec/`](./spec/SPEC.md) 0.7.0 — its independent model and the crate answer all 79 vectors as written, each kept as released |
| Total tests | 493 |
| Code coverage (productive) | 100%, over every tier and over the conformance tier alone |
| Crappy functions (CRAP score) | 0 |
| Unsafe code | 0 — enforced by `#![forbid(unsafe_code)]` |
| `no_std` | Verified on `thumbv7em-none-eabihf` |
| Published | [crates.io](https://crates.io/crates/beggar) 0.6.0 |

---

## Design principles

- **Accounts, never decides** — no detection, no peer choice, no timing, no policy.
- **A recovery is a collection of chunks** — logs or snapshots, pushed or pulled, under any protocol.
- **A registry, not a state machine** — every output is a function of the command and the session's record.
- **The specification is normative** — the crate is a reference implementation; where they disagree, the
  specification is right.
- **One entry point, one output** — every call is a command, every command answered by one receipt.
- **Observer, not logger** — the `Observer` receives every call with its receipt. Wire it to anything.
- **The log is the record** — numbered receipts rebuild the registry; the checkpoint is a query.
- **The sender opens and states the count**, and **the accountant names the session** — the peer adopts the name.
- **Retention governs queries, never emission** — what is kept changes what a query answers, and nothing else.
- **One struct per file**, and **no `&mut` parameters** — prefer return values over in-place mutation.

Each principle is recorded as an Architecture Decision Record — eighteen, in [docs/ADRs/](./docs/ADRs/README.md).

---

## Non-goals

`beggar` deliberately does not:

- Detect that a node is behind, or choose a peer to ask
- Time a retry, or own a clock
- Fall back to a snapshot, or choose between log and snapshot
- See a payload — the caller validates a chunk, and reports it only then
- Know heights, terms or any protocol's types
- Store anything durably — the checkpoint is the caller's to keep

---

## Quality gates

```sh
just stage1   # format, clippy, the no_std check and the tests
just stage2   # house rules, duplication, CRAP, coverage, mirrored tests, file risk, dependencies
```

Both gates must pass before any commit lands. Stage 2's gates are
[`serge4rust`](https://github.com/umbgtt10/serge4rust)'s, the house's quality gates as a library; `xtask/src/main.rs`
holds only which of them run, in what order, and with what settings. Until serge4rust 0.2.0 is on crates.io the
xtask takes it by path, from a `serge4rust` checkout beside this one. They drive these tools, each installed once
from crates.io:

| Tool | Used by | Install |
|---|---|---|
| [`just`](https://crates.io/crates/just) | Runs the two stages | `cargo install just` |
| [`cargo-stern4rust`](https://crates.io/crates/cargo-stern4rust) | Stage 2 checks the house coding rules in every package | `cargo install cargo-stern4rust` |
| [`cargo-dry4rust`](https://crates.io/crates/cargo-dry4rust) | Stage 2 refuses duplicated code in `crates/` | `cargo install cargo-dry4rust` |
| [`cargo-crap4rust`](https://crates.io/crates/cargo-crap4rust) | Stage 2 scores CRAP (Change Risk Anti-Patterns) complexity over `crates/` | `cargo install cargo-crap4rust` |
| [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) | Stage 2 requires every line, region and function of `crates/` covered, by every tier and by conformance alone | `cargo install cargo-llvm-cov` |
| [`cargo-twin4rust`](https://crates.io/crates/cargo-twin4rust) | Stage 2 requires a mirrored test file for every source file | `cargo install cargo-twin4rust` |
| [`cargo-iceberg4rust`](https://crates.io/crates/cargo-iceberg4rust) | Stage 2 bounds each file's private implementation risk | `cargo install cargo-iceberg4rust` |
| [`cargo-deny`](https://crates.io/crates/cargo-deny) | Stage 2 checks advisories, licences and sources | `cargo install --locked cargo-deny` |
| [`cargo-ripe4rust`](https://crates.io/crates/cargo-ripe4rust) | `just ripe` plans the upgrades the workspace could take, judged against the two stages; it applies nothing | `cargo install cargo-ripe4rust` |

---

## Roadmap

`beggar` was built in five additive slices -- open a recovery and see it, send it to completion, the receiving end,
abandon it, look inside and keep what finished -- each a release, each answering every vector of the ones before
it unchanged. All five are built. A first protocol's integration is built, its recovery split into a control layer
with `beggar` beneath it; IBFT's follows, beneath the control layer it already has.

See [IMPLEMENTATION-PLAN.md](./docs/IMPLEMENTATION-PLAN.md) for the plan and [DESIGN.md](./docs/DESIGN.md) for the
design.

---

## Workspace

The workspace is laid out by role: [`spec/`](./spec/SPEC.md) holds the normative specification, `crates/` the
library, `validation/` everything that checks it, and `xtask/` the gates.

| Package | Role |
|---|---|
| `crates/core/` | The registry: `beggar`, the crate on crates.io |
| `crates/spec-model/` | The specification's independent model, `no_std` |
| `validation/ladder/` | Conformance, the exhaustive matrix, property and replay tiers |
| `validation/core-validation/` | The seeded cluster simulation |
| `validation/memory/` | The memory tier, a counting allocator in a binary of its own |
| `xtask/` | The gates, taken from [`serge4rust`](https://github.com/umbgtt10/serge4rust): `cargo xtask stage2` and the commit scores |

---

## Siblings

`beggar` is one of four bookkeepers a consensus protocol orchestrates, none aware of the others:

| Bookkeeper | Keeps |
|---|---|
| [`faction`](https://crates.io/crates/faction) | Membership: whether the cluster is ready, who joins and leaves, who still answers |
| [`beggar`](https://crates.io/crates/beggar) | Recovery: which chunks of a transfer have moved, at both ends |
| `notary` | Credentials — not yet built |
| [`cartel`](https://crates.io/crates/cartel) | Consensus: who voted for what, and when a threshold is reached |

All are `no_std`, protocol-independent and specified, each with one entry point and an observer; the protocol
translates for each at one integration point. The difference between `faction` and `beggar` is load-bearing:
membership has genuine states a node moves between, so `faction` is a state machine and needs transition coverage;
recovery has none, so `beggar` is a registry and needs invariants.

---

## License

MIT. See [LICENSE](./LICENSE).

---

## Links

- [SPEC.md](./spec/SPEC.md) — the normative specification: data model, decision table, prose and conformance vectors
- [DESIGN.md](./docs/DESIGN.md) — the design: the stance, the vocabulary, the matrix, where `beggar` sits
- [ADRs](./docs/ADRs/README.md) — architecture decision records: the design rationale, one decision per file
- [IMPLEMENTATION-PLAN.md](./docs/IMPLEMENTATION-PLAN.md) — the slices, and the integrations after them
- [CHANGELOG.md](./CHANGELOG.md) — version history
