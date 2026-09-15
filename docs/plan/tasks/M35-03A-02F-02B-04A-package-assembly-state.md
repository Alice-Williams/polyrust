# M35-03A-02F-02B-04A — Function-independent package assembly

- Status: complete
- Parent: [compiler public constants](M35-03A-02F-02B-04-source-constant-mappings.md)
- Depends on: M35-03A-02F-02B-03

## Contract

Implement [package/function state](../../specification/typed-generation/rust-source-package-state.md).
Separate package registration/output state from function analysis in both C and
Java compiler lowerers. Package state owns registries/builders, declarations,
function bindings, foreign bindings, origin caches and package-wide budgets.
It must not require a function root, TypeckResults or a dummy body.

Construct a fresh Reader only for an actual registered function. Move persistent
state into that reader and recover it after lowering, while resetting local
bindings, temporary counters, control scopes and per-expression depth. Preserve
Java's package-wide remaining-expression budget across functions. Record identity,
dependency authority, stable ordering and registry identity must survive.

Extract body lowering and package-state lifecycle into focused modules. Remove
roots[0] initialization. An empty body inventory is a valid no-op at this internal
layer; public API admission remains unchanged until child04B supplies complete
constant declarations, reads and export evidence.

## Definition of done and tests

- Existing selected-entry/public-package native behavior, docs, visibility,
  call/record identity and certification remain unchanged.
- Direct compiler probes exercise the actual empty-body helper and verify no
  function/record/member is synthesized and registration authority is preserved.
- Multi-function probes verify fresh local state and preserved package budgets;
  no per-function budget reset permits a package to exceed existing limits.
- Existing compiler AST/production-byte comparisons, native tests, negative
  admission, capability contracts and full release/lint gates pass.
- Independent review and a scoped tested checkpoint precede child04B.
  This refactor alone does not enable constants-only source packages or retire
  any legacy functionality.

## Implementation and proof

Both compiler lowerers now have a focused package.rs lifecycle module. Their
State types contain persistent registration state but no current function or
TypeckResults. Actual body loops construct Readers only after finding a registered
function, and recover the same package authority afterward. Empty loops return
unchanged state and no body output. Facade/file assembly stays outside Readers.

Direct C and Java compiler probes exercise empty inventories, rejection of an
unregistered crate root before type checking, exact current function/type-check
identity, unchanged function/import maps and shared origin cache identity. They
poison counters/local state across Reader round trips and check fresh state.
Java probes additionally carry a shrinking budget, verify zero-budget rejection
and restore the original budget before production output comparisons. The C
harness requires all three probe executions; the Java harness requires its marker
for each of nine existing native/AST fixtures.

Seven-target preflight tree 878d3dbd090ac2ebb74e704457753c84aaddadf4 passed in
23.377 seconds, invocation 86caa98d-b8b2-46af-abdb-516b9e645582: C package manifest
probe, Java source AST/native comparisons, docs, buildifier, rustfmt, Clippy and
source policy. An earlier command used a nonexistent Java target name; it is not
counted as a passing gate. The corrected invocation includes the actual target.

A fresh Sol Extra High review audited both production changes and the subsequent
C probe-execution wrapper in tree 8c23fa76f181b8f2d48b15076d47b8c36f27b2c8. It found
no concrete implementation, architecture or proof defects. The isolated full gate for that tree passed all 624 tests across 816 targets in
149.707 seconds, invocation ec6c5858-e52d-42db-9c88-06159c3186ff. It exercised 50
noncached test targets, including compiler/native/integration proofs, with all
existing C/Java/shared suites and Rust/Bazel lint gates enabled. The archive's
2,578 Git blobs and executable modes were verified before testing.
Completion bookkeeping is gated again before the scoped commit and push.

Actual Bazel-generated public C and Java package artifacts were exported and
inspected at ignored host directory generated/examples/source-package-state-8c23fa7,
under c and java. They contain source and metadata, not compiled classes or a
custom runtime. No generated artifact is staged.
