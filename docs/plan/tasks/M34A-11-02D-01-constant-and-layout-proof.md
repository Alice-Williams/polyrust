# M34A-11-02D-01 — Exact C constants and object layouts

- Status: planned
- Depends on: M34A-11-02D-00

## Goal

Replace constant-expression category alone with exact checked values and layout
facts, using the frozen Linux x86_64 ABI rather than host arithmetic assumptions.

## Definition of done

- Implement private checked scalar values, integer promotions/conversions and
  layout size/alignment/padding calculations from authoritative registered types.
- Traverse typedefs and complete aggregates; reject recursive by-value layouts,
  incomplete SizeOf/AlignOf targets and checked size/alignment/array overflow.
- Evaluate all admitted integer constant operators with target-width semantics;
  reject signed overflow, division/remainder zero and minimum/-1, invalid shifts
  and unproved narrowing. Unsigned wrapping follows its actual promoted type.
- Re-evaluate static initializers and assertions from actual AST children;
  require assertion truth. Respect conditional evaluation without skipping
  structural checks of unselected syntax.
- Treat opaque FILE distinctly from complete MaxAlign. Values, identities and
  errors are typed enums/wrappers, never integer proof labels or safe flags.
- Expose only crate-private immutable facts for later analysis/resources.
  Compiler resource budgets and native frame accounting remain stage 04.

## Tests and proof

- Every scalar width/rank, signed extrema, unsigned wrap and usual-conversion
  pair; zero divisors, minimum/-1, shift boundaries and narrowing mutations.
- SizeOf/AlignOf for scalars/pointers/enums/aliases/arrays/structs/unions, padding,
  mutual cycles, incomplete objects, huge checked products and alignment sums.
- True/false assertions, invalid constant children and skipped-invalid-operation
  controls paired with unconditionally invalid structural-child rejection.
- Independent pinned native ABI/constant controls, focused Rust/compile-fail/lint,
  full cached tracked/release/eight-target gates.

## Commit gate

Record exact passing evidence, then commit/push M34A-11-02D-01. A constant or
layout fact is not a whole-program safety or rendering certificate.
