# P0-ADR-ARegistryNotAStateMachine

- **Status:** Accepted
- **Date:** 2026-11-06
- **Priority:** P0 (axiom)

## Context

A node that fell behind faces three uncertainties it cannot resolve from
inside: whether a peer is ahead, whether it will serve, and whether to ask one
peer or several. Silence and refusal look identical from outside. Initiating a
recovery is begging for help, and the design refuses to assume any of the
three answers.

`faction`, the sibling crate, is a state machine — membership has genuine
states a node moves between — and the same shape is the obvious default here.

## Decision

Beggar is a registry of sessions, not a state machine: every output is a
function of the command and the session's record, and no state label does any
work.

## Forcing constraints / Evidence

The test that settles it: in a state machine, the same command with the same
arguments yields different results in different states. Here it never does.

A machine over these uncertainties would need a label telling *waiting on a
willing peer* from *waiting on an unwilling one*. That distinction is
unknowable from inside, because the two produce identical observations; any
such label would be a guess encoded as structure. A registry records what
arrived and says nothing about what a silence means.

## Rejected alternatives

**A state machine, `faction`'s shape.** Rejected, for the reason above.

**A refusal command.** Rejected: an unwilling peer simply never produces a
partial, and beggar cannot tell refusal from silence. Willingness survives in
the ledger only as the reason the caller gives when it cancels.

**Returning the admissible commands with every output**, as `faction` does.
Rejected: the set only means something relative to a state. What a sender
needs in order to resend is data — a query of the missing chunk ids.

## Consequences

Correctness is invariants, not paths: the chunks received are within
`1..=chunks`, no record survives a terminal output, and the mapping from
command to output is total. A registry is tested by invariants and properties;
it does not borrow a state machine's transition coverage, and `faction` does
not borrow its invariants.

Serving needs no rule to stay possible. The node that answers a request is the
one already in sync, and a registry refuses it nothing.

The stance is about not assuming, not about not acting. A caller may ask
repeatedly and ask several peers at once; what beggar may not do is presume
the answer.

## Enforcement

The matrix in `../DESIGN.md` §4: every row's output follows from the command
and the session's record alone, and the conformance vectors check every row.

## Related

`faction`'s ADRs — the state machine this registry deliberately is not.
