# beggar

A `no_std`, protocol-independent **recovery registry**.

**All five slices are built** (2026-11-10): an instance, created fresh or from a checkpoint, with an observer
told of every call; both ends of a transfer — `StartRecovery` and `SendRecoveryPartial` on the side that holds the
chunks, `AdoptRecovery` and `ReceiveRecoveryPartial` on the other — and `CancelRecovery`, carrying the caller's own
reason; the queries — `ListRecoveries`, `ShowRecovery`, `ListMissingChunks`, and `ShowRegistry`, the checkpoint;
and retention — each with every row it has, conformant to the specification's version 0.6.0, in
[`spec/`](spec/SPEC.md). Version 0.5.0 is tagged in this repository; version 0.0.1 on crates.io only reserves the
name.

The foundations are four ADRs in [`docs/ADRs/`](docs/ADRs/README.md); the design is
[`docs/DESIGN.md`](docs/DESIGN.md). Both are authoritative; this file is the summary. The order of building it is
[`docs/IMPLEMENTATION-PLAN.md`](docs/IMPLEMENTATION-PLAN.md).

## The idea

Recovery is a supplicant. A node that fell behind — for whatever reason — has no leverage
over the peers that did not. It asks for the entries it missed, the peer **may refuse**, it may be handed
something other than what it asked for, and if ignored its only recourse is to ask again.

That asymmetry is the whole mechanism, and it is the reason for the name.

## What it actually does

**Pure accounting.** Beggar is a registry of ongoing transfer sessions. A session is opened with an expected
chunk count and two node identities, and beggar names it; the peer's registry adopts the name. Each run of chunks
that moves — a single chunk is a run of one — is recorded against it; when the set is full, or the caller cancels, the session terminates and is evicted.
An instance may be given a bound on how many sessions are open at once; past it, an open is refused.
A command it refuses gets an output saying why — never an exception. A caller can ask which sessions are open, one
session's record, and the chunks it still lacks, as maximal runs.

**Retention** keeps finished sessions for those queries: as many as the instance's retention, the latest to
finish, each as it stood. It changes what a query answers and nothing else, so a host kept for analysis and a
board keeping none answer every other command alike. Both settings are recorded in two columns, because a library's
default quoted as a deployed value is a trap this project has already fallen into:

| setting | default | deployed |
|---|---|---|
| retention | none — every instance states its own | 0 on the IBFT cluster; Raft's chosen at its integration; high on a host kept for analysis |
| bound on open sessions | none — no bound | the integration's to choose |

It deals in **no bytes, blocks, arrays or wire types** — session identities, chunk counts, chunk ids and node
identities, and nothing else. The payload moves elsewhere; beggar records that it moved.

**A recovery is just a collection of chunks**: a request for blocks, a lagging follower's missing log entries, a
snapshot sent in pieces; in IBFT or in Raft; pulled by the node behind or pushed by the node ahead. Beggar knows
none of it. What a chunk holds, in what order chunks move and under what policy are the caller's.

**It is deliberately not a control layer.** It does not detect that a node is behind, does not choose a peer,
does not time a retry and does not decide to fall back to a snapshot. It owns no clock and knows no heights.
Those are the caller's, and keeping them out is what makes the rest of the properties hold.

## Why it can be protocol-independent

Because there is no vocabulary in which a protocol-specific type could be expressed. Session ids, chunk counts
and peer ids mean the same thing everywhere.

**And there is nothing to inject but an observer.** An earlier draft made **admission** — whether an entry is
valid and who may serve it — a trait boundary. It is not one: beggar never sees an entry. The caller validates, and
only then reports that a chunk arrived; a rejected entry is simply never reported. No clock either. Every answer is
returned, and reported to the observer the instance was constructed with, which answers nothing.

**Nothing injected that answers**, which is why a conformance suite can be run against any implementation in any
language without stubbing anything.

## The spec is the artefact; this crate is a sample

The intent is a **programming-language-independent specification** of beggar, with this Rust crate as one
conforming implementation rather than the definition. That is practical here for reasons specific to the design:
no wire types means no encoding to specify, no clock means no temporal logic, and a registry rather than a state
machine means the normative core is a data model plus one total function — the matrix in
[`docs/DESIGN.md`](docs/DESIGN.md) §4.

What will make it a specification rather than prose is `spec/`, written first and by hand: a JSON Schema data
model, a JSON decision table, normative prose, and a **committed, machine-readable conformance suite** in JSON: vectors of `(initial state, command sequence) → (expected receipts, final state)`, consumed by this
crate's tests and by any other implementation's.

## Sibling to `faction`, and the difference is load-bearing

[`faction`](https://crates.io/crates/faction) is the same idea applied to the moment a cluster *starts*.

**Faction is a state machine; beggar is not.** Membership has genuine states a node moves between, and
faction's `Bootstrapped` is terminal in a way its invariants depend on. Beggar has no states: every outcome is a
function of the command and the session record, so there is no label doing any work.

That single difference decides how each is tested — **an FSM needs transition coverage, a registry needs
invariants** — and neither should borrow the other's strategy.

One thing both share: **serving must always work.** The node that answers a request is precisely the one already
in sync, so anything that only accepts serve requests while behind goes deaf exactly when it is the one peers
need. In a registry this is automatic rather than a rule to remember.

## Observability

One entry point, `(receipt) = process(command, state)`, and every call is logged whole. Because the outcome
is a pure function of command and record, a capture is **replayable**: the crate can be validated decision by
decision instead of against aggregates.

Every call is reported to the **observer** the instance is constructed with, as `faction`'s is: the command and
its receipt, once, a refusal and a query included. `NoOpObserver` is there for a caller with no use for one.

Every call is answered with a **receipt numbered by beggar**, one more than the last, so a gap in the numbers
*proves* loss. Absence
of a log line otherwise proves nothing.

## Licence

MIT. See [LICENSE](LICENSE).
