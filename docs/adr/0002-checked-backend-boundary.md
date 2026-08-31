# ADR-0002: Backends accept only checked programs

- Status: accepted
- Milestone: M00
- Date: 2026-08-31

## Context

If emitters resolve names or reinterpret ill-typed expressions independently,
target behavior can diverge and a future frontend can bypass semantic checks.

## Decision

- Unchecked IR retains names and authoring source information.
- One target-independent checker resolves and validates the program.
- Checked-program constructors remain private to the checker.
- Every safe backend API accepts `CheckedProgram`, never unchecked IR.
- Target preflight may reject unsupported checked capabilities but may not alter
  their meaning.

## Alternatives considered

- Validate inside each emitter: rejected because diagnostics and accepted
  programs would drift by target.
- Let the Rust type system alone validate builder calls: rejected because loaded
  IR and future parsers require the same runtime checker.
- Allow backend recovery from invalid nodes: rejected because plausible output
  would conceal a non-portable program.

## Consequences

Frontends and backends are independently extensible around a stable semantic
waist. The checker is a critical component and must produce deterministic,
structured diagnostics.

## Enforcement

M04 owns the opaque checked type. M08 compile tests prove backend code cannot
construct it or call emitters with unchecked IR. Every generated fixture passes
through the checker first.
