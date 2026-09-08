# M34A-11-02C — C contextual scope and control-flow verification

- Status: planned
- Depends on: M34A-11-02B

## Goal

Implement this bounded part of M34A-11-02 without introducing a raw-source path
or advertising capabilities before their mappings exist.

## Definition of done

- Verify namespace/linkage/file-role legality, complete object requirements, prototype/definition agreement, exact member/call types and return coverage.
- Independently rederive file-initializer and static-assertion constant-expression
  categories from their actual trees. Forward complete-object obligations for
  SizeOf/AlignOf and constant arithmetic/assertion truth to the 02D proof stage.
- Verify initialization on all reachable paths, duplicate switch constants after promotion, exhaustive non-fallthrough arms, bounded loop targets, labels and forward cleanup exits.
- Keep verifier evidence private and derived from actual registrations and AST. Unknown target input is fallible; target verification cannot certify legacy source.

## Tests and proof

- Positive/rejected mutation for every contextual rule, including crossed scopes and aliases, missing/mutually deleted registration evidence and duplicate promoted switch values.
- Control-flow tests for branch initialization joins, unreachable exits, wrong break/continue targets and labels bypassing declarations.
- Exact counted-loop containing/body scopes and counter/bound declaration
  dominance, including Continue across nested switches; follow c/counted-loops.md.
- Every Switch carries its exact registration. Reject duplicate statement
  occurrences, wrong function/scope, crossed switch IDs and Break naming an
  outer loop/switch. Continue targets the innermost loop across nested switches.
- Block/root scope identities match actual lexical parent/function ownership
  with exactly one occurrence. Reject swapped siblings, duplicated/absent scopes,
  wrong parents and local declarations in another scope; accept dominating
  ancestor reads and legal distinct shadow bindings.
- Full Rust/lint/compile-fail and cached tracked/release/eight-target gates.

## Commit gate

Record exact commands, invocation IDs and outcomes. Commit and push this slice
with M34A-11-02C; keep its parent M34A-11-02 and overall C compliance open
until their remaining obligations pass. Use focused modules below the source
size limits and distinct Bazel targets only at real independent boundaries.
