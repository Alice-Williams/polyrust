# Rust unit results in C17

- Status: normative design; implementation in progress
- Contract: [shared unit results](../../rust-unit-results.md)

## Target representation

Use the existing CReturnType::Void, bare C return statement and typed void-call
statement. CObjectType and CScalarType remain object/value-only: do not add void
as a scalar, allow sizeof(void), or create a fake result object.

The shared graph needs an explicit CPrimitiveType enum with Scalar(CScalarType)
and Void cases. CDialect::PrimitiveType uses this vocabulary; every existing
scalar projection is wrapped in Scalar. Only a callable result projects Void.
Object/value references, parameters, fields and dependency constants still derive
their types from genuine CObjectType values. Void has no header dependency.

## Certificate and dependency boundary

Extend the bounded shared profile to allow void function returns, bare returns
and typed direct void-call statements. Existing AST constructors and control-flow
verification reject a bare return in a value function, a returned value in a void
function, a void call in value position, wrong arguments and alien callable
handles. Walk every argument and call target; retaining a statement must not
bypass dependency, namespace, effect, ownership or resource validation.

CDependencyApi and shared callable signatures admit scalar parameters with scalar
or void result. Original producer identity, full transitive closure and function
call/resource measurements remain mandatory. Unit-returning functions still count
as real functions/call frames. They are not alias-only packages or constants.
Void must not appear in the imported-constant type domain.

## HIR lowering and publication

FunctionSignatures maps built-in Rust () results to CReturnType::Void.
C UnitEffects emits existing typed statements. Evaluate and materialize scalar
arguments before constructing the void call; no temporary receives its result.
Keep normal extern prototypes/public headers and private internal linkage.

Unit-aware C owner metadata uses a new explicit schema, retaining existing
scalar-only versions. Callable results distinguish unit from bool/i32/i64;
parameters remain scalar-only. Standalone and bundled publication reconstruct
these signatures from the original registered function and producer witness.
Do not discover return types by reading emitted source or a JSON description.

## Proof

Target tests construct void producers/consumers, compile every public header,
separately compile/link with GCC and Zig at O0/O2, and verify ordinary void ABI.
Negative tests reject value/void return mismatches, void-as-value, wrong signatures
and recertified/missing owners. Compiler tests add actual Rust unit functions,
source-order/conditional call traces, docs and mixed/transitive packages.
No additional runtime/header, unit record, macro or accessor is emitted.
