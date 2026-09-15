# M35-03A-02F-01 — Compiler-evaluated scalar constant reads

- Status: complete
- Parent: [M35-03A-02F](M35-03A-02F-scalar-constants.md)
- Depends on: M35-03A-02E
- Specification: [scalar constant reads](../../specification/typed-generation/rust-scalar-constants.md)

## Contract

Add a private checked ConstantInput and executable ScalarConstants slot for
C/Java. Resolve module/inherent-associated constant paths to compiler DefIds;
accept only nongeneric, unadjusted bool/i32/i64 reads. Evaluate using the
pinned compiler, preserve the resolved origin, and emit ordinary typed literal
nodes. Do not parse initializer text, trust a user-provided value, or borrow
a constant's storage. Initializer arithmetic can be evaluated by rustc without
enabling the same arithmetic at runtime.

Public constant export declarations, block-local constant declarations,
generic/trait-associated constants and other value types are separate work.
Existing rejection of unmapped public declarations must remain intact.

## Definition of done and tests

- Native two-crate Rust/C/Java consumers agree with independent exact values:
  signed boundaries, computed initializers, bools, same-spelling constants in
  different modules, aliases, inherent/primitive associated constants, record
  fields, branches and imported function arguments.
- Actual mapper probes check constant origin, type and literal value, with
  production-byte identity. Deliberately wrong values must fail native truth.
- Missing/duplicate/wrong capability/context/output/input and private input
  compile-negative checks pass separately for C and Java.
- Overflow, invalid/panicking constants, generic and storage/unsupported forms
  reject before publication, preserving existing output. Long-running constant
  evaluation cannot disable the compiler limit through a source lint attribute.
- Runtime-free artifact inventories, GCC/Zig O0/O2, Java 21 warning-clean
  consumers, all existing Bazel/lint gates and independent reviews pass.
- Export ignored examples; commit/push only this completed bounded step.

## Progress

Specification preceded implementation. The corrected source and test tree
`a5b4581d69161deb3badda01f20823a75cf0b71c` passed all 606 tests across
789 targets in the isolated Linux container (68.761 seconds; invocation
`31f79d3e-4a06-4077-be5a-5930348a0a3e`). The archive matched 2,490 exact
Git blobs and executable modes. Rust and Bazel linters passed; unchanged test
results remained cached.

The proof includes 38 independent exact native results across two real crates,
separate GCC/Zig O0/O2 library/consumer compilation and Java 21 strict warnings.
A wrong-value mutation is detected in both targets. Actual mapper probes inspect
23 constant reads per target, checking resolved origin and exact scalar AST
type/value with byte-identical production output. Fourteen compile-negative
contracts and 52 absent/existing-output atomic rejection checks pass.

The initial review independently confirmed four verification issues already
fixed in this candidate: a Clippy eq_op fixture, target-specific public-API
diagnostic wording, the target-free capability probe's missing new input, and
lack of a direct false-valued constant fixture. All were accepted and fixed;
the first review found no production correctness error.

The next independent review found an admission-contract error: constants on
concrete impls of generic nominal types passed the expression/definition
generic checks. This included omitted default type/const arguments. Although
their folded values were correct, the acceptance contradicted the explicit
F01 generic exclusion, so the finding was accepted. The repair normalizes the
compiler's inherent impl self type and rejects semantic generic arguments or
unmodeled owner kinds. Four regression shapes run against both adapters with
absent/existing outputs, extending the atomic matrix to 68 checks. The full post-fix
gate and fresh independent review passed as recorded below.

Tested, ignored examples are exported under `generated/m35-constant-a5b4/`.
The root C implementation is `c/polyrust_2b9474c26c32e326.c`; the root Java
source is `java/src/main/java/org/polyrust/generated/r2b9474c26c32e326/Generated.java`.
Both bundles also contain their separately generated dependency. No generated
outputs or legacy runtime removals are included in this checkpoint.

## Closure evidence

Post-fix tree `4f4cad78a81b9eec7e47fddc2cbc0a91ecffb99c` passed all
606 tests across 789 targets in 203.020 seconds, invocation
`00be018d-c5c0-4b7a-ae1e-a983bb9b5552`. All 68 atomic checks, native and
typed-AST proofs, mapping contracts and Rust/Bazel linters pass. The archive
matched 2,490 exact Git blobs/modes. Exported examples are byte-identical to
these post-fix tested bundles.

The finding reviewer confirmed the repair with both rebuilt adapters and found
no second core error. A separate fresh Sol Extra High blind review of the
post-fix tree found no core or optional findings. Its additional launcher
probes covered crate-qualified and qualified-inherent paths, enum-associated
constants, false values, exact wide negatives and private const-function
initializer evaluation. No finding was dismissed or left unresolved.

This completes only scalar constant reads. Public/local declarations, wider
types, runtime arithmetic, full catalogue parity and legacy removal remain
separate planned work.
