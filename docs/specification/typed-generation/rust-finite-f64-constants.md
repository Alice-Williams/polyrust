# Checked Rust finite binary64 constants

- Status: complete through checked C/Java source admission
- Plan: [02U](../../plan/tasks/M35-03A-02U-finite-f64-constants.md)
- Targets: [C17](languages/c/rust-finite-f64-constants.md), [Java21](languages/java/rust-finite-f64-constants.md)

## Source boundary

Extend the existing nongeneric module/inherent constant reads, block-local
declarations, public declarations, imported reads and authenticated value aliases
to compiler-evaluated finite f64. Preserve the existing definition/owner/context
and source-export guards. Type-alias uses, borrowed storage, generic/trait
constants, f32 and nonfinite constant values remain rejected.

Rust compiler evaluation may consume a constant expression that is not an
admitted runtime expression. This does not authorize translating that runtime
expression: the constant mapping receives the checked compiler value, not a
reimplementation of evaluation. Never silently normalize negative zero.

## Typed representation and lowering

Add a distinct finite-value variant to ScalarConstantValue containing the
existing FiniteBinary64 witness. Decode exactly eight compiler scalar bytes
only after confirming f64 type and size; validate finiteness through that
witness. Use representation equality for provenance/value matching, not f64
numeric equality (which conflates the two zeros). LiteralInput and ConstantInput
remain distinct types. Existing executable constant mappings must exhaustively
handle the new variant; no new support-only marker or raw target fragment.

Public constants remain normal source-owned declarations. Imported constants
must match original declaration, owner certificate, type and exact bits.
Re-export aliases do not clone storage or manufacture independent authority.
No generated runtime, custom helper library or third-party dependency.

Descriptive C/Java manifests encode finite constant values with scalar `f64`
and a string containing `0x` followed by exactly sixteen lowercase hexadecimal
representation digits. This preserves both zero signs without JSON-number
rounding. Existing Boolean/integer encodings are unchanged. These descriptions
do not create or replace opaque original producer certificates.

Any serialized f64 declaration, import or export selects the binary64 metadata
schema: C version 8 (or version 9 when system linkage is present) and Java
version 6. This includes constant-only producers and alias-only owners without
functions. Older Boolean/integer-only packages retain their existing versions.

## Proof

Use independent integer bit-pattern expectations and native Rust constant
observations. Cover both zeros, subnormal/normal boundaries, extremes,
representative exponent/significand patterns, decimal rounding boundaries and
computed finite constants. Native C/Java observations recover bits in test
clients only, not through a new translated bit-conversion capability.

Challenge zero-sign loss, premature f32 rounding and wrong-value substitutions
with compiling controls. Mutation tests must distinguish original producer
authority even when numeric values agree. Prove original APIs/docs/privacy,
atomic rejection and actual Bazel invalidation after producer-value changes.
Export actual generated packages at source integration. Keep wider constant
parity and all other migration families explicitly incomplete.
