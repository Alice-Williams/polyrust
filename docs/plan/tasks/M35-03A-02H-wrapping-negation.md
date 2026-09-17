# M35-03A-02H — Wrapping signed-integer negation

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [unit results](M35-03A-02G-unit-results.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-negation.md)

## Contract

Implement the built-in i32/i64 wrapping_neg operation without custom runtime
files, signed-overflow UB, width changes or new third-party dependencies.
This is one bounded arithmetic increment, not general wrapping arithmetic.

## Ordered checkpoints

1. [02H-01 — Target proof](M35-03A-02H-01-target-negation.md) — complete:
   admit typed
   negation and the safe C guard/conditional in certified source/dependency
   profiles; prove native boundaries, original authority and rejection.
2. [02H-02 — Checked compiler mapping](M35-03A-02H-02-compiler-negation.md) — complete:
   authenticate the actual built-in method, register WrappingNegation through
   the typed builder, preserve exact receiver evaluation and export examples.

## Definition of done

Both checkpoints pass their full isolated Linux Bazel release/lint/native gates
and fresh independent reviews, with separate commits/pushes. Keep legacy paths
and all previous source rejection controls enabled; unit and scalar signatures
and metadata do not change. Addition/subtraction/multiplication/division,
ordinary potentially overflowing source unary minus, casts, other widths and
user-defined lookalike methods remain outside this increment.

## Completion

The target foundation is committed as c914d2a and GitHub run 35183183665 is
fully green, including release/cache-save. The compiler implementation passes
all 778 Linux Bazel test targets, with 84 atomic source negatives, 14 new
compile contracts, 12 mapper observations per target, 52,500 native results,
and a fresh clean independent review. See the child receipts for exact trees
and invocations. Existing runtime retirement remains blocked on wider parity.
