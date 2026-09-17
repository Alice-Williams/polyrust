# Rust unit results in Java 21

- Status: implemented and independently reviewed (M35-03A-02G)
- Contract: [shared unit results](../../rust-unit-results.md)

## Target representation

Map the function result to existing JavaType::primitive(JavaPrimitive::Void), method invocation statements
and Return(None). Never use java.lang.Void, null, boxing, an allocated singleton,
a fabricated field or a helper runtime. Value TypePlan and scalar expression
constructors remain distinct from the effect-only unit mapping.

## Certificate and dependency boundary

The closed source/dependency profile admits scalar parameters and scalar/void
results, while storage, constants and ordinary expression-value positions remain
scalar/record-only. Permit direct void invocation statements and bare returns
only through existing typed AST and verifier contracts. Visit call arguments and
targets, preserve exact signatures and reject mismatched returns or void values.

Retain original JavaDependencyFunction authority, qualified member paths, complete
owner closure, visibility and source identity for unit-returning calls. Account
for every actual call and its call height; pure/void flags do not justify erasing
the call or skipping admission of its body. Existing expression/node budgets and
original-certificate reconstruction remain mandatory.

## HIR lowering and publication

FunctionSignatures maps only the checked empty-tuple result to JavaType::primitive(JavaPrimitive::Void).
UnitEffects creates statement-position calls, preserving scalar argument order
through existing materialization. Unit conditionals lower to ordinary if blocks;
no result local is needed. Unit completion emits a bare return or falls through
as specified by the typed control mapping.

Unit-aware owner metadata uses schema 5 if any retained owned function,
including private functions, returns primitive void. Its result is encoded as
unit; parameters, fields and constants remain bool/i32/i64. Existing scalar-only
schemas 1 through 4 and the bundle-index schema stay unchanged. Function result encoding admits unit independently of scalar
parameters and constant values. Reconstruct metadata from the original certified
method/signature, not from rendered Java or a caller-supplied type string.

## Proof

Separately compile producer, transitive consumer and handwritten Java 21 client
with strict lint; reflect on return types to prove primitive void and no unit
helper classes/fields. Typed tests reject void-as-value, forged/wrong callable
signatures and return mismatches. Native source tests cover empty/explicit unit,
mixed scalar methods, conditional/tail/statement calls, docs and original producer
paths. Controlled instrumentation checks sequencing without enabling arbitrary
I/O in production source admission.
