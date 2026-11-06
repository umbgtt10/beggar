# beggar

A `no_std`, protocol-independent **recovery registry**.

**Version 0.0.1 reserves the name. There is no implementation yet — do not depend on this crate.**

The foundations are four ADRs in [`docs/ADRs/`](docs/ADRs/README.md); the design is
[`docs/DESIGN.md`](docs/DESIGN.md). Both are authoritative; this file is the summary. The order of building it is
[`docs/IMPLEMENTATION-PLAN.md`](docs/IMPLEMENTATION-PLAN.md).

## The idea

Recovery is a supplicant. A node that fell behind — severed by a partition, or simply slow — has no leverage
over the peers that did not. It asks for the entries it missed, the peer **may refuse**, it may be handed
something other than what it asked for, and if ignored its only recourse is to ask again.

That asymmetry is the whole mechanism, and it is the reason for the name.

## What it actually does

**Pure accounting.** Beggar is a registry of ongoing transfer sessions. A session is opened with an expected
chunk count and two node identities, and beggar names it; the peer's registry adopts the name. Each chunk that
moves is recorded against it; when the set is full, or the caller cancels, the session terminates and is evicted.
A command it refuses gets an output saying why — never an exception.

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

**And there is nothing to inject.** An earlier draft made **admission** — whether an entry is valid and who may
serve it — a trait boundary. It is not one: beggar never sees an entry. The caller validates, and only then
reports that a chunk arrived; a rejected entry is simply never reported. No clock either, and emission is
returned rather than pushed to a sink.

**Zero injected dependencies**, which is why a conformance suite can be run against any implementation in any
language without stubbing anything.

## The spec is the artefact; this crate is a sample

The intent is a **programming-language-independent specification** of beggar, with this Rust crate as one
conforming implementation rather than the definition. That is practical here for reasons specific to the design:
no wire types means no encoding to specify, no clock means no temporal logic, and a registry rather than a state
machine means the normative core is a data model plus one total function — the matrix in
[`docs/DESIGN.md`](docs/DESIGN.md) §4.

What will make it a specification rather than prose is a **committed, machine-readable conformance suite** in
JSON: vectors of `(initial state, command sequence) → (expected output sequence, final state)`, consumed by this
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

One entry point, `(output) = process(command, state)`, and every call emits the whole tuple. Because the outcome
is a pure function of command and record, a capture is **replayable**: the crate can be validated decision by
decision instead of against aggregates.

Every command carries a **caller-supplied monotonic ordinal**, so a gap in the sequence *proves* loss. Absence
of a log line otherwise proves nothing — and a partitioned node is close to unobservable in practice, because
its log path is severed along with its data path.

## Licence

MIT. See [LICENSE](LICENSE).
