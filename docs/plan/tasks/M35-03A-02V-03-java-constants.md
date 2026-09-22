# M35-03A-02V-03 — Java signed-infinity constant foundation

- Status: complete
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [C foundation](M35-03A-02V-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-infinite-f64-constants.md)

## Contract

Register typed Double infinity fields and exact primitive-double constant
inventory values. Preserve declaration/producer authority and structural
rendering. Do not generalize arbitrary initializer expressions or finite literals.

## Definition of done and tests

Native strict separate Java21 compilation observes both signs from owned,
imported and aliased fields/readers under normal/-Xint execution. Compiling
faults are detected after recompiling dependents. Incorrect shape/type,
lookalike producer, resource and byte-bound checks reject. Both valid signs
are retained exactly; checking an expected Rust sign belongs to source integration.
All old output
hashes remain unchanged. Full gate and fresh independent review pass before
separate commit/push. Compiler source admission remains disabled.

## Implementation sequence

1. Add DoublePositiveInfinity/DoubleNegativeInfinity to JavaKnownField and
   PositiveInfinity/NegativeInfinity to JavaMemberName. Reuse JavaKnownType::Double
   as owner and primitive JavaPrimitive::Double as field type. Existing typed
   name resolution and structural field rendering remain authoritative.
2. Add a focused JavaScalarConstantValue model for Boolean, I32, I64, finite
   F64 and Infinity(Binary64Sign). Its finite-literal projection returns None
   for infinity. Reconstruct exact values from certified literal/known-field
   syntax; do not evaluate arbitrary expressions to manufacture an inventory.
3. Update dependency inventory and opaque constant witnesses, retaining every
   source registration, public/static/final, owner, exact type and certificate
   check. Admit only the two exact standard fields in the source-shaped body
   profile. Update source-byte accounting for resolved owner, dot and member
   spelling, including qualified names selected to avoid collisions.
4. Adapt finite-only compiler joins, java_bundle projection/constant manifest
   serialization and AST probes explicitly. They must reject an absent finite
   projection until 02V-04; source_capabilities constant admission is unchanged.
5. Add focused fixture, inventory/rejection and native modules. Reuse registered
   source constant producers, constant_exports_fixture alias metadata and
   authenticated consumer readers. Prove both signs, qualified owner reservation
   and conservative rejection of value-shadowed owners,
   wrong field/type/precedence/owner/modifiers, exact bounds and overflow limits.
6. Compile actual producers, facade packages and consumers separately with
   Java21 strict warnings. Compare raw bits against the independent infinity
   oracle under normal/-Xint execution. Recompile all dependents for each
   compiling sign-loss/finite-clamp/zero mutation, and require detection.
7. Preserve all prior output and unrelated WIP hashes; run the full Linux Bazel
   release/lint gate and independent broad Sol Extra High review. Resolve core
   findings, record optional findings explicitly, then commit/push this checkpoint.

Keep implementation helpers and proof fixtures in small responsibility-specific
files. No generic support flag, copied runtime, raw target expression, NaN
constant admission or additional source capability belongs in this foundation.

## Implementation and review evidence

The JavaScalarConstantValue inventory distinguishes finite literals from
Infinity(Binary64Sign). Both standard fields have primitive Double type and
authoritative catalogue entries. Exact initializer recognition, descriptions,
original producer/alias authority, standard-owner dependencies and source-byte
reservation are typed; rendering reuses the existing structural field renderer.
Finite-only rustc joins and all three manifest paths reject infinity through
the explicit literal projection until 02V-04.

The corrected candidate d127a40e4bc05a60b2b0c6a0955fd36bc2042202 passes
all 1,006 Linux Bazel release/lint targets (116 executed, 890 cached), invocation
67467bec-5e15-4502-852c-2d355744576e. The main Java unit target passes 423 tests;
the separate infinity native target passes in 41.24 seconds, and the retained
24,576-value finite native corpus passes in 232.50 seconds. The new backend
cases and manifest boundary case cover exact values, registered owner/type/
modifier/precedence rejection, alias authority, direct known-field method
bodies, conservative value-shadowing rejection, qualified owner reservation,
missing names, depth limits and byte exhaustion.

Native producer, alias facade, readers and external client compile separately
with Java21 strict warnings. Normal and interpreted runs observe exact bits.
Sign-loss, finite-clamp and zero mutants compile; all dependents are recompiled,
and measured output both disagrees with truth and matches the independent
integer-only fault oracle. Reflection checks the alias facade has no copied
fields, wrapper methods or public constructor.

Independent Sol Extra High broad review of that immutable candidate found no
core correctness defects. Two optional hardening suggestions are deferred:

- A catalogue-wide field exhaustiveness invariant is broader than this bounded
  foundation. Both new entries are registered and exercised by symbol admission
  and native generation; the suggestion would protect future catalogue edits.
- Additional whole-bundle infinity rejection matrices would duplicate the shared
  certified_value helper's direct zero-output rejection tests. All three
  manifest paths call it, and the unchanged source-level infinity/NaN rejection
  suite proves atomic publication. Extra bundle-level redundancy is useful
  future hardening, not an uncovered admission path in this candidate.

All 462 prior generated-output hashes and all 38 unrelated WIP hashes are
unchanged. No generated output is committed, no custom runtime is introduced
and no legacy path is deleted. Checked Rust-source infinity integration remains
the separately gated next checkpoint.
