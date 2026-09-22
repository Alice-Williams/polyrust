# M35-03A-02V-02 — C signed-infinity constant foundation

- Status: complete
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [oracle](M35-03A-02V-01-constant-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-infinite-f64-constants.md)

## Contract

Register the exact typed HUGE_VAL standard constant and signed-negation form.
Separate constant inventory values from finite literal nodes. Extend only
matching F64 constant certification/import facts; keep renderer structural.

## Definition of done and tests

Native owned/imported/aliased objects and generated readers agree on both exact
infinity signs with GCC14/Zig O0/O2 and UBSan. Standalone headers, inferred
math.h and no spurious libm dependency are proven. Wrong sign/type/shape/owner,
resource and forged dependency controls reject. Finite output hashes remain
unchanged. Full gate and clean independent review precede commit/push; no
compiler source or Java admission changes in this checkpoint.

## Implementation and review evidence

The closed CScalarConstantValue inventory carries exact signed infinity without
widening CLiteral or FiniteBinary64. Standard identity, type, arithmetic-constant
category, header and spelling are catalogued; only HUGE_VAL and its exact typed
negation enter the constant inventory. Existing compiler joins explicitly
project back to finite literals until 02V-04.

Focused C unit/native tests passed on tree 9839a7f: native proof took 31.9 seconds
and the unit target 93.6 seconds. The native proof separately compiles producer,
alias facade and generated readers with GCC14/Zig O0/O2 and GCC UBSan, checks
standalone headers without libm, and rejects compiling sign-loss, finite-clamp
and zero substitutions against the independent integer-only oracle.

The full gate exposed two test-integration fixes: an exact-path handwritten
oracle exemption (with adjacent-path rejection tests) and an old frontend
probe comparison that needed the finite-literal projection. Production
admission was not relaxed. Corrected tree 22d28e213c362788153f4bd31d23cc842cc97c7a
passes all 1,005 Linux Bazel release/lint targets (19 executed, 986 cached),
invocation f38081ca-7e76-4105-a4c1-404a20287e0f. This includes the native infinity
proof, the 24,576-value finite regression, compile-negative certificates,
source-policy injection tests and Rust/Bazel linters.

Independent Sol Extra High broad review covered tree dfcab545 and the narrow
follow-up through d8bb98d, with no remaining core correctness findings. The
reviewer's optional IEC-profile sentinel is deferred: this checkpoint supports
the pinned Linux LP64 GCC14/Zig profile, whose exact infinity behavior is tested,
not arbitrary C17 implementations satisfying only finite representation checks.
Expanding that profile requires its own infinity-support evidence.

All 462 previous source-owned output hashes and 38 unrelated WIP hashes remain
unchanged. Java and Rust-source infinity admission are separate later gates;
NaN constant admission and wider runtime parity remain outside this checkpoint.
