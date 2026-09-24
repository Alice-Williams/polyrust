# M35-03A-05A-02C — Measured C result selected-arm execution

- Status: complete
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

## Implementation and verification history

The prepared fixture is registered as its own cached native test. It certifies
the correct body and three value-preserving faults: inactive-arm evaluation,
duplicate scrutinee evaluation and selected-arm evaluation before the scrutinee.
The last uses explicit branches with branch-local initialized values. Neither
mutable I32 assignment nor calls nested inside a conditional expression is
admitted by the current C profile; the test does not widen production admission.

The corpus is 1,036 total cases: 518 payload rows across both tags (514 distinct
values; four edge rows repeat values in the contiguous range). Each
GCC14/Zig O0/O2 round checks pristine values, with measured pristine branch frames,
and a separately instrumented trace. Exact certified definition names select
observer hooks; no production runtime or observer is added. Fault runs must fail
all 1,036 trace cases rather than stopping after the first failure. The complete
matrix requires 16 value controls, four correct trace rounds and 12 fault rounds.

Source-byte bounds and exact local-frame plus imported-constructor stack cost
composition are asserted. Physical frame reports cover the pristine consumer
object; existing producer tests cover producer frames. This is not a claim of
measuring a single combined instrumented process stack.

Read-only preflight found no design blocker, but did not establish compilation
or certification. The parent subsequently caught and removed the inadmissible
I32 assignment described above. The first focused run then rejected the revised
conditional-expression variant at the full-expression call boundary, after
the other three variants completed their native rounds. The fixture now uses
ordinary branch-local declarations with direct call initializers. Focused/native/
full gates and a fresh final review of the tested snapshot remain required.

The corrected explicit-branch fixture passed all six focused targets, including
the native matrix, then the C unit suite passed (884 cases, 11 separate native
partitions). Clippy identified a large Rust test-state enum; the larger variant
now uses indirection without suppressing the lint or changing the generated AST.
The corrected immutable candidate is
`7369506167b71b14299a39c8d8fe86e0d0c7f372`; its full release/lint run passed.

An independent GPT-6-SOL Extra High reviewer, not the preflight reviewer,
examined the complete focused-tested diff and the three-line indirection change.
No required defect was found. Its optional corpus-wording clarification is
included above. The full release gate independently passes.

## Completion evidence

All 1,041 Linux dev-container `test //... //:release_gate` targets pass on the
corrected production/test tree above (two jobs, established isolated output
root, lockfile mode off). Seventeen tests executed, the remainder were cached;
elapsed time was 1,225.830 seconds. The native trace target passed in 17.7 seconds,
and the C unit suite passed 884 cases with all eleven native partitions retained.
Rust Clippy/rustfmt and Bazel lint are included; no lint or test was disabled.

The matrix executes 16 pristine value controls (16,576 cases), four correct
observed runs (4,144 cases) and 12 compiling faulty runs (12,432 cases). Every
faulty row preserves the value but fails the exact execution trace, with no
sanitizer diagnostic. Consumer frame reports and imported stack composition pass.
All 530 prior generated file hashes, four newer character-constant bundles and
45 unrelated WIP hashes are unchanged. Completion docs are gated again on the
exact final commit snapshot before push.

This closes the C scalar-result transport/public-ABI parent obligations only.
Java transport, canonical cross-crate source-instance placement, compiler
admission and wider runtime retirement remain separate work.
