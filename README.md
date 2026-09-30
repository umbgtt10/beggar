# beggar

A `no_std`, protocol-independent **recovery registry**.

**Version 0.0.1 reserves the name. There is no implementation yet — do not depend on this crate.**

The design is [`docs/DESIGN.md`](docs/DESIGN.md), settled 2026-09-30. It is authoritative; this file is the
summary.

## The idea

Recovery is a supplicant. A node that fell behind — severed by a partition, or simply slow — has no leverage
over the peers that did not. It asks for the entries it missed, the peer **may refuse**, it may be handed
something other than what it asked for, and if ignored its only recourse is to ask again.

That asymmetry is the whole mechanism, and it is the reason for the name.

## What it actually does

**Pure accounting.** Beggar is a registry of ongoing transfer sessions. A session is opened with an expected
chunk count and two node identities; each chunk that moves is recorded against it; when the set is full, or the
caller cancels, the session terminates and is evicted.

It deals in **no bytes, blocks, arrays or wire types** — session identities, chunk counts, node identities, and
nothing else. The payload moves elsewhere; beggar records that it moved.

**It is deliberately not a control layer.** It does not detect that a node is behind, does not choose a peer,
does not time a retry and does not decide to fall back to a snapshot. It owns no clock and knows no heights.
Those are the caller's, and keeping them out is what makes the rest of the properties hold.

## Why it can be protocol-independent

Because there is no vocabulary in which a protocol-specific type could be expressed. Session ids, chunk counts
and peer ids mean the same thing everywhere.

What does **not** generalise is **admission** — whether an entry is valid and who is entitled to serve it. In
IBFT that is chain continuation from a validator; in Raft it is term and log-index matching. So admission is an
**injected trait**, asked and never inspected.

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
