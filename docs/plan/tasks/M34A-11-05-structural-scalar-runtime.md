# M34A-11-05 — Build structural scalar, numeric and UTF-8 runtime

- Status: planned
- Depends on: M34A-11-04

## Goal

Implement required C runtime algorithms as certified AST with explicit safe effects.

## Definition of done

- Add typed scalar/unit/portable-error declarations and exact known-callable contracts.
- Implement checked/wrapping arithmetic, shifts/conversions, F64 raw bits/inspection/arithmetic and Unicode scalar/UTF-8 primitives.
- Generate safety guards and sequence plans structurally, with verified arithmetic/bounds facts rather than opaque trusted labels.
- Add allocator/transport-status, immutable text/byte factories, private storage, borrow/clone/drop and cleanup foundations.
- Keep portable Result failures distinct from allocation/input transport failures; no errno, abort or semantic macros.

## Tests and proof

- Boundary matrices against evaluator: integer minima/maxima/overflow/zero divisors/shifts; float bit payloads/signed zero/NaN behavior; Unicode/embedded zero/invalid UTF-8.
- Native certified runtime consumers and exact helper include closure.
- Fail each allocation and check outstanding allocation count, output emptiness and allocator identity; GCC ASan/UBSan.
- Runtime-plan mutations, source policy, C/compiler/sanitizer and complete cached gates.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-05 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
