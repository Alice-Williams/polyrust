# Checked binary64 truncation

## Semantic boundary

FloatingTruncation admits the standard inherent f64::trunc method and associated
forms from original compiler-checked HIR. A private input witness retains exact
node/typeck/callee/receiver identity. Ordinary free functions with the same name
are DirectCalls, not builtin mappings. Other widths, trait methods, adjustments,
casts, rounding operations and arbitrary standard calls remain unadmitted.

Every non-NaN result is the exact integral part rounded toward zero, preserving
the sign of zero and infinities. NaNs are observed by category only; payload,
sign, signalling behavior, fenv flags/traps and errno are outside this portable
contract. Execution retains the existing nontrapping binary64 environment and
no-fast-math policy. Do not use integer conversion to implement this operation.

## Layers

The frontend authenticates a private TruncationInput. Supports-style executable
bindings lower it to existing checked target AST nodes. Arguments are lowered
and materialized once before target selection. Certification validates the
whole package, actual dependencies and resource bounds. Renderers only render
certified structure. Standard library calls are normal typed dependencies, not
copied runtime source or special renderer cases.

- [C mapping](languages/c/rust-floating-truncation.md)
- [Java mapping](languages/java/rust-floating-truncation.md)

## Evidence and limitations

Use an integer-only binary64 oracle: below unit exponent retain the sign bit;
for exponent 0..51 clear fractional significand bits; at exponent >=52 retain
the original bits. Test NaNs by category. Native Rust/C/Java values and receiver
traces, AST probes and rejection/mutation controls are separate proof layers.
Target foundations do not by themselves admit Rust source or establish full
arithmetic parity.

Source reference: [Rust f64::trunc](https://doc.rust-lang.org/std/primitive.f64.html#method.trunc).
