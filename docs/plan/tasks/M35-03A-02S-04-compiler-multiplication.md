# M35-03A-02S-04 — Checked Rust wrapping-multiplication integration

- Status: complete
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [Java foundation](M35-03A-02S-03-java-multiplication.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-multiplication.md)

## Contract

Authenticate core primitive wrapping_mul in canonical HIR/TypeckResults using a
private operation-specific witness. Register required executable target mappings.
Fully materialize the original left operand before the right, exactly once each.

## Definition of done and tests

Three source crates and external consumers agree with independent truth/native
Rust at both widths; measured operand traces detect value-preserving dropped,
duplicated and reordered calls. Exact original APIs, imports, visibility and docs
have mutation-sensitive controls. Missing/duplicate/wrong binding and private
witness construction fail compilation. Actual typed dataflow and hostile witness
probes detect faults; probe/production bytes agree. Native composition covers
method/associated calls, nesting, grouping and same-spelled ordinary functions.
Unsupported widths, borrows, casts, trait/generic lookalikes, ordinary * and other
unsupported operations reject atomically. Update obsolete unsupported wrapping_mul
fixtures to a still-unsupported operation without disabling their gates. Export
actual packages, preserve old output/WIP, pass full gate and fresh review, then
commit/push. Wider runtime parity remains incomplete.

## Implementation and pending proof

The private MultiplicationInput authenticates canonical HIR, the original
TypeckResults, core primitive DefId, nongeneric safe signature, unadjusted original
operands and exact I32/I64 width. Both call inventories recognize the operation
while retaining traversal of original operand calls. The 30th required consuming
builder slot binds an executable WrappingMultiplication mapping; previous
compile-negative fixtures supply this new slot without masking their own errors.

C materializes each operand left before right, converts to matching U32/U64,
multiplies unsigned and uses the existing guarded signed reconstruction. Java
materializes matching Int/Long values and constructs Multiply with Multiplicative
precedence. Neither mapping adds a runtime, helper catalogue or raw target text.

New proof targets cover three original crates, strict separate native consumers,
the 34,546-case independent product oracle, measured Rust/target operand traces,
three compiling value faults and three value-preserving evaluation faults.
Composition includes both call forms, nesting with a non-identity factor of three,
grouped complement and an ordinary same-spelled function. Typed probes expand
actual temporary bindings, challenge forged witnesses and compare probe/production
bytes. Fourteen mapping compile negatives and 72 atomic source rejections are
required. Old unsupported wrapping_mul controls now use unsupported wrapping_pow;
their atomic-publication checks remain active.

## Focused evidence and review

Reviewed implementation tree b014ec9fd3b4eb8d1b0d6b31fecbe652c594db29 passes
all 21 focused targets, c02d2615-791d-4d0b-b0ec-b5d805cfee54. These include
14 mapping compile negatives, source Clippy, actual target AST/dataflow/native
composition, 72 atomic source rejections and the three affected older rejection
suites. Native proof compares 34,546 Rust/oracle inputs and 69,092 target
observations per run with separate strict Java21/GCC14/Zig O0/O2 and GCC UBSan
builds. All six compiling value/order faults are detected. Ten observations per
target exercise canonical/context/width witnesses, hostile copies and swapped
operands, exact expressions, expanded dataflow and identical probe/production
emission.

Independent whole-scope Sol Extra High review of all 61 scoped files is clean:
no concrete defect, required proof gap or optional feature request. It checked
the complete registration/old-negative delta, compiler and target identity,
native fault sensitivity, atomic publication, source APIs/docs/privacy and the
partial-only parity inventory. An additional structural registration audit
confirms every C/Java slot replaces one Missing and preserves all other 29 fields.

Actual native-test output is exported to the ignored host folder
`generated/examples/wrapping-multiplication-b014ec9f/`: 24 files comprising the
original Rust, generated three-crate C/Java packages and external clients.
Recursive comparison confirms it is byte-identical to the test's output.

## Full completion evidence

The exact reviewed tree passes all 966 Linux release/lint targets,
944b06dd-4ecd-49a3-9852-00a515d8a27b (81 executed, 885 cached). The isolated
checkout matches the scoped index. After the gate finishes, all 389 previous
generated files match their baseline hashes; the new bundles add 17 files for
406 total. All 38 unrelated WIP hashes remain unchanged. No gate is disabled,
no dependency/runtime is added, and no legacy path is removed.

Documentation-only closure receives a final full cached gate before the separate
commit/push. Wider conversion, arithmetic, ownership and runtime parity remain
open; this is completion of the explicit wrapping-multiplication contract only.
