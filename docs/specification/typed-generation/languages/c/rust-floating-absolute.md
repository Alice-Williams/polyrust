# Rust f64::abs in C17

- Status: normative bounded design; implementation pending
- Contract: [shared](../../rust-floating-absolute.md)

## Certified target foundation

CExpressions::conditional already constructs typed CValueKind::Conditional.
Extend the shared package profile with one exact branch: Bool condition and
F64 then/else/result with identical types. Do not widen existing integer
promotion rules, admit mixed types or disable ownership/sequencing checks.
Scalar-call shape traversal must continue traversing every child and preserve
original imported identities. Existing renderer parentheses and double literal
formatting remain authoritative.

## Checked source mapping

CFloatingAbsolute validates the private compiler witness, lowers the original
receiver once and materializes one exact F64 local. Construct positive zero
with the checked FiniteBinary64 witness. Equal and Less comparisons against
that zero have Int results; normalize each to Bool. Build the inner negative
conditional and outer zero conditional with exact F64 branches. Unary Negate
is the existing admitted exact F64 node. Use no casts, bit conversion, math.h
helper or runtime file. Only pure local reads occur inside these branches.

## Proof

Separate original-owner producer/consumer C units must compile under pinned
GCC14 and Zig, C17 strict warnings, O0/O2, no-fast-math and the existing
nontrapping environment. Check exact non-NaN magnitude bits, positive zero for
both zero inputs, NaN category, original receiver evaluation once and rejected
malformed conditional types. Test mutations must establish zero/sign/dataflow
observability. Full release/lint and independent review are required.
