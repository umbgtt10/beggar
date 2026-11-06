# Architecture Decision Records

Each ADR documents one load-bearing property of `beggar` — succinct,
self-contained, citable on its own. Priority marks the dependency tier, not
importance:

- **P0** — foundational axiom; the registry's identity rests on it.
- **P1** — derived or shape decision; follows from a P0.
- **P2** — structural; real, but below the core.

The ADRs say *why*; [`../DESIGN.md`](../DESIGN.md) says *how* — the vocabulary,
the matrix, the semantics, retention, observability and memory.

## Index

| ADR | Property |
|---|---|
| [P0-ADR-ARecoveryIsACollectionOfChunks](P0-ADR-ARecoveryIsACollectionOfChunks.md) | Logs or snapshots, IBFT or Raft, pushed or pulled: beggar knows sessions, chunk counts, chunk ids and node ids, and nothing else. No bits or bytes. |
| [P0-ADR-ARegistryNotAStateMachine](P0-ADR-ARegistryNotAStateMachine.md) | Every output is a function of the command and the session's record; no state label does any work. |
| [P0-ADR-AccountsNeverDecides](P0-ADR-AccountsNeverDecides.md) | No detection, no peer choice, no timing, no policy; zero injected dependencies. The control layer sits above. |
| [P0-ADR-SpecificationIsNormativeCrateIsReference](P0-ADR-SpecificationIsNormativeCrateIsReference.md) | A language-independent specification with JSON conformance vectors; this crate is one conforming implementation. |
| [P1-ADR-TheAccountantNamesTheSession](P1-ADR-TheAccountantNamesTheSession.md) | `StartRecovery` mints the name — instance id and a count — and the peer adopts it; the instance id never repeats, restarts included. |
| [P1-ADR-OneEntryPointOneOutput](P1-ADR-OneEntryPointOneOutput.md) | One `process(command)`, one output for every command on a numbered receipt — queries and refusals included — never an exception. |
| [P1-ADR-TheLogIsTheRecord](P1-ADR-TheLogIsTheRecord.md) | Every call answered with a receipt numbered by beggar, logged as `(number, command, output)`; state rebuilt by replay; the checkpoint a query. |
| [P1-ADR-RetentionGovernsQueriesNeverEmission](P1-ADR-RetentionGovernsQueriesNeverEmission.md) | What is kept changes what a query answers, and nothing else; every terminal is emitted. |
| [P1-ADR-CancelEvictsWithoutATombstone](P1-ADR-CancelEvictsWithoutATombstone.md) | A cancel evicts and wins; no tombstones; ending abandoned sessions is the caller's stated policy. |

## Template

```markdown
# <Priority>-ADR-<Name>

- **Status:** Accepted | Proposed | Superseded by <ADR>
- **Date:** YYYY-MM-DD
- **Priority:** P0 (axiom) · P1 (derived) · P2 (structural)

## Context
The forces and tension this resolves.

## Decision
The choice, in one quotable sentence.

## Forcing constraints / Evidence
Why this was forced, not freely chosen — the real evidence. `N/A` if none.

## Rejected alternatives
What we did not do, and why.

## Consequences
What it commits us to; what it costs; obligations pushed onto consumers.

## Enforcement
The specific test, gate, or structural mechanism that keeps it true.

## Related
Links to other ADRs (same or sibling repos) and architecture docs.
```

Fields that do not apply are marked `N/A` rather than padded.

Each ADR is a snapshot of the decision as it stands today, not a changelog:
state the current shape as fact, and do not narrate what an earlier version
of this document — or of any other document — used to say. Keep
cross-references minimal; link another ADR only where the relationship is
load-bearing.
