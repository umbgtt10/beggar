# beggar

A `no_std`, protocol-independent recovery state machine.

**Version 0.0.1 reserves the name. There is no implementation yet — do not depend on this crate.**

## The idea

Recovery is a supplicant. A node that fell behind — severed by a partition, or simply slow — has no
leverage over the peers that did not. It asks for the entries it missed, the peer **may refuse**, it may be
handed a snapshot instead of the entries it asked for, and if ignored its only recourse is to ask again.

That asymmetry is the whole mechanism, and it is the reason for the name.

## Why it can be protocol-independent

The control layer generalises: detect that you are behind, choose a peer, request from index `H` for `N`
entries, wait, retry on timeout, fall back to a snapshot when the peer has pruned past you, stop at parity.
None of that mentions a particular consensus protocol.

What does not generalise is **admission** — whether an entry is valid and who is entitled to serve it. In
IBFT that is chain continuation from a validator; in Raft it is term and log-index matching. So admission is
the trait boundary rather than an implementation detail.

## Sibling to `faction`

[`faction`](https://crates.io/crates/faction) is the same idea applied to the moment a cluster *starts*.
This crate is intended to follow its shape: a deterministic state machine the host drives, one `process()`
entry point, a read-only probe carried as a command rather than a side door, and the set of admissible
commands returned on every result — so "why is it not asking?" is answered by the machine instead of
inferred from logs.

Two things differ, and both change the design:

- **Faction converges once.** `Bootstrapped` is terminal, and that terminality is load-bearing for its
  invariants. Recovery is cyclic — re-entered at every partition — so episodes need an explicit counter.
- **Serving must stay admissible in every state.** The node that answers a request is precisely the one
  already in sync. A machine that only accepts serve requests while behind goes deaf exactly when it is
  the one peers need.

## Intended shape

Timers and sends are returned as values and executed by the host, so the whole of recovery is testable
without a network. That matters because a partitioned node is close to unobservable in practice: its log
path is severed along with its data path.

Transitions are intended to be emitted as `(from, event, to, epoch)` with a monotonic counter, so that a
gap in the counter *proves* loss. Absence of a log line otherwise proves nothing.

## Licence

MIT. See [LICENSE](LICENSE).
