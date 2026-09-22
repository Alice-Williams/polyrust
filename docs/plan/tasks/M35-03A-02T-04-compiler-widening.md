# M35-03A-02T-04 — Checked Rust signed-widening integration

- Status: complete
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [Java foundation](M35-03A-02T-03-java-widening.md)
- Specification: [shared](../../specification/typed-generation/rust-signed-widening.md)

## Contract

Authenticate canonical HIR Cast and original TypeckResults with exact unadjusted
i32 operand and i64 result. Use a private operation witness and a required
executable SignedWidening mapping slot. Preserve one original operand evaluation,
declaration/call identities, crate/module ownership, public/private API and docs.

## Definition of done and tests

Three original Rust crates and separately built C/Java external consumers agree
with independent/native truth. Measure native source and target operand traces;
detect dropped/duplicated calls even when values agree, and compiling value
faults. Cover literals, original local/imported calls, records and composition
with existing scalar operators. Typed probes verify exact target shape, original
operand dataflow, hostile copied/context/width witnesses and identical production
bytes. Missing/duplicate/wrong bindings and private witness construction fail
compilation. Unsupported casts reject atomically without changing old outputs.
Update older rejection fixtures only if they reject the newly supported exact
widening operation; maintain each fixture's intended failure. Export actual
packages, update partial inventory, preserve WIP, pass full gate and fresh review,
then commit/push. Checked narrowing and arbitrary From/Into calls stay outside.

## Implementation and pending proof

The private WideningInput authenticates the original canonical HIR Cast and
TypeckResults, exact unadjusted i32 input/i64 result, and original operand
identity. The 31st required consuming builder slot supplies an executable
SignedWidening mapping. Previous compile-negative fixtures supply the new slot
while retaining their intended failures. Both callable inventories recognize
the cast and still traverse the original operand for dependency ownership.

C and Java materialize the checked operand once and construct the existing
Numeric(I64) and primitive Long Cast nodes respectively. No target syntax is
embedded in production lowering, no renderer changes are needed, and no runtime
or third-party dependency is introduced.

Proof targets cover 73,890 independent/native inputs, three original crates,
two external entry points, strict separate GCC14/Zig O0/O2 and GCC UBSan,
and Java21 with normal and interpreted execution. Three compiling value faults
and two value-preserving dropped/duplicated-call faults challenge the actual
generated bodies. Original Rust producer traces are measured independently.
Exact APIs/imports/docs, standalone C headers and external private-access
negative/positive controls accompany the corpus.

Five typed observations per backend challenge copied HIR, another function's
checking context and a wrong-width substituted operand; reconstruct actual
temporary dataflow and original callable identities; and preserve identical
production bytes. A detached-result control retains the original call but
disconnects its result. Seven native compositions cover literals, nested calls,
shared borrows, record fields and wrapping/bitwise operators. Fourteen compile
negatives and 72 atomic rejected source invocations protect the admission edge.

Existing unsupported cast programs remain unsupported; their expected message
now identifies the exact signed-widening boundary rather than the former generic
missing-expression message. Alias uses remain behind the existing provenance
guard. Completion still requires full-gate, preservation, exported-example and
fresh independent review receipts below.

## Completion evidence

Reviewed implementation tree 64bd5dedaf6da87e1730fa8cb6dc29489008fe0b passes
all 988 Linux release/lint targets, invocation
1d10aff7-032c-4f43-b3e9-7ae7fdf23f49 (4 executed, 984 cached).
Focused source proof passed 5/5 targets in
7013363c-fe4a-4899-9a53-13bfa3915958; the 14 compile-negative mapping targets
also pass in the full gate. Native proof checks 73,890 inputs and 147,780
three-crate target observations per run, plus seven compositions per input.
All five compiling value/evaluation faults and 72 atomic source rejections pass.

The initial full run, 45ea0eb4-17b9-47b3-837d-21b1a704e139, passed 982/988:
five existing scaffolding targets needed explicit capability use/imports, and
one external-owner fixture's incidental cast intercepted its intended error.
The standalone probe now exercises the new witness; missing-slot fixtures
import the mapping; the external fixture retains its original call but discards
its result instead of casting. No lint, test, intended diagnostic or atomic
check was disabled. The corrected full run above covers every target.

Fresh whole-scope Sol Extra High review of all 77 scoped files found no
actionable defect or required proof gap. It independently audited the prior
cast-fixture changes and both test-only correction deltas. A structural audit
also confirms each C/Java builder slot replaces exactly one Missing and retains
all other 30 fields.

The isolated checkout matches the scoped index. All 406 previous generated-file
hashes match; new three-crate bundles add 17 files, for 423 total. All 38 preserved
unrelated WIP hashes match. Actual native-test artifacts are exported to
`generated/examples/signed-widening-85bb237e/`: 24 files of original Rust,
generated C/Java packages and handwritten clients, recursively byte-checked
against the test output. Generated files remain ignored/uncommitted.

Documentation-only closure receives a final full cached gate before this
checkpoint's commit/push. Checked narrowing and wider conversion/constant/heap
families remain open; no legacy runtime or path is removed by this increment.
