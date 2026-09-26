# beggar

Protocol-agnostic recovery for replicated logs.

**Version 0.0.0 reserves the name. There is no implementation yet — do not depend on this crate.**

## The idea

Recovery is a supplicant. A node that fell behind — severed by a partition, or simply slow — has no
leverage over the peers that did not. It asks for the entries it missed, the peer **may refuse**, it may be
handed a snapshot instead of the blocks it asked for, and if ignored its only recourse is to ask again.

That asymmetry is the whole mechanism, and it is the reason for the name.

## Why it can be protocol-agnostic

The control layer generalises: detect that you are behind, choose a peer, request from index `H` for `N`
entries, wait, retry on timeout, fall back to a snapshot when the peer has pruned past you, stop at parity.
None of that mentions a particular consensus protocol.

What does not generalise is **admission** — whether an entry is valid and who is entitled to serve it. In
IBFT that is chain continuation from a validator; in Raft it is term and log-index matching. So admission is
the trait boundary rather than an implementation detail.

## Intended shape

A state machine that computes actions rather than performing them: timers and sends are returned as values
and executed by the host. That makes the whole of recovery testable without a network, which matters
because a partitioned node is close to unobservable in practice — the log path is severed along with the
data path.

Transitions are intended to be emitted as `(from, event, to, epoch)` with a monotonic counter, so that a
gap in the counter *proves* loss. Absence of a log line otherwise proves nothing.

## Licence

Apache-2.0. See [LICENSE](LICENSE).
