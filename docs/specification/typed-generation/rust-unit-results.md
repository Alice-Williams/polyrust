# Checked Rust unit results

- Status: implemented and independently reviewed (M35-03A-02G)
- Scope: function results and effect-only expressions, not general zero-sized storage
- Implementations: [C17](languages/c/rust-unit-results.md), [Java21](languages/java/rust-unit-results.md)

## Source boundary

Admit the built-in empty tuple type () as the result of a safe nongeneric
ordinary Rust function. Existing bool/i32/i64 parameters remain unchanged.
Unit parameters, unit locals/fields/constants, arbitrary tuples, references to
unit, never-returning functions, generic calls and external library effects stay
unsupported until their own mappings exist. Selected-entry mode keeps its
existing fn(i32) -> i32 contract; public-package selection gains unit results.

A unit result is absence of a returned value, not null, integer zero, an empty
record, an allocated singleton or a runtime object. Do not route it through a
scalar value constructor or pretend that C void is an object type.

## Typed layers and executable mappings

Keep the existing HIR, TypeckResults, original callable identity and target
certificate boundaries. FunctionSignatures classifies the actual checked
signature. A dedicated UnitEffects capability has private checked UnitInput,
with a closed operation enum and executable target mapping. Its output is
target statements/effects, not a fabricated CValue or Java scalar Value.

Initially admit empty () expressions, empty block completion, unit-returning
direct calls, unit-typed blocks and if/else. Statement-position unit calls may
precede a scalar or unit tail. A missing else is valid only for unit control.
Use existing lexical scopes and budgets. Reject early return until explicit
control-flow evidence is added; do not silently drop statements after it.

Unit calls retain registered local or original certified imported callable
authority. Evaluate scalar arguments exactly once in Rust source order before
the call, retain the call as an effect statement, and never assign a void call
to a temporary. Tail unit effects precede a bare return or ordinary completion.
ControlCompletion::Return and ControlCompletion::Effect distinguish return
contexts from statement blocks; only return completion can append a bare return.
Conditional effects remain in their branch. All call-graph, source traversal,
depth, package and target resource limits still apply.

A callable's pure signature flag is not authorization to omit its body or call.
The supported bodies remain within the existing no-mutation/no-throw source
profile; unit support itself does not admit I/O, allocation, panic or arbitrary
library calls.

## Publication and proof

Preserve public/private declarations, crate files, imports, docs, original
producer closures and exact return/parameter agreement. Version manifests that
extend the result domain; old scalar-only schemas keep their meaning. Metadata
does not construct a callable witness.

Prove target certificate admission independently of compiler lowering. Then
prove checked Rust source generates separately compiled C/Java libraries and
native consumers. Positive cases include empty/explicit unit, nested calls,
mixed scalar/unit functions, conditionals and transitive imports. Negative cases
cover wrong result type, void used as a value, unsupported unit storage/parameters,
forged callable authority and mismatched body/return statements. Tests inspect
typed call/effect nodes and native ABI; controlled native instrumentation checks
argument and branch sequencing. No runtime, wrapper or unit storage is emitted.
