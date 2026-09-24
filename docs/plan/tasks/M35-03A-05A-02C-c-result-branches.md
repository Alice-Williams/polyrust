# M35-03A-05A-02C — Measured C result selected-arm execution

- Status: planned
- Parent: [C result transport](M35-03A-05A-02-c-results.md)
- Depends on: [nominal imports](M35-03A-05A-02B-02-c-result-imports.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

Close the parent contract's explicit inactive-arm effect proof before marking
C transport complete or starting Java transport. Existing result tests prove
tag/payload/fallback values and closed call/resource safety, but their pure
read/constant alternatives cannot detect an eagerly evaluated inactive call.
Do not weaken or silently move that obligation to compiler source integration.

Build a focused typed C fixture using certified result imports, materialize its
scrutinee exactly once, and place distinguishable helper calls in success/error
arms. Generate both correct and deliberately eager variants through the same
ordinary typed AST/certification path; keep values identical so trace evidence
is necessary. No new source capability or runtime artifact is required.

Observe actual native calls with strictly test-only instrumentation. Instrument
the rendered function definitions using exact certified identities and unique
match checks, or use equivalent compiler instrumentation with a reliable native
oracle. Do not simulate execution in the test or manufacture expected traces
from the lowered AST. Keep an uninstrumented native value control as well.
Test scaffolding may maintain observer state; production generated code must
remain ordinary runtime-free library code.

## Definition of done and tests

- Correct code observes the scrutinee once and only the selected helper,
  retaining order for success/error, zero, signed extrema and representative
  payloads across independently compiled producer/consumer files.
- Compiling eager-arm, duplicated-scrutinee and reversed/wrong-arm mutations
  preserve the ordinary values where applicable but fail the native trace oracle.
  Count observations and mutation executions explicitly; failures cannot depend
  on compilation errors, undefined behavior or sanitizer reports.
- GCC14/Zig O0/O2 and sanitizer controls pass, together with normal value
  transport, exact typed dependencies and conservative frame/resource checks.
- Add a separately cached native Bazel target if appropriate; retain all cases
  in the full release gate. Preserve prior outputs and unrelated WIP, update
  evidence, obtain fresh GPT-6-SOL review, and pass full Linux release/lint before
  a separate milestone commit/push.

This is target execution evidence, not admission of Rust match syntax or proof
of source Result-instance identity. Those remain in 05A-04.
