# M35-03A-02U-02 — C finite-constant foundation

- Status: complete
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [oracle](M35-03A-02U-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-finite-f64-constants.md)

## Contract

Extend the certified shared constant profile and dependency inventory to exact
const F64 storage with finite F64 literal initialization. Retain original
producer authority, declaration identity and existing platform/resource proof.
Do not admit source constants in this checkpoint.

## Definition of done and tests

Public certificate tests reject mismatched scalar types, nonliteral initializers,
wrong linkage/owners and forged/lookalike imports. Real rendered producer,
forwarder and external-consumer files pass strict GCC14/Zig O0/O2, standalone
header and UBSan checks, matching the independent bit oracle. Compiling
zero-sign, f32-rounding and wrong-value controls fail the oracle. Resource bounds
and old integer/Boolean constants remain covered. Full gate/review pass,
old output/WIP match, then commit/push this foundation separately.

## Implementation and proof scope

The shared object profile and certificate-derived dependency inventory admit
only const F64 objects initialized with the existing finite F64 witness. Imported
numeric facts use the same typed number decoding as literal expressions; the
integer-only decoding helper remains private to integer handling. No renderer,
runtime or compiler-source admission is changed.

Public certificate tests cover exact bits/types, both zero signs, original
producer identities through alias facades, actual imported readers, source and
frame bounds, and matching declarations with different value certificates.
Constructor/catalogue/profile negatives cover wrong scalar types, linkage,
files, foreign registrations, owner replacement and nonliteral initializers.
Mutable F64 storage remains rejected. An oversized documentation attachment
on one finite object must fail actual certification with TargetResourceLimit.

The native proof uses all 24,576 oracle observations as actual certified const
objects, split into 384-object packages below the unchanged resource limits.
Separately compiled producer/external-consumer files run under GCC14 and Zig
at O0/O2, plus GCC UBSan, with standalone public-header checks. Eight additional
boundary values flow through certified alias facades and generated imported-reader functions;
three compiling mutations alter original stored constants, not the observer.
Premature f32 rounding is substituted using integer-derived literal bits, with
test-only infinity for overflow, avoiding undefined out-of-range C float casts.

The native corpus has an independently cached Bazel execution partition. The
partition contract proves every Rust test remains selected exactly once in the
complete suite.

## Completion evidence

All 992 Linux Bazel release/lint targets pass (123 executed, 869 cached),
invocation `65d525dd-5d3b-4eff-aa6d-843d6f617a66`. The independently cached
native corpus target passes in 415.26 seconds; the complete partition contract
and remaining unit, capacity, compiler-adapter, source-policy and lint tests pass.
Fresh independent Sol Extra High review of implementation tree
`17389c70b1e50cf948cc05ff26db15d09738f587` against `df20a14` found no
actionable defects across all 17 scoped files. Final documentation closure is
gated once more before commit. All 423 prior generated file hashes and 38
unrelated ownership/conditional-source WIP hashes remain unchanged.

Development probes caught and corrected an integer-only imported-number fact,
an oversized initial corpus batch, and a test-client name-prefix assumption.
The final native proof retains the full bit corpus and existing resource limits.
This completes the C target foundation only; Java and checked compiler-source
admission remain separate unfinished checkpoints.
