# Checked Rust block-local scalar constants

Status: implemented and verified, M35-03A-02F-02A.

## Shared compiler boundary

LocalConstants consumes a private LocalConstantInput constructed from an actual
HIR item statement. Require a plain Const definition, no generic parameters,
and bool/i32/i64 compiler type. Preserve the statement and exact definition
identity. Reuse the scalar-constant compiler evaluator and width/type checks;
do not introduce a second interpreter or permit a fabricated value.

Validate even an unused local declaration. Other item kinds, wider values and
borrowed storage remain unsupported. rustc still diagnoses duplicate names,
invalid initializers and forbidden long-running evaluation before publication.
Source admission budgets, alias restrictions and complete compiler analysis
remain mandatory.

Rust resolves const items throughout their lexical block, independently of
runtime execution order. Do not put them into the runtime let-binding map or
manually resolve forward references/shadowing. ScalarConstants resolves each
use to its compiler DefId and emits the existing exact literal representation.

## C17 mapping

Register CLocalConstants with Reader context and unit output. The mapping
explicitly accepts the closed checked scalar value enum and emits no CStatement,
CLocalRef, CObjectRef, prelude, storage or import. LexicalControl dispatches
item statements to this mapping before ordinary let lowering; unknown item
statements produce diagnostics. Preserve CBlock scopes and let sequencing.

Existing reads use CLiteral/CSignedLiteral as in
[scalar constant reads](rust-scalar-constants.md). Never turn a local const into
a C static object or hide a definition inside a raw string.

## Java 21 mapping

Register JavaLocalConstants with Reader context and unit output. Emit no local
variable, field, static initializer, helper, prelude or import for the
declaration. Keep runtime locals and lexical scopes unchanged. Reads continue
to produce exact primitive boolean/int/long literal nodes through
ScalarConstants. This is compile-time erasure, not a Java Runtime feature.

## Evidence and exclusions

Probe actual mapper invocations and verify unchanged runtime state plus exact
compiler origin/value; probe adapters must emit byte-identical artifacts.
Native two-crate consumers and independent expectations cover forward reads,
nested shadowing, false/true, signed limits, computed values, ordinary lets,
records and branches. Compile-negative contracts guard registration and input
construction; atomic negatives guard unsupported item/types and invalid consts.

Public constant APIs, source generics/traits, constant borrows, wider scalar or
aggregate constants and general block expressions remain separate work. This
step does not complete the legacy JavaConstants capability or retire runtimes.
