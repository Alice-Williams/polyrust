# M35-03A-02F-02B — Public scalar constant APIs

- Status: in-progress
- Parent: [M35-03A-02F-02](M35-03A-02F-02-constant-declarations.md)
- Depends on: M35-03A-02F-02A

## Contract

Add explicit constant declaration/reference identities to the certified C and
Java package and dependency APIs. Preserve crate/module ownership, effective
visibility, aliases/re-exports and documentation. Constants-only packages must
work without synthetic functions or indexing an empty function-root list.

Before implementation, write dedicated per-language specification and bounded
child tasks for registry/certification, dependency manifests and HIR integration.
Reuse typed C object declarations/definitions and Java static-final fields.
Public reads must use authenticated target constant references; private/local
reads may remain folded. No raw source fallback or custom runtime is permitted.

C exposes a const-qualified exact-width object via an extern header declaration
and one ordinary source definition. This is a value API, not a promise that the
C name is an integer constant expression. Java exposes a primitive static final
field with a compiler-evaluated literal initializer. Reference/storage address
semantics and wider types remain separate capabilities.

## Definition of done and tests

- Independently compiled consumers read exported bool/i32/i64 values from
  constants-only, mixed and multi-crate packages, including aliases.
- Registered identity, type, value, readonly/finality, declaring owner and
  emitted declaration agree; disconnected/mutated target nodes fail checks.
  Coupled semantic mutations must fail independent native truth.
- Public reads import authenticated dependency constants; private declarations
  are not accidentally exported. Stale metadata fails before publication.
- Preserve inspectable documentation and complete export inventories.
- Existing local/private constant proofs remain green. Add compile-negative,
  target AST, dependency, native, readonly-assignment and atomic rejection tests.
- Per-language specs/tasks, evaluated independent reviews, exported examples
  and isolated full Bazel/lint gates precede scoped commits and pushes.

## Ordered implementation

1. [01 — Shared certified dependency values](M35-03A-02F-02B-01-dependency-values.md) — complete.
2. [02 — Certified C constant declarations and imports](M35-03A-02F-02B-02-c-constant-api.md) — complete.
3. [03 — Certified Java constant fields and imports](M35-03A-02F-02B-03-java-constant-api.md) — complete.
4. [04 — Compiler public constants and constants-only packages](M35-03A-02F-02B-04-source-constant-mappings.md) — complete.
5. [05 — Multi-crate constant metadata and integration proof](M35-03A-02F-02B-05-constant-bundles.md) — in progress.

Specifications precede implementation: [shared source](../../specification/typed-generation/rust-public-constants.md),
[dependency values](../../specification/typed-generation/certified-dependency-values.md),
[C17](../../specification/typed-generation/languages/c/rust-public-constants.md), and
[Java21](../../specification/typed-generation/languages/java/rust-public-constants.md).
Each child has independent exit evidence and a separate reviewed commit/push.
