# M35-03A-02R-03 — Java wrapping-subtraction foundation

- Status: complete
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [C foundation](M35-03A-02R-02-c-subtraction.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-wrapping-subtraction.md)

## Contract

Admit exact primitive Int/Long Subtract with Additive precedence and recursively
checked operands. Reuse normal target AST, authority, certification and renderer.
Keep existing Add/Double behavior unchanged and unrelated integer operators closed.

## Definition of done and tests

Both widths compile in separate strict Java21 producer/client units and agree
with independent modular truth. Detect addition, reversal, saturation, narrowing
and disconnected-operand faults. Mixed widths, boxed/string/Boolean values,
incorrect result/precedence and missing-import/arity faults reject in both operand
positions. Depth, call-height and byte-budget controls remain active. Full gate,
preserved old output/WIP, fresh review and separate commit/push. No source admission.

## Implementation sequence

1. Prepare shared integer fixtures and separate subtraction shape/native tests;
   no production admission before the C foundation checkpoint closes.
2. Extend the exact primitive dependency-body category from Add to Add/Subtract,
   preserving recursive validation. Update the direct reader's exhaustive type/
   operator/result/precedence matrix and obsolete subtraction-negative assertion.
3. Run focused subtraction and addition regressions, full Linux release/lint gate,
   preservation checks and an independent whole-checkpoint review.
4. Record actual evidence, commit and push. Only then open checked source work.

## Implemented proof structure

The production reader adds Subtract beside Add in the exact Int/Long category;
both child expressions still recurse through the same authority/budget reader.
Double arithmetic and rendering are unchanged. A direct reader matrix checks
operand/result types, operator and precedence independently of public validation.
The public-certificate tests reject mixed/boxed/String/Boolean types, hidden
unsupported expressions in either child, incorrect imports and arity, and excess
depth. Original callable height and derived source-byte bounds remain checked.

Separate Java21 producer, forwarding consumer and external client compilations
use --release 21, -Xlint:all, -Werror, -implicit:none and an empty source path.
Both owners are compared with 15,790 independent signed-difference cases. Five
compiled controls (addition, reversed operands, saturation, half-width narrowing
and disconnected operands) must differ at each width and match their respective
fault oracle. Existing addition/negation native regressions remain active.

The first independent review identified one additional obsolete rejection in
dependency_api.rs missed by the initial test update. It now positively certifies
both Add/Subtract at both Int/Long widths. This changes the intended admission
expectation, not a safety obligation; all malformed/unsupported cases stay gated.
The receipts below close this target-only checkpoint.

## Release and review receipts

Amended tree 22217c78a887ed5a5e67ba463590ef9bb1cae831 passes all 927 isolated
Linux Bazel release/lint targets under 5f62f9ce-f062-4e99-b501-6f836fad1fd6
(9 executed, 918 cached), including all 400 Java unit tests. Clippy, rustfmt,
buildifier and source policies pass. All 372 existing generated files and all
38 protected ownership/adjacent WIP hashes are unchanged.

The preceding gate (8989e090-815b-45ae-afe9-3c298e424fb9) passed 926 targets
and failed only the obsolete dependency_api expectation described above. It ran
399 passing Java unit tests and that one failing test; no gate was disabled.

A fresh Sol Extra High whole-checkpoint review of the amended tree found no
actionable core issue. Optional suggestions were an operation-neutral native
harness filename and an additional rendered nested-subtraction associativity
assertion. Neither is a correctness repair: the reused test-only filename keeps
the existing source-policy boundary stable, and the unchanged renderer already
parenthesizes every binary expression. Checked-source nesting receives its own
native/typed-dataflow proof in the next checkpoint.
