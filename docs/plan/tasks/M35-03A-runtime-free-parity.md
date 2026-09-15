# M35-03A — Runtime-free C/Java feature parity and cleanup

- Status: in-progress
- Parent: [M35-03](M35-03-rustc-integration.md)
- Specification: [ordinary generated packages](../../specification/typed-generation/runtime-free-packages.md)
- Priority: user-requested migration planning and guards may proceed before M35-02 completes

## Contract

Remove the old C runtime templates and Java custom runtime architecture without
dropping supported functionality. Implement missing operations in the new
Rust-source system, through its typed mappings and normal target packages.
Retain legacy entry points/tests until replacement coverage is complete.
Do not claim heap parity from source-only ownership evidence.

## Ordered work

1. [03A-01 — Inventory and artifact guards](M35-03A-01-runtime-parity-inventory.md).
2. [03A-02 — Scalars, constants and numeric operations](M35-03A-02-scalar-parity.md).
3. Finish [M35-02](M35-02-rustc-owned-values.md) before enabling owned target shapes.
4. [03A-03 — Nominal types and structured behavior](M35-03A-03-nominal-parity.md).
5. [03A-04 — Text, Unicode and bytes](M35-03A-04-text-parity.md).
6. [03A-05 — Collections and option/result operations](M35-03A-05-collection-parity.md).
7. [03A-06 — Corpus cutover and deletion](M35-03A-06-legacy-retirement.md).

The implementation steps must be split into smaller operation-specific tasks
before enabling each mapping. This umbrella is not permission to combine
unrelated capability changes in one commit or remove unfinished proof checks.

## Definition of done

Every supported legacy C/Java feature has a documented and tested replacement;
all affected real-world/consumer cases run through the new pipeline. Ordinary
packages have no custom runtime dependency or copied target-source fragments.
Obsolete runtime/generator and special-case infrastructure is removed, with
no hidden compatibility fallback. Full native/lint gates and independent review
pass before each checkpoint and the final cutover.

## Current audit

M35-03A-01 is complete: the inventory and real-bundle guards are reviewed and
gated. M35-03A-02A adds reviewed, native-tested built-in bool negation through
both new target paths. M35-03A-02B adds built-in lazy bool and/or with native
evaluation-order traces and scoped AST probes. M35-03A-02C adds exact i64 values,
comparisons and mixed-width signatures with native call-order and admission
controls. M35-03A-02D adds built-in i32/i64 complement and bitwise and/or/xor,
with independent native oracles, call-order mutations and typed AST probes.
M35-03A-02E adds built-in eager bool And/Or/Xor with exhaustive value/trace
controls, lazy-composition and real imported-call proofs.
Implementation parity remains incomplete: arithmetic, floating-point,
char/unit values and the other capability families remain outstanding.

Legacy C embeds runtime.c/runtime.h and always requests runtime.core. Its
portable generator remains active in CLI, conformance, benchmarks and examples.
Legacy Java constructs Runtime.java through typed AST, but still requires a
special runtime file, catalogue and renderer composition path. Typed construction
alone is not the requested architectural cleanup. The current Rust-source C/Java
paths do not create these runtime artifacts, but their feature coverage is much
narrower. No legacy implementation is deleted by this planning checkpoint.

M35-03A-02F splits scalar constants into compiler-evaluated reads (02F-01) and
proper public/local declaration support (02F-02). Completing the read step
alone does not complete scalar parity or authorize removing legacy constants.
