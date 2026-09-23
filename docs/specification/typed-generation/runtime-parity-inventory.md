# C/Java runtime migration inventory

- Status: baseline recorded; replacement parity incomplete
- Contract: [ordinary generated packages](runtime-free-packages.md)
- Plan: [M35-03A](../../plan/tasks/M35-03A-runtime-free-parity.md)

This is a coverage map, not a claim that all legacy capabilities already work
through Rust source. The machine-readable inventory is
`experiments/rustc-frontend/test/runtime_parity_inventory.json`. Its drift test
accounts for all 42 registered Java capability mappings, 38 explicitly admitted
C intrinsic operations, nine Java helper families and 17 C template sections.
Java helper variants must match the executable ALL catalogue registrations,
just as capability slots must match their consuming builder registrations.
These counts describe the current implementations, not a shared language spec.

## Current replacement coverage

Both targets currently share the closed no-heap source subset below. Neither
target has full replacement evidence for any broad legacy capability family.

| Functionality | Rust-source C | Rust-source Java | Remaining work |
| --- | --- | --- | --- |
| Values and comparisons | i32/i64/bool literals and evaluated constant reads, full-domain char literal/transport/comparison mappings and authenticated char constants/imports/aliases, immutable places, built-in bool negation, lazy/eager operators, signed integer bitwise operations, i32/i64 wrapping operations, finite f64 literals/transport/comparisons and finite/signed-infinity constants | Same source subset | unit storage, NaN/wider constants, type aliases and remaining integer/float/character operations: M35-03A-02 |
| Functions and modules | Closed scalar/unit-result signatures and value/effect calls, crate-owned headers and implementations | Same signatures/calls with crate-owned Java packages | Wider signatures, methods and migrated consumers: M35-03A-03/06 |
| Records and control | Closed scalar-field records/shared borrows, local bindings, structured branches | Same source subset | Owned shapes, enums, interfaces, loops and patterns: M35-02 then M35-03A-03 |
| Text, Unicode and bytes | No general replacement mapping | No general replacement mapping | All legacy operations and explicit encoding/indexing policies: M35-03A-04 |
| Lists, options and results | No general replacement mapping | No general replacement mapping | Container operations, nested values and failure propagation: M35-03A-05 |
| Portable tests, CLI and corpus | Legacy portable generator still active | Legacy portable generator still active | Move consumers only after their feature proofs pass: M35-03A-06 |

Compiler observations of Box/owned-record operations are not executable C or
Java heap support. Their work remains in M35-02; do not count them as parity.

[M35-03A-02A](../../plan/tasks/M35-03A-02A-boolean-negation.md) adds partial
JavaBooleanLogic coverage through built-in bool negation. Its
`boolean_negation_native_test` checks 8,204 inputs and 13 results against Rust
and independent truth values, with Java lint and GCC/Zig O0/O2 consumers.
`boolean_negation_rejection_test` covers unsupported/invalid operands and
atomic publication; `boolean_negation_contract_test` checks registration and
input privacy.

[M35-03A-02B](../../plan/tasks/M35-03A-02B-short-circuit-booleans.md) extends
that partial coverage with built-in bool lazy and/or. Its native test checks
112 results and exact operand-call traces in Java and GCC/Zig O0/O2 against
Rust truth and an independent oracle. Eager, duplicate and reordered native
mutants preserve truth but fail the trace oracle. The AST probe checks all
24 lazy nodes per target and byte-identical probe/production output; source
boundary and mapping-contract tests retain atomic rejection and typed inputs.
The broader legacy operand grammar is not yet migrated: full_features stays
empty, and this evidence does not enable source mutation or heap values.

[M35-03A-02C](../../plan/tasks/M35-03A-02C-i64-values.md) adds partial
JavaI64Values and wide comparison coverage. `i64_native_test` compares 17,640
results across two actual crates against Rust and an independent exact-integer
oracle, with separately compiled Java/GCC/Zig O0/O2 consumers. Deliberate
narrowing mutations must disagree with that oracle. Instrumented test copies
check exact wide operand-call order, and six reordered-comparison mutants must
retain truth but fail trace expectations. Width metadata mutations
must fail the signature contract. `i64_rejection_test` checks 44 atomic invalid
or unsupported cases and Java's 255/256 parameter-slot boundary. Input-privacy
compile contracts and target dependency/platform/resource tests protect the
typed boundary. This does not enable arithmetic, casts, foreign re-exports or
heap shapes, and full_features remains empty.

[M35-03A-02D](../../plan/tasks/M35-03A-02D-integer-bitwise.md) adds partial
JavaIntegerBitwise coverage for built-in i32/i64 complement/and/or/xor.
`bitwise_native_test` checks 176,384 exact results across real two-crate
Rust/C/Java packages and independent integer truth, including each bit position,
sign bits and alternating patterns. Native traces detect reordered calls even
when results are unchanged; dropped-complement controls fail the value oracle.
`bitwise_ast_test` inspects 28 mapped nodes per target and requires identical
probe/production bytes. Fourteen compile-negative mapping contracts, 40 atomic
source rejections, and C/Java target admission/purity/certificate tests protect
the boundaries. Shifts, casts, arithmetic and heap shapes remain unsupported.
The milestone's full release gate and independent review passed; the whole
runtime migration is still incomplete and full_features stays empty.

[M35-03A-02E](../../plan/tasks/M35-03A-02E-eager-booleans.md) adds
built-in bool eager And/Or/Xor. Its passing proof covers 104 exhaustive results,
real two-crate native consumers, evaluation mutations, 15 actual mapped AST
nodes per target, atomic rejection and executable mapping contracts. The full
588-test gate and independent reviews passed; this is not full catalogue parity.

Evidence anchors for the current subset are the executable registrations in
`src/c_lower/capabilities/mod.rs` and `src/java_lower/capabilities/mod.rs`, their
object/literal type mappings, and the crate native differential targets under
`experiments/rustc-frontend`. The new `runtime_free_bundles_test` checks actual
four-crate bundles produced by these paths. It does not prove semantic parity
or detect every possible support routine inlined into a source-owned file.

## Legacy-specific limits and disposition

C's callable-container validator admits List<String> and Option<I64>, not all
container shapes. It emits enum ABI declarations but does not provide general
callable enum lowering. Its callable blocks reject statements and bounded
iteration. The intrinsic inventory is the validator's allowlist, not every
Intrinsic enum variant or occurrence in the generator. C ABI/type handling and
constant restrictions must also be audited when their replacement tasks start.

Java registrations cover a broader portable model, including interfaces,
enums, collection/result operations and test lowering. A registration is not
proof that every nesting, ownership or generic shape is supported. Each task
must enumerate its successful legacy cases and add corresponding source and
native tests before declaring that family complete.

Every machine-inventory entry has exactly one primary parity task. Helper
families may serve several kinds of operation: for example, C allocation and
ownership template sections are filed under text/bytes for retirement tracking,
but their replacements also depend on M35-02 and collection work. This grouping
does not authorize deleting a shared helper when just one caller is migrated.

## Baseline checks and limitations

`runtime_parity_inventory_test` compares lexical registrations/allowlists and
template markers against the checked-in inventory. It rejects injected missing,
extra and duplicate entries. It deliberately is not a Rust parser or a semantic
admission checker; implementation refactors must update its extraction rules.

`runtime_free_bundles_test` requires exactly the expected source-owned files,
manifests and root identities, with no extra runtime or arbitrarily named
support file. Fault controls remove source, duplicate crates, add files and
introduce legacy and renamed support references. Exact index/member/manifest
key sets and manifest versions reject hidden support metadata. C imports must
identify an external definition in the declared owner's header, and this fixed
corpus has an exact include inventory. Java dependency owners must be unique,
non-self and in-bundle; this corpus needs no Java import declarations.

These are fixed-corpus lexical/metadata regression checks, not a general source
parser. Fully qualified Java expressions, C declarations, inlined routines and
arbitrary program behavior remain the typed compiler/native/privacy gates'
responsibility. No guarantee is inferred from the absence of an import line.

Future coverage updates must cite feature-specific positive, negative and native
tests. Preserve the baseline inventory until each old implementation and all
its consumers can be removed together with reviewed replacement evidence.

## Scalar constant read increment

[M35-03A-02F-01](../../plan/tasks/M35-03A-02F-01-constant-reads.md) adds
compiler-evaluated private module and inherent-associated bool/i32/i64 reads
through the ScalarConstants slot. Native two-crate, exact-literal/provenance,
compile-contract and atomic-rejection tests accompany the implementation.
Public/local declarations remain 02F-02 work; this is not full constant parity
or permission to remove the legacy constant/runtime entry points.

## Unit-result increment

[M35-03A-02G](../../plan/tasks/M35-03A-02G-unit-results.md) adds checked Rust
unit function results and effect-only direct calls/blocks/conditionals, mapped
to ordinary C/Java void. This is partial JavaUnitValues coverage, not unit
storage/parameters or full family parity. `unit_ast_test`, `unit_native_test`,
`unit_rejection_test` and `unit_contract_test` prove typed mapping, three-crate
ABI/behavior, call/condition order, atomic rejection and builder/input privacy.
Typed target void tests retain original-owner/signature/resource controls.
Legacy custom runtimes remain until the other inventory gaps are closed.

## Wrapping-negation increment

[M35-03A-02H](../../plan/tasks/M35-03A-02H-wrapping-negation.md) adds partial
JavaWrappingIntegerArithmetic coverage: actual built-in i32/i64 wrapping_neg,
not ordinary potentially overflowing unary minus. The compiler's private
canonical witness maps through the typed builder to guarded C and native Java.
wrapping_native_test compares 52,500 values from 8,750 boundary/random inputs
against Rust and independent modular truth, with separately compiled owners,
GCC/Zig O0/O2, GCC UBSan and Java 21 strict lint. Value-preserving receiver
drop/duplicate mutations fail trace checks. wrapping_ast_test proves exact
node shape, width, receiver count and identity; wrapping_contract_test and
wrapping_rejection_test enforce compile/atomic boundaries. All 778 test targets
and fresh review pass. No full legacy family is retired by this increment.

[M35-03A-02I](../../plan/tasks/M35-03A-02I-binary64-values.md) adds partial
JavaF64Values and exact float comparison coverage. Checked rustc evaluation
constructs private finite witnesses; the targets render primitive hexadecimal
double literals, not bit-wrapper runtimes. Three-crate native proof compares
31,548 results across Rust/Java/GCC/Zig, with exact finite bits, nonfinite
classification/comparisons and call-order/count fault detection. AST probes
check original compiler/target identities and output equivalence; 52 atomic
boundary cases and exact Java slot limits pass. The full gate passes 786 tests,
and 138 files across 18 old bundles are byte-identical. Floating constants,
arithmetic, casts, methods and NaN payload preservation remain outside this
increment; full_features remains empty and no legacy runtime is removed.

## Binary64 inspection increments

The subsequent bounded source mappings add primitive negation (02J), standard
inherent NaN classification (02K), absolute value (02L), and truncation (02M).
Each has a private compiler witness, executable C/Java mappings, native
value/category and call-count controls, AST/dataflow observations and unsupported
source diagnostics. Truncation additionally carries certificate-derived C
system-library metadata through package publication and consumer linking.
These extend partial floating-point coverage, not complete replacement of
JavaFloatingPointArithmetic or JavaFloatingPointInspection. Track completion
and exact gate receipts in the corresponding operation tasks; full_features
remains empty and the legacy runtime cannot yet be removed.

## Binary64 arithmetic increment

[02N](../../plan/tasks/M35-03A-02N-floating-arithmetic.md) adds checked built-in
f64 addition, subtraction, multiplication and division through the private
canonical ArithmeticInput and executable C/Java FloatingArithmetic slots.
Three actual Rust crates produce ordinary runtime-free C/Java packages; 45,038
exact observations per target run, independent call traces and nine compiling
fault controls cover grouping, separate rounding and once-only ordered operands.
Canonical AST/dataflow probes, fourteen negative mapping contracts, forty
atomic unsupported-source cases and strict native composition clients accompany
the source mapping. The full 879-test release/lint gate passes.

The following increment adds remainder. Neither increment marks a broad legacy
family fully migrated or authorizes removing its runtime implementation;
full_features remains empty.

## Binary64 truncating remainder increment

[02O](../../plan/tasks/M35-03A-02O-floating-remainder.md) adds built-in f64 %
through a separate private RemainderInput and executable FloatingRemainder
binding. C uses its typed standard fmod catalogue and certificate-derived -lm;
Java uses primitive double %. No custom runtime artifact is introduced.
Three source crates produce 22,832 target observations against 11,416 native
Rust cases and an independent integer/rational oracle. Actual Rust operand
traces, six compiling semantic faults, exact import/declaration/documentation
inventories and external privacy failures have mutation-sensitive controls.
All 900 release/lint targets pass; operation task 02O-04 records exact tree,
review and gate receipts. Integer %, Euclidean remainder, overloaded Rem and
f32 remain outside this capability. Wider legacy parity remains incomplete.

## Negative-zero composition proof

[02P](../../plan/tasks/M35-03A-02P-negative-zero-composition.md) expresses the
existing Apache-2.0 stdlib numeric predicate as ordinary Rust: compare with
zero, lazily call a private reciprocal function and compare its result with
zero. It reuses existing checked mappings and adds no special frontend form,
capability, runtime helper or standard math-library dependency.
The additive proof compares 86,017 raw-bit inputs with native Rust, upstream
JavaScript, an independent bit oracle and strict C/Java consumers. Per-input
native traces and compiling eager/duplicate-call mutations exercise laziness.
This does not replace the old eight-language example path yet; legacy corpus
gates remain enabled and full_features remains empty.

## Wrapping signed-addition increment

[02Q](../../plan/tasks/M35-03A-02Q-wrapping-addition.md) adds actual core i32/i64
wrapping_add through a canonical private compiler witness and executable typed
builder mappings. C uses certified unsigned arithmetic and guarded signed
reconstruction; Java uses primitive Int/Long Add. No runtime helper or raw target
source is introduced. Ordinary Rust + and other integer operations remain closed.

Three original crates yield 31,580 target observations per run against 15,790
native Rust/independent modular cases. Actual operand traces, compiled value/
evaluation faults, typed expanded dataflow, exact API/docs/import inventories and
external privacy controls pass. All 924 release/lint targets pass and two source
reviews are clean. This is still partial WrappingIntegerArithmetic coverage;
full_features remains empty and legacy paths stay enabled.

## Wrapping signed-subtraction increment

[02R](../../plan/tasks/M35-03A-02R-wrapping-subtraction.md) adds actual core
i32/i64 wrapping_sub through private canonical witnesses and executable builder
bindings. C uses certified unsigned subtraction and guarded signed reconstruction;
Java uses primitive Int/Long Subtract. Source operands are materialized once,
left before right, and original dependency/API/visibility/docs identities remain.

All 945 release/lint targets pass with a clean whole-scope review. Native proof
checks 15,790 Rust/oracle inputs and 31,580 target observations per run, measured
operand traces, seven compiling value/order faults, typed expanded dataflow and
atomic rejection. Actual three-crate examples are exported. This remains partial
WrappingIntegerArithmetic coverage; full_features stays empty and legacy gates
remain enabled until the wider migration has replacement proof.

## Wrapping signed-multiplication increment

[02S](../../plan/tasks/M35-03A-02S-wrapping-multiplication.md) adds authenticated
core i32/i64 wrapping_mul through private canonical inputs and executable builder
bindings. C uses exact-width unsigned multiplication and guarded signed
reconstruction; Java uses primitive Int/Long Multiply. Original operands are
evaluated once, left before right, with original source ownership/API/docs intact.

All 966 release/lint targets pass and independent whole-scope review is clean.
Native proof compares 34,546 Rust/oracle cases and 69,092 target observations per
run, including measured operand traces and six compiling value/order faults.
Typed expression/dataflow/forged-witness, compile-negative, atomic publication,
external privacy and actual package-export evidence pass. All 389 previous
generated files and 38 unrelated WIP hashes are unchanged. Wrapping-integer
coverage remains partial, the machine inventory still has no full_features,
and all legacy gates stay enabled.

## Finite binary64 constants increment

[02U](../../plan/tasks/M35-03A-02U-finite-f64-constants.md) extends the existing
constant capabilities with a distinct finite-value witness, checked only after
rustc confirms f64/eight-byte scalar representation. Both targets retain exact
bits, including zero sign, through public/private/local/inherent reads,
original producer certificates, imports and aliases. Descriptive manifests use
fixed-width hexadecimal bits; they never manufacture dependency authority.

The C/Java target foundations independently cover 24,576 constant observations.
Checked source integration is complete: its focused native proof passes 34
original Rust reads and 62 target observations/configuration, with strict
GCC/Zig O0/O2, UBSan and Java21 normal/interpreted execution. Six compositions,
three compiling producer-value faults, original docs/privacy/alias inventories,
typed mapping probes and 80 atomic boundary rejections pass. All 1,001 release/
lint targets pass, fresh independent review is clean, actual producer value/
zero-sign changes invalidate affected Bazel outputs and restoration reuses
cached passing tests. Four actual packages are exported and hash-verified;
all 423 prior generated files and 38 unrelated WIP hashes remain unchanged.
This is partial Constants coverage; full_features remains empty.

## Signed-infinity constant integration

[02V](../../plan/tasks/M35-03A-02V-infinite-f64-constants.md) extends the distinct
checked constant domain with Infinity(Binary64Sign), preserving the finite
literal invariant. Both targets use existing standard-symbol ASTs: C's typed
HUGE_VAL/negation and Java's typed Double infinity fields. Public declarations,
private/local/inherent reads, exact producer certificates, imports and aliases
retain original identity. No copied runtime or new rendering escape hatch exists.

Native integration proof passes 39 original Rust reads and 66 target observations
per configuration, including three compiling producer faults with recompiled
dependents, exact docs/privacy and symbol-derived dependency inventories.
All 1,012 release/lint targets pass and broad independent review is clean.
Actual producer-sign changes rerun seven affected actions while the independent
producer stays cached; restoration recovers original hashes and cached native
success. Actual packages are exported and old generated output/WIP is unchanged.
NaN constants, f32 and wider storage/constant forms remain rejected. This remains
partial Constants coverage; full_features stays empty and legacy gates remain.

## Unicode scalar source integration

[02W](../../plan/tasks/M35-03A-02W-character-values.md) adds checked Rust char
literals, immutable places, scalar signatures/calls/conditionals, six comparisons
and scalar record fields. C uses U32 and Java uses Int. Typed source facts retain
Char separately from I32, original function identities and each field's owning
record; compiler authentication and certified target reconciliation are distinct.
Foreign calls additionally check the producer's source signature. Foreign callers
must satisfy the original Unicode scalar domain; target integers alone do not
prove that precondition. Character constants, casts, methods, direct references
and general text/encoding operations remain unsupported.

The focused source gates cover 1,116,517 native corpus rows, 19 literal boundaries,
actual operand traces and compiling value/order faults, 30 atomic type/owner/call
faults, eight unsupported foreign re-exports and 48 source-boundary cases. Actual
three-owner C/Java examples are exported to the ignored generated directory.
All 1,024 release/lint targets pass and fresh broad reviews are clean. Actual
producer-value changes rebuild seven affected actions, fail old truth and pass
updated truth; restoration recovers original hashes and cached native success.
This is partial JavaCharValues coverage, not permission to remove any legacy runtime;
full_features remains empty.
