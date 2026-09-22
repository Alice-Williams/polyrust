# M35-03A-02Q-04 — Checked Rust wrapping-addition integration

- Status: complete
- Parent: [02Q](M35-03A-02Q-wrapping-addition.md)
- Depends on: [Java foundation](M35-03A-02Q-03-java-addition.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-addition.md)

## Contract

Authenticate the actual primitive core operation in canonical HIR/TypeckResults.
Register a private-input executable WrappingAddition mapping per language through
the typed builder. Materialize original left then right exactly once.

## Definition of done and tests

Actual multi-crate Rust generates normal C/Java packages. Native modular results
and per-input producer traces agree with Rust and the independent oracle; exact
original APIs, imports, docs and external privacy controls pass. Missing,
duplicate/wrong bindings and private witness construction fail compilation.
Canonical/input/context and actual typed AST/dataflow probes detect faults,
including operand reversal despite commutative results. Probe/production bytes
match. Unsupported widths, casts, borrowed receivers, generic/trait lookalikes,
ordinary + and other wrapping operations reject atomically. Existing ordinary
functions named wrapping_add keep normal direct-call semantics.
Export actual examples, preserve older output/WIP hashes, pass the full isolated
release/lint gate and fresh review, then commit/push separately.

## Implementation and focused evidence

WrappingAddition authenticates canonical HIR, original TypeckResults, the core
primitive inherent DefId, nongeneric safe signature and unadjusted I32/I64
operands/result. C and Java register executable mappings in their consuming
builders. Both materialize the original left operand completely before the
right. C constructs certified unsigned addition and guarded signed conversion;
Java constructs exact primitive Add. Neither mapping emits source text or a
custom runtime. Original direct-call dependency discovery still visits both
operands and ordinary functions named wrapping_add keep direct-call semantics.

Fourteen compile-fail registration/private-witness contracts pass. Seventy-two
atomic rejection cases cover unsupported widths, borrowed operands, casts,
ordinary addition, other arithmetic, generic operands and method lookalikes;
existing output sentinels remain untouched.

Three actual source crates retain ten original function identities (including
two private helpers), a public module, exact signatures, docs and dependency
inventories. Native proof compares 15,790 Rust/oracle inputs and 31,580 target
observations per run. Java21 and GCC14/Zig producers and external clients compile
separately; C runs at O0/O2 and under UBSan. Measured native Rust and target
operand traces agree. Three compiled value faults (XOR, disconnected right
result and narrowing) and three value-preserving evaluation faults (dropped,
duplicated and reversed calls) are detected. External private access fails;
exposing the same helpers in disposable copies makes those controls succeed.

Ten actual typed-AST observations per target check method/associated forms,
canonical/context/width witnesses, nested addition, complete expanded dataflow
and ordered original calls. Copied nodes, swapped stored children, wrong width
and disconnected right results are rejected or detected. Probe and production
output bytes match. Separately compiled native composition covers both call
forms, nesting, complement grouping and an ordinary same-spelled function.
Focused AST/composition gate on tree a6ae25563c0f85ce36f7c3bd5a6c978c7c2c5efc:
6cc93be7-d6c8-4537-90f9-6c73fc4cd24a.

Initial harness corrections used the actual scalar manifest schema versions
(Java 1, C 2), checked C owned signatures in real headers/definitions rather
than nonexistent metadata fields, and normalized rustc query results in the
test-only witness probe. These were test repairs, not production relaxations.

Actual packages, original Rust and baseline external clients are copied and
compared byte-for-byte at generated/examples/wrapping-addition-a6ae2556/README.md.
These are ignored local artifacts, never checked-in generated output.
All 355 older generated package files and 38 recorded unrelated WIP hashes
match. The new bundles add 17 files without altering earlier packages.

## Review and integration repair

Independent Sol Extra High review found no core mapping or proof defect across
compiler authentication, typed registration, ordered lowering, numeric safety,
native/dataflow/mutation proof, provenance and privacy. Its only hygiene note
was to keep Python bytecode untracked; the explicit candidate index excludes it.

The first full gate (77bd413a-4ee6-4f13-b5be-dd1b99ddf68e) passed 539 tests,
then stopped at a pre-existing constant-domain compile-fail fixture. Its shared
source-capability re-exports now included the new capability, but its standalone
root did not exercise those types, adding an unwanted unused-import error beside
the intended E0308. The root now exercises discovery, context and accessors,
the Capability bound and both width variants. No lint was suppressed and the
exact intended error count remains required. Independent delta review is clean.
The second gate (7529e093-b5d0-4b32-b98c-504a0a7d61fb) passed 923 of 924
tests. The remaining failure was an older wrapping-negation rejection case
expecting wrapping_add to be unsupported. That case now uses wrapping_mul,
which remains outside admission; its atomic-output checks stay enabled and
addition's positive native/composition coverage remains mandatory. No production
mapping changed for either integration-fixture repair. A fresh whole-scope
review and the final full-gate result follow.

Repaired source tree 0db12329b7060fdede7bd4758b20e90e93dfaab6 passes all
924 Linux Bazel/native/lint targets, c88d2697-bead-411b-b9eb-23fcd2085c90
(8 executed, 916 cached). The tested checkout matches the candidate index.
All 355 earlier generated files, 38 protected WIP hashes and the actual exported
example bytes still match after that gate. No legacy path or test was disabled.

A fresh second whole-scope Sol Extra High review found no remaining core defect
or required proof gap, including both older-fixture repairs. Its optional
checked_add program-level rejection case was added to the existing atomic suite
(18 valid Rust programs, two targets, new/existing destinations: 72 checks).
The final documentation/test-only closure is gated again before commit/push.
