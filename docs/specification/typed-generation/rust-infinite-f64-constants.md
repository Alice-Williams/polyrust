# Checked Rust signed-infinity constants

- Status: independent oracle complete; target foundations and source admission pending
- Plan: [02V](../../plan/tasks/M35-03A-02V-infinite-f64-constants.md)
- Targets: [C17](languages/c/rust-infinite-f64-constants.md), [Java21](languages/java/rust-infinite-f64-constants.md)

## Semantic boundary

Extend the distinct compiler constant domain to the two exact binary64
infinities, with bits 0x7ff0000000000000 and 0xfff0000000000000. Rust exposes
these as [f64::INFINITY and NEG_INFINITY](https://doc.rust-lang.org/std/primitive.f64.html).
Compiler-evaluated expressions producing either value have the same contract.
NaN constants, f32, type aliases, borrowed constant storage and generic/trait
constants remain unsupported. This does not add new runtime operations.

Preserve the existing FiniteBinary64 invariant. Add an explicit Infinity
variant carrying Binary64Sign to the compiler constant enum; do not widen the
finite literal witness or use a raw f64 payload. Confirm the original rustc
definition/context, exact f64 type and eight-byte ScalarInt before classifying
bits. Only a zero fraction with an all-ones exponent is infinity; every nonzero
fraction at that exponent remains a rejected NaN.

## Typed lowering and publication

Keep literal and constant inputs distinct. All existing read, local/public
declaration, import and alias mappings must exhaustively consume the extended
constant enum. Lower through the target's typed standard-constant nodes, not
raw source strings, copied helpers, division-by-zero tricks or reparsed text.
Constant inventories must retain exact type, sign, original owner and defining
declaration. Introduce closed target constant-value enums to represent literal
and signed-infinity values; do not pretend a standard field/macro is a finite
literal. Recognition is structural and tied to the known-symbol catalogue.

Owned declarations remain ordinary source-owned C objects / Java fields.
Alias-only owners retain original producer certificates and do not duplicate
storage. Descriptive manifests keep scalar f64 and sixteen hexadecimal bit
digits, with the same C8/Java6 binary64 schemas (C9 for actual math linkage).
Including a standard header alone must not invent a system-link requirement.
Existing finite/Boolean/integer outputs remain byte-identical.

## Proof contract

An independent integer oracle classifies exact bit patterns. Native Rust const
evaluation at O0/checks-on and O2/checks-off must match exact signed bits for
named constants, negated constants, overflow and signed division by zero.
NaN sign/payload patterns and the existing finite corpus challenge admission.
Actual compiling faults must expose sign loss, finite clamping and zero
replacement. Classification controls distinguish infinities from NaNs.

Target foundations are separately gated/reviewed before compiler admission.
Native C/Java external consumers read original declarations and generated
readers, with alias-only/mixed packages, strict separate compilation, C O0/O2
and UBSan, Java normal/interpreted runs and dependency recompilation after
mutations. Prove typed authority, resource accounting, dynamic imports,
API/docs/privacy, atomic failures and actual Bazel sign-change invalidation.
Export actual packages. Wider scalar parity and NaN bit semantics remain open.
