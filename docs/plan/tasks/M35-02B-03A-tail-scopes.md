# M35-02B-03A — Preserve tail-nested ownership scopes

- Status: complete
- Parent: [M35-02B-03](M35-02B-03-structured-owned-places.md)
- Depends on: M35-02B-02
- Specification: [tail-nested scopes](../../specification/typed-generation/languages/c/rust-owned-tail-scopes.md)

## Contract

Extend one-constructor whole-value correspondence to tail-nested lexical blocks.
Keep the actual structured HIR scope tree, each binding's declaration scope and
the final read/drop scope. A nested block must not become an anonymous flattened
list of operations or give a moved outer owner another cleanup obligation.

This first structured increment retains one finite execution path and one final
owner. Multiple owners, sibling non-tail blocks, branch/early-return exits,
partial moves and owned function transfers remain explicit later increments.

## Definition of done and tests

- Every admitted scope is canonical HIR with a retained parent/child relation;
  bindings and final dereference carry their actual scopes.
- Reuse the established typed producer/move/read/drop relation rather than
  authorizing a mapping through source spans, debug names or matching counts.
- Zero-, one- and several-level nesting, construction inside a nested block,
  ownership moved inward, shadowing and empty wrapper blocks pass.
- Outer moved owners have no additional drop; cleanup belongs to the final
  live binding's scope before returning through enclosing tail blocks.
- Scope substitutions, duplicate/omitted scopes, wrong binding scope, extra
  cleanup and unsupported sibling/branch/escape shapes reject in tests.
- Preserve root-only coverage and all private evidence/registration contracts.
  Focused and full isolated Linux/Bazel gates, fresh independent review,
  documented evidence, commit and push complete the increment.

No C/Java heap output is enabled by this compiler-only checkpoint.

## Verified checkpoint

- Focused Linux/Bazel gate `9f37a322-2ea0-40f4-9e02-0ff796f08c10`: 6/6 tests
  passed in 24.631 seconds, including root-only regressions, scope metadata/MIR
  mutation controls, exact privacy failures, Clippy and fixture/module format.
- The source reader records scope claims, and a separate canonical containment
  walk certifies them. The unchanged MIR relation authenticates the one final
  live owner and its actual drop; read and owner/drop scopes are separate.
- Isolated tree `f8ce82bf5b069b15bd4d124ca31bb569dd6e1a86`: full gate
  `1b33118c-3d82-4404-bc6c-1476379ed32c` passed 390/390 tests across 508
  targets in 46.285 seconds. Eleven tests executed; all valid cached results
  remained enabled. Historical C/Java/native proofs, Rust/Bazel lint and format,
  shared codegen contracts and documentation remained enabled.
- Fresh independent Sol Extra High review found no core findings or optional
  changes. It confirmed canonical containment, separate owner/read scopes,
  the single-chain drop association, private evidence, mutation non-vacuity,
  root-only regression behavior and Bazel/source coverage.
- The exact Git archive's blob bytes and executable modes were verified. The
  final documentation-inclusive tree and successful gate are recorded in the
  checkpoint commit message before push. No heap output was enabled.
