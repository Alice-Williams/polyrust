# M35-03A-02I-02 — Certified C/Java binary64 target values

- Status: complete
- Parent: [binary64 values](M35-03A-02I-binary64-values.md)
- Depends on: [finite witness](M35-03A-02I-01-finite-literals.md)
- Specifications: [C](../../specification/typed-generation/languages/c/rust-binary64-values.md),
  [Java](../../specification/typed-generation/languages/java/rust-binary64-values.md)

## Contract

Use FiniteBinary64 in actual target literal enums. Emit finite hexadecimal
floating syntax from typed sign/significand/exponent data, without decimal
rounding or helper calls. Extend only primitive value/field/signature transport
and six comparison operators, including original-owner dependency admission.
Explicitly restrict integer-only operators after widening any shared scalar
predicate; float arithmetic/conversion must not leak through a broad match.

## Definition of done and tests

- Both signs, signed zeros, subnormals, boundary exponents and finite random
  payloads compile with exactly matching bits in GCC/Zig O0/O2 and Java 21
  strict lint. Public producers and consumers compile separately.
- Native inputs include infinities and NaNs; comparison results use independent
  category/order expectations. NaN payload identity is not claimed.
- C binary64 platform assertions and pinned execution assumptions are explicit;
  wrong/missing assertions reject before certification.
- Primitive double remains exact in signatures, fields, locals, conditions,
  source bounds and original dependency witnesses. Wrong widths/operators,
  narrowing, invalid authorities and insufficient bounds reject.
- Deliberate sign/exponent/subnormal corruption and dropped/reordered operand
  calls are caught by bit/trace oracles. No numeric epsilon is used.
- Full Linux release/lint gate and fresh independent review pass before push.
  Rust-source admission stays unchanged in this checkpoint.

## Integration prerequisite

Inspection found that the shared linker supports named symbol imports and
generated-file imports, but not an unnamed standard-library requirement.
[M35-03A-02I-02A](M35-03A-02I-02A-library-imports.md) adds that independently
reviewed boundary first. The remaining C/Java target implementation stays in
progress until every parent exit criterion has its own evidence.

## Proof receipt

The target-only corpus contains 272 finite bit patterns (both signs, both zeros,
subnormal/normal/exponent boundaries and deterministic random payloads), plus
infinity and quiet/signaling NaN categories for all six comparisons. Generated
producers, consumers and record transport compile separately under GCC 14 and
Zig at O0/O2 with strict C17 warnings and floating options, and Java 21 with
strict lint. Exact raw-bit checks catch sign/exponent/subnormal corruption.
Instrumented, separately compiled calls catch dropped and reordered operands
even when their return values remain identical. No epsilon comparison or
generated bit-conversion runtime is used.

Focused tests passed seven C and eight Java cases. Full-suite verification
also covers existing authority, budgets, compiler-negative and capacity tests.
The 127-double/128-double Java boundary proves two-slot argument accounting.
Each of the eight C layout/property assertions has removal and wrong-value
rejection controls. Unsupported float arithmetic/casts and mismatched widths
remain rejected.

### Review disposition

The first independent Sol Extra High review found three release-gate blockers:
directive spelling outside the certified renderer interface, a handwritten
consumer helper without an explicit test-only annotation, and a stale Java
budget rejection assertion. All three were accepted and corrected; neither
the source policy nor the negative unsupported-type test was removed.

An optional observation concerned existing internal initialized C pointer
aliases now carrying F64. No new fence is added: this is exact primitive
transport through existing checked ownership/address/dereference rules, not
pointer reinterpretation, public pointer signatures, pointer fields or a new
mutation capability. The Rust-source integration must still prove whichever
shared-borrow source forms it admits; these target fixtures are not source
equivalence evidence.

A second independent Sol Extra High reviewer examined the complete corrected
implementation tree `b0907000212b57f3991371d12bb1f905ae9fcdc5` against
`4c966222a58b956c67805be7922dbf08db141b20` and found no core correctness,
build-structure or proof-adequacy errors. The first reviewer's conclusions were
not supplied as evidence to that reviewer.

The second reviewer suggested adding DoubleHasSubnormals to older "every known
constant" table fixtures. This is retained as non-blocking test hygiene, not
missing capability proof: the new platform tests install, type-check, project,
render and natively compile that exact constant, and reject removal or changed
expectations. No existing table assertion is suppressed or weakened.

The exact corrected tree passed all 780/780 Linux dev-container Bazel tests
across 1,176 targets, including the release gate and Rust/Bazel linters
(111 executed, 669 cached; 766.345 seconds). Invocation:
`f7044b38-8306-415a-867c-33550b2454af`.
The C unit suite passed 784 tests (five capacity cases run as separate targets),
and Java passed 364. The source-policy check passed with no policy exemptions.

Rust-source f64 admission, multi-crate source equivalence and exported source
examples remain the next checkpoint, M35-03A-02I-03. This target checkpoint does
not claim floating arithmetic, complete scalar parity or runtime retirement.
