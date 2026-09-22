# M35-03A-02S-03 — Java wrapping-multiplication foundation

- Status: complete
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [C foundation](M35-03A-02S-02-c-multiplication.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-multiplication.md)

## Contract

Extend dependency-body certification for exact primitive Int/Long Multiply with
Multiplicative precedence and equal operand/result types. No boxing, promotion,
Math.multiplyExact or custom helper. Preserve all child/dependency/budget checks.

## Definition of done and tests

Both widths certify and compile through separately built Java21 producer,
forwarder and external client. Oracle agreement and compiling wrong-operation,
saturation, narrowing and disconnected-operand controls pass. Mixed widths,
boxed/String/Boolean values, wrong result/precedence and unrelated operators
reject. Existing depth, call-height, arity, source-byte and both-child authority
gates stay active. Preserve existing bytes, full gate and clean independent
review; no compiler-source admission yet.

## Implementation and focused evidence

Dependency-body certification now admits exact primitive Int/Long Multiply,
with Multiplicative precedence, equal operand/result types and recursive child
checking. Existing Add/Subtract and Double rules, import authority, arity,
call-height, depth and source-byte budgets remain in place. The structural
renderer is unchanged. Shared test fixtures name multiplication methods distinctly
and choose the correct operator precedence. Older Add/Subtract rejection lists
no longer reject now-supported Multiply; their other negative shapes stay active.

All 22 focused wrapping tests pass on tree 32d3262d1367e61405600162dfde80bb992b6e9a,
900aaf0f-879f-48f2-9fdf-bbfba89956b1. New tests cover all precedence variants,
wrong/boxed/String/Boolean types, both operand imports/arity, unsupported nested
operators/casts, depth, original dependencies and bounds. Strict separately built
Java21 producer, forwarder and external client agree on 69,092 observations per
run against the 34,546-case oracle. Five compiling faults (saturation, addition,
narrowed result, narrowed operands and disconnected right operand) are detected
at both widths. The narrowed-operands control explicitly widens each narrowed
operand back to the intended arithmetic width before multiplying.

Full regression passes all 948 release/lint targets (108 executed, 840 cached),
4c4a867a-82e7-425b-89b8-e158f5db505a. All 405 Java unit tests pass. All 389
previous generated files and all 38 unrelated WIP hashes remain unchanged.

Independent whole-scope Sol Extra High review found no core correctness or
required proof-gap issue. It checked both-child recursion, exact primitive widths
and precedence, all budget/import/arity guards, separate native compilation and
all five faults at both widths. Its sole editorial finding, a misspelled module
comment, is corrected. No production change was needed after review.

The comment/documentation closure is gated once more before the separate tested
commit/push. Checked compiler-source admission follows in 02S-04; this checkpoint
does not change source admission, dependencies or legacy/runtime paths.
