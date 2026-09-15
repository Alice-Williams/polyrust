# M35-03A-02F-02A — Block-local scalar constant declarations

- Status: complete
- Parent: [M35-03A-02F-02](M35-03A-02F-02-constant-declarations.md)
- Depends on: M35-03A-02F-01
- Specification: [local constants](../../specification/typed-generation/rust-local-constants.md)

## Contract

Add LocalConstants with a private compiler-derived declaration input and
executable C/Java mappings. Admit only actual const item statements of
bool/i32/i64 type inside admitted function blocks. Evaluate and validate them
with the same compiler definition checks as scalar constant reads. Do not
silently skip unknown item statements or unsupported unused constants.

A local const declaration has no runtime initialization or addressable local
binding. The target mapping explicitly returns unit without adding target
statements, locals, fields, helpers or imports. Later reads keep resolving the
compiler DefId through ScalarConstants, including nested shadowing and forward
references. Ordinary let scope/evaluation behavior is unchanged.

## Definition of done and tests

- Native Rust/C/Java agree for all three scalar types, both bool values, exact
  signed boundaries, computed initializers, forward references, unused admitted
  declarations and nested same-spelling consts. Include branches, ordinary
  locals, records and a real dependent crate.
- Inspect actual declaration mappings: no target prelude/local/storage changes,
  exact compiler statement/definition/type/value, byte-identical production.
  Read probes and independent native values distinguish shadowed definitions.
- Missing/duplicate/wrong capability/context/output/input/private-input
  compile-negative tests pass for both backends.
- Invalid/panicking/long-running initializers, unsupported widths/arrays,
  storage borrows, static/type/function/module item statements and duplicate
  names reject atomically. Existing public-constant rejection remains intact.
- Convert the prior simple local-const rejection into explicit positive
  evidence; retain an unsupported local-constant negative in its place.
  No test is disabled because the feature has become supported.
- Keep files focused; Rust/Bazel lint, strict native consumers, full isolated
  gate, evaluated independent reviews and ignored example exports precede
  the dedicated milestone commit/push.

## Progress

Specification preceded implementation. The private declaration input, shared
compiler evaluator, executable C/Java unit-output mappings and sixteenth builder
slot are implemented. Public declarations remain 02F-02B.

Initial candidate `055d95a1e4469918efd99bc25f6e2db83f7ed5af` passed 18 of
19 targeted tests: native equivalence/mutation, fourteen compile-negative
contracts, fixture Clippy and both rejection suites. The AST probe did not build
because it used the compiler's unnormalized type API. Both probes now explicitly
normalize that compiler type. The subsequent full gate and independent reviews
are recorded below.
No test was disabled and no legacy runtime was removed.

## Verification evidence

Tree `4ff1910540f5224f733727dc2aa11ec63616aba9` passed all 624 tests
across 816 targets in the isolated Linux/Bazel gate (42.305 seconds, invocation
`053d3503-8083-4dcb-874d-8b91e35d47d2`). The archive matched 2,511 exact
Git blobs and executable modes. Rust and Bazel linters passed and unchanged
test results remained cached.

Native proof covers 30 independently specified results across 15 public root
functions and one separately compiled dependency function, with GCC/Zig O0/O2
and Java 21 strict warnings. A wrong computed value fails native truth in both
targets. Actual mapper probes check 21 declarations and 20 reads per target;
exhaustive Reader projections are unchanged and probe/production artifacts
match byte-for-byte. Fourteen new compile-negative contracts, 68 local atomic
rejection cases and all 68 prior constant-read rejections pass.

The first broad review confirmed missing imports in two older negative fixtures
and an incomplete state snapshot. Both findings were accepted: imports restore
the intended ten E0599 controls; exhaustive snapshots now retain all output-
relevant maps, registries, counters and origin caches. Immutable compiler
handles use identity checks. The initial probe normalization/debug API build
errors were also repaired. No production constant semantics were changed by
these proof repairs. The first reviewer confirmed those repairs; a fresh Sol
Extra High blind review of the passing 4ff191 tree found no errors or optional
extensions. No finding was dismissed or remains unresolved.

Tested, ignored examples are exported to `generated/m35-local-constant-4ff1/`.
Root C: `c/polyrust_e14d20761cfd7cfe.c`. Root Java:
`java/src/main/java/org/polyrust/generated/re14d20761cfd7cfe/Generated.java`.
Both bundles include the separately generated dependency; exported bytes match
the tested bundles. No generated output or legacy removal is staged.

This completes only the block-local declaration step. Public/dependency
constant APIs remain 02F-02B; complete legacy parity and runtime removal remain
open. Unrelated ownership work was preserved and its 20-file hash inventory
rechecked unchanged.
