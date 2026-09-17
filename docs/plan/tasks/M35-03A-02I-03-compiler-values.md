# M35-03A-02I-03 — Checked Rust f64 value mappings

- Status: complete
- Parent: [binary64 values](M35-03A-02I-binary64-values.md)
- Depends on: [target foundation](M35-03A-02I-02-target-values.md)

## Contract

Extend the existing executable ObjectTypes, LiteralValues, ResolvedPlaces,
record, signature, call and ScalarComparisons mappings with exact f64 rules.
Obtain literal values from rustc's checked evaluation, not a duplicate decimal
parser. Preserve canonical source identities, documented public/private crate
boundaries and exactly-once left-to-right calls.

Literal construction initially accepts only finite results, including negative
zero. Runtime f64 parameters may carry any binary64 category under the specified
observable value/comparison contract. Public/local f64 constants, nonfinite
construction, casts, arithmetic and methods remain diagnosed until separately
specified and proven. Version metadata explicitly if new type spellings require
it; old scalar-only package bytes/schemas remain unchanged.

## Definition of done and tests

- Real multi-crate Rust and generated C/Java packages match independent exact
  finite-bit and nonfinite-category/comparison oracles, with native call traces.
- Compiler/AST probes prove original identity, exact type, literal bits, operand
  placement and identical probe/production bytes; counterfeit witnesses fail.
- Unsupported f32, casts, arithmetic, nonfinite construction, adjustments and
  mixed widths reject atomically after valid Rust analysis where applicable.
- Actual examples are exported outside Docker without committing generated
  output. Full Linux release/lint gate and a fresh independent review pass.
- No replacement-family completion or runtime retirement is claimed.

## Implementation order

Completed prerequisite: [03A — Closed constant value domain](M35-03A-02I-03A-constant-domain.md).
Literal and constant witnesses now use distinct enums, preserving the closed
constant domain before literal support is widened.
Then extend checked compiler literal evaluation, source representations and
capability mappings, followed by metadata/native/negative/probe proof. Each
independently completed child has its own passing checkpoint; this parent
remains incomplete until the full source-equivalence contract is demonstrated.

## Source integration sequence

1. Add a finite F64 variant only to LiteralValue. Pass the compiler context
   into LiteralInput and obtain Float/F64 values through rustc's lit_to_const
   query (including the unary sign), with an explicit eight-byte check before
   decoding bits. Depend directly on the target-independent binary64 crate.
2. Extend exact object/signature/record/comparison mappings and Java TypePlan.
   Keep entry-harness ABI, constant witnesses, arithmetic and casts closed.
   Test shared-reference transport without broadening public reference ABIs.
3. Emit C manifest version 8 and Java owner manifest version 6 only when
   serialized declarations/signatures need the new f64 type spelling. Keep
   existing versions and bytes for packages without that spelling.
4. Add multi-crate fixtures, compiler-witness/AST probes and atomic rejection
   tests. Compile generated C with GCC and Zig and Java with strict javac.
   Compare Rust/native results against independently derived bit/category and
   comparison expectations, including operand-order controls.
5. Export actual examples, run the full release gate and independent review,
   then record evidence and commit/push this completed contract.

## Proof receipt

Implementation tree: `b66e5f348d3e30d919315e2608e74ca87ec9392b`.
The exact Linux dev-container Bazel release/Rust/Bazel-lint gate passed
786/786 tests across 1,193 targets (4 executed, 782 cached; 36.519 seconds),
invocation `b40253a9-c5b0-4560-8816-801042f52aac`.
The preceding full implementation gate also passed 786/786 tests, invocation
`3a7947dc-188c-4d27-8fae-1241ca33e452`.

- `binary64_native_test`: 31,548 exact results per implementation from 72
  boundary/deterministic inputs, eleven finite literals and six transport
  functions, across three independently compiled crates/packages. Original
  Rust, Java21 strict lint, and GCC14/Zig C17 at O0/O2 agree with an independent
  integer-bit oracle. NaNs compare/classify correctly without payload claims.
  Value-preserving reordered/dropped-call faults fail exact trace expectations;
  negative-zero and subnormal literal faults fail exact value expectations.
- `binary64_ast_test`: eleven canonical compiler/literal observations and six
  comparison observations per target; exact types/bits, original callable
  identities, left/right placement and copied-HIR/wrong-context rejections.
  Probe and production output files are byte-identical.
- `binary64_rejection_test`: 52 atomic rejection cases, including valid Rust
  unsupported operations and overloaded reference comparisons, plus Java's
  accepted 255-slot/rejected 256-slot f64 signatures. Existing compile-negative
  input privacy, registration and distinct constant/literal domain probes pass.
- All 138 relative-path/SHA-256 entries across 18 existing C/Java bundles
  match the preceding commit `bdad2193d2a607b2f01d0f66ae442e930a5d2191`
  byte-for-byte. The exact prior Git archive was rebuilt through Bazel at the
  same location, invocation `77d7dc28-1794-4740-96bd-615d2fdecc3b`.
  This covers scalar/local/public/imported/aliased constants, i64, unit and
  wrapping packages. New f64 metadata uses C8/Java6; old schemas remain intact.
- Actual uninstrumented packages, handwritten consumers and Rust inputs are
  exported by the native test and copied to the host under
  `generated/examples/binary64-values-caadbfd5/` (24 files). They are ignored
  generated artifacts, not committed sources or promised future examples.

An independent Sol Extra High review examined the complete implementation and
the exact final proof delta and found no remaining core or required-test issues.
Four older probe macros initially lacked the new direct binary64 dependency;
these were repaired, not disabled, before the passing full gates. Pending-status
wording is closed in this documentation checkpoint. The parity inventory marks
only partial JavaF64Values coverage and corrects its stale description of
already-completed public/import/alias constant support; constant admission is
not widened here.

General floating arithmetic/constants/casts/methods, remaining scalar families,
and the broader runtime retirement work remain incomplete.
