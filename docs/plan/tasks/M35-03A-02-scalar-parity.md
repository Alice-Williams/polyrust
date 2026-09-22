# M35-03A-02 — Scalar, constant and numeric feature parity

- Status: in-progress
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-03A-01

## Contract

The first bounded implementation,
[M35-03A-02A — Boolean negation](M35-03A-02A-boolean-negation.md), is complete.
Other scalar operations remain separate work; this does not claim full parity.
[M35-03A-02B — short-circuit Boolean expressions](M35-03A-02B-short-circuit-booleans.md)
is also complete, including native operand-call traces and typed AST probes.
[M35-03A-02C — exact i64 values and comparisons](M35-03A-02C-i64-values.md)
is complete, including mixed-width signatures, native call-order controls and
target-AST admission tests. Other scalar families below remain outstanding.

[M35-03A-02D — integer bitwise operations](M35-03A-02D-integer-bitwise.md)
is complete, including native value/trace controls and direct mapper AST probes.

[M35-03A-02E — eager Boolean operators](M35-03A-02E-eager-booleans.md)
is complete, with exhaustive truth tables, native eager/lazy evaluation controls
and exact typed mapper probes. Other scalar families remain outstanding.

[M35-03A-02F — scalar constants](M35-03A-02F-scalar-constants.md) is complete
for bool/i32/i64 reads, local/public declarations, authenticated imports and
aliases, with native/atomic/cache/review evidence. Wider families remain open.

[M35-03A-02G — unit function results](M35-03A-02G-unit-results.md) is complete:
typed void foundation and checked compiler integration, with 760/760 release
tests, original-authority/native/AST proof and clean independent review.
Unit storage/parameters remain a separate capability.

[M35-03A-02H — wrapping signed negation](M35-03A-02H-wrapping-negation.md)
is complete: certified C/Java target foundation and identity-checked source
capability, with 778/778 release tests and clean fresh compiler review.
Method/associated i32/i64 wrapping_neg retain exact receiver evaluation and
original dependencies. Other arithmetic remains separate work.

[M35-03A-02I — binary64 values](M35-03A-02I-binary64-values.md) is complete:
a dependency-free finite literal witness, C/Java target proof and checked
compiler value/comparison integration, with 786/786 release tests and clean
independent review. Floating arithmetic/constants/casts remain separate work.

[02J — built-in binary64 negation](M35-03A-02J-floating-negation.md) is complete:
primitive target admission and canonical compiler integration, 804/804 release
tests, exact value/dataflow/atomic proof and clean independent reviews.
[02K — binary64 NaN classification](M35-03A-02K-floating-nan.md) is complete:
standard inherent method/associated forms, original receiver and owner proof,
822/822 release tests, exported examples and clean independent reviews.
[02L — binary64 absolute value](M35-03A-02L-floating-absolute.md) is complete:
certified F64 conditionals and checked compiler integration, 840/840 release
tests, exact magnitude/NaN-category/ordinary-call proof and actual examples. Other floating inspection
and arithmetic require separate bounded contracts.

[02M — binary64 truncation](M35-03A-02M-floating-truncation.md) is complete:
C/Java target foundations and checked Rust source integration, certificate-derived
C system-library manifests, 858 release/lint tests and clean independent reviews.
Native value/category/trace controls and actual three-crate examples are included.
Other floating inspection and binary arithmetic remain separate work.

[02N — binary64 arithmetic](M35-03A-02N-floating-arithmetic.md) now has its
independent rational oracle, separate C/Java target foundations and checked
source capability integration complete. All 879 release/lint tests pass and
two independent final reviews are clean. Remainder and other scalar families
remain separate.

[02O — binary64 truncating remainder](M35-03A-02O-floating-remainder.md)
is complete through its own oracle, C/Java target and compiler checkpoints.
All 900 release/lint targets pass; native Rust traces, exact imports/docs and
external privacy controls accompany the checked source proof. Other scalar
families remain open.

[02P — negative-zero composition](M35-03A-02P-negative-zero-composition.md)
is complete: the existing Apache-2.0 real-world predicate is expressed as
ordinary Rust, using existing comparison, lazy conjunction, calls and division.
Its dedicated C/Java source proof is additive to the existing eight-language
corpus until the later cutover; it does not widen frontend admission. All 903
release/lint targets pass and the independent review is clean.

[02Q — wrapping signed addition](M35-03A-02Q-wrapping-addition.md) is complete:
independent modular truth, certified C/Java foundations and checked primitive
source integration. All 924 release/lint targets pass; measured source/target
operand traces, exact typed dataflow, atomic controls and exported multi-crate
examples accompany two clean whole-scope source reviews. Other integer operations
remain separate capabilities, and the legacy runtime is not retired by this step.

[02R — wrapping signed subtraction](M35-03A-02R-wrapping-subtraction.md) is
complete through independently gated/reviewed oracle, C/Java foundations and
checked compiler integration. All 945 release/lint targets pass. Native modular
truth, measured ordered operands, seven compiling faults, typed dataflow,
atomic rejection and original API/docs/privacy proof accompany exported packages.

[02S — wrapping signed multiplication](M35-03A-02S-wrapping-multiplication.md)
is complete through its independently gated oracle, C/Java foundations and
checked source integration. All 966 release/lint targets pass; clean independent
review, 34,546 Rust/oracle cases, measured operand traces, six compiling faults,
typed dataflow, atomic/API/privacy proof and actual exported packages accompany
the increment. Wider scalar families remain open.

[02T — lossless signed widening](M35-03A-02T-signed-widening.md) is complete
through independent truth, C/Java foundations and checked i32-to-i64 source
casts. All 988 release/lint targets pass and whole-scope review is clean.
Native truth, original once-only operands, five compiling faults, seven
compositions, typed/atomic/API/privacy proof and actual exported packages
accompany the increment. Checked narrowing remains a distinct future
failure/result contract, not an unchecked cast substitute.

[02U — finite f64 constants](M35-03A-02U-finite-f64-constants.md) next extends
the separate compiler constant domain without conflating it with literal
inputs. Its independent oracle precedes separately gated C/Java constant
foundations and checked source/alias/import integration. The oracle checkpoint
is complete with all 991 release/lint targets passing and clean independent
review. The C target foundation is also complete: exact finite const storage,
original dependency authority, full native bit corpus and compiling fault
controls; all 992 release/lint targets pass with clean independent review.
Java target support and checked compiler-source admission are now complete.
All 1,001 release/lint targets pass and fresh whole-scope review is clean.
Native Rust/C/Java bit observations, original owner/type/value authority,
80 atomic source-rejection controls, exact manifest versions, actual Bazel
value/sign invalidation and restored cached tests accompany exported packages.
The broader scalar milestone remains incomplete.

[02V — signed-infinity constants](M35-03A-02V-infinite-f64-constants.md) next
extends the distinct constant domain to two exact signed values using typed
standard C/Java constants. Oracle, target foundations and source admission
are separately gated; NaN constants remain outside this increment.
The independent oracle is complete with all 1,004 release/lint tests passing
and clean whole-scope/hardening reviews. The C foundation is complete too:
typed standard constants, exact signed inventory/import facts, native bit and
compiling-fault proof, all 1,005 release/lint targets passing and clean broad
review. Java foundation is complete too: exact standard fields, typed signed
inventory and original alias/import authority, native Java21 normal/-Xint proof
with recompiled fault dependents, all 1,006 release/lint targets passing and
clean broad review. Checked source integration is complete too: 39 original
Rust reads, 66 target observations/configuration, typed authority/atomic controls
and three compiling faults pass. All 1,012 release/lint targets pass, broad
review is clean, and actual producer-sign invalidation/restored cache reuse is
proven. Real packages are exported; all 462 prior generated output hashes and
38 unrelated WIP hashes remain unchanged. NaN constants and wider parity stay open.

[02W — Unicode scalar values](M35-03A-02W-character-values.md) next addresses
Rust char literal/transport/comparison semantics. The independent full-domain
oracle precedes C U32 and Java Int target foundations and checked source
integration. No frontend character admission is enabled by the oracle step.
Character constants, conversions and text behavior remain separate.
The independent oracle is complete: 1,112,064 scalars, all 2,048 surrogates,
80 out-of-range inputs and 4,453 comparison pairs match both Rust profiles.
Seven actual value/admission/order faults are detected. All 1,015 release/lint
targets pass and broad review is clean; existing output/WIP is unchanged.
The C character foundation is complete: certified full-domain U32 transport,
six comparisons, original dependency forwarding and private record/helper
coverage pass with GCC14/Zig O0/O2 and GCC UBSan, including compiling width/order
faults. All 1,016 release/lint targets pass and fresh broad review is clean;
501 prior output hashes and 38 unrelated WIP hashes remain unchanged.
Java foundation and checked Rust-source admission are still pending.

Add remaining f64/char/unit values, constants/aliases, Boolean operations,
integer checked/wrapping/bitwise/shift/conversion operations, comparisons and
floating arithmetic/inspection to the Rust-source C and Java mappings. Existing
i32/bool support is only partial parity. Specify source forms and target rules
per operation; register real typed lowering functions, never support markers.

## Definition of done and tests

- Separate operation-specific implementation tasks before enabling support.
- Native Rust/C/Java agree on boundaries, overflow/division/shift/conversion
  failures, short-circuit evaluation, NaN, infinities, negative zero and bits.
- No C signed-overflow UB or Java masking/narrowing approximation.
- Unsupported source rejects before output; constant/type/call provenance and
  compile-negative mapping contracts remain enforced.
- No copied/custom runtime artifact; generated support uses ordinary typed AST.
- Full isolated gates, fresh review and separate tested commits per operation.
