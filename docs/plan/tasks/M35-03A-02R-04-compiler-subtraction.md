# M35-03A-02R-04 — Checked Rust wrapping-subtraction integration

- Status: complete
- Parent: [02R](M35-03A-02R-wrapping-subtraction.md)
- Depends on: [Java foundation](M35-03A-02R-03-java-subtraction.md)
- Specification: [shared](../../specification/typed-generation/rust-wrapping-subtraction.md)

## Contract

Authenticate actual core i32/i64 wrapping_sub in canonical HIR/TypeckResults.
Register a private-input WrappingSubtraction mapping per target through typed
builders. Fully materialize original left before right, once each.

## Definition of done and tests

Multi-crate native Rust/C/Java values and measured traces agree with independent
truth. Exact original API/import/doc inventories and external privacy controls
pass, including deliberate corruptions. Missing/duplicate/wrong mapping bindings
and private witness construction fail compilation. Typed AST/dataflow and hostile
canonical/context/child probes detect faults; production/probe bytes match.
Method/associated calls, nesting, grouping and same-spelled ordinary functions
have positive composition coverage. Unsupported widths, borrowed receivers,
casts, generics/traits, ordinary subtraction and unrelated operations reject
atomically. Export actual packages outside Docker. Preserve old outputs/WIP,
pass the full gate and clean fresh review, then commit/push separately.

## Implementation sequence and distinguishing controls

Prepare ordinary Rust fixtures while the Java foundation is being verified, but
do not activate source targets or production discovery until that checkpoint is
complete. Keep operation-specific private input and mapping types in focused
capability modules; builder registration requires the full executable signature.

Use three original source crates (operand producers, a public operations module,
and a forwarding crate), plus a standalone composition fixture. Preserve both
call forms, nested/grouped expressions and an ordinary function named wrapping_sub.
The C mapping changes the internal unsigned operation only; the guarded signed
reconstruction remains independently certified. Java uses exact primitive Subtract.

Subtraction needs two distinct reversal controls: swapping the result operands
must produce the wrong value, whereas moving the original call statements while
retaining their values must preserve the result but change the measured trace.
Also detect wrong addition, narrowing, disconnected operands, dropped calls and
duplicate calls. Existing addition rejection of wrapping_sub becomes obsolete;
replace it with another unsupported operation rather than disabling the gate.

Update earlier missing-slot compile-negative fixtures with the newly required
binding so each still isolates its original missing capability. Keep all existing
addition output bytes and earlier source proof gates unchanged. Only final passing
and reviewed source integration earns the completed status and exported examples.

## Implementation and focused evidence

The private SubtractionInput authenticates canonical HIR, original TypeckResults,
the actual core primitive DefId, exact nongeneric safe signature and unadjusted
I32/I64 operands/result. Both consuming builders require an executable
WrappingSubtraction binding. Each target fully materializes the original left
operand before the right. C builds unsigned Subtract and certified guarded signed
reconstruction; Java builds primitive Subtract with Additive precedence. No raw
target text, runtime helper or new dependency is introduced.

The first focused run exposed an omitted call-inventory integration: associated
subtraction was being inspected as an ordinary direct call. Both inventory walkers
now recognize the checked witness and still traverse its original operands. All
older missing-slot fixtures supply the new binding, preserving their original
intended failure. The obsolete addition rejection of wrapping_sub now rejects
checked_sub instead; no negative gate is disabled.

Amended implementation tree 51c5b43c61ffba9efad09b7740e94290c0f51c3e passes
all 21 focused tests. Fourteen compile-negative mapping/private-input contracts,
72 atomic source rejections and ten actual AST observations per target pass.
Hostile copied/context/width/child witnesses and disconnected operand dataflow
are detected; production and probe output bytes agree. Native composition covers
method/associated forms, nesting, complement grouping and an ordinary function
whose name is wrapping_sub.

Three original crates preserve ten function identities, two private helpers, a
public module and exact signatures/docs/imports. Native Rust agrees with independent
modular truth on 15,790 inputs; each target run checks 31,580 observations. Java21
and GCC14/Zig producers and clients compile separately; C runs at O0/O2 and under
UBSan. Measured Rust/target operand traces agree. Four compiling value faults
(addition, reversed values, disconnected right operand and narrowing) and three
value-preserving evaluation faults (dropped, duplicated and reversed calls) fail
their respective oracles. External privacy controls and deliberately corrupted
API/import/doc inventories behave as required.

Actual tested packages, original Rust and clients are exported and recursively
byte-compared at generated/examples/wrapping-subtraction-51c5b43c/README.md.
They remain ignored artifacts. All 372 older generated files and 38 protected
unrelated WIP hashes match; the new source bundles add 17 files. Full release/lint
and independent review receipts are required below before completion.

## Completion evidence

The exact implementation tree above passes all 945 Linux Bazel/native/lint
targets, invocation 12f46681-3d4e-4492-989b-3a5cd6e66c6a (77 executed,
868 cached). No test or lint was disabled. Independent whole-scope Sol Extra
High review found no core or optional issue across witness authentication,
all 29 binding slots, both inventory walkers, numeric safety, ordered lowering,
native/dataflow controls, composition, provenance and privacy. This review
included the focused-run repair and older compile-negative fixture updates.

Documentation-only closure is gated again before the separate commit/push.
This completes wrapping subtraction, not the broader scalar/runtime migration.
