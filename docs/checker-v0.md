# Portable IR v0 checker contract

M04 converts an unchecked `portable_ir::v0::Document` into an immutable
`portable_check::v0::CheckedProgram`. This is the semantic boundary for the
reference evaluator and future backends. The earlier `CheckedModule` remains
temporarily available only for the pre-v0 prototype and is replaced at the
backend boundary in M08.

## Safe API boundary

`check_program(Document)` is the only safe construction path. `CheckedProgram`
has private fields and a crate-private constructor. Its read-only API exposes:

- the checked document/module;
- the portable type of every expression node;
- the stable `SymbolId` selected for every local reference; and
- minimal capability sets for the whole program, each declaration, and each
  requiring node.

Backends must never accept `Document`, `Module`, or `Expression` directly.
They consume `CheckedProgram` after M08. A compile-fail rustdoc example protects
the private constructor, and a separate Bazel smoke crate proves downstream code
can obtain and inspect a checked value only through `check_program`.

## Deterministic passes

The checker runs these target-independent passes:

1. reject an incompatible IR major or invalid/duplicate structural node IDs;
2. index declaration, field, variant, contract-method, and implementation-method
   node identities;
3. validate portable identifiers and declaration/member scope collisions;
4. resolve and normalize type aliases, rejecting direct or indirect cycles;
5. validate type positions and declaration signatures;
6. type constants, functions, methods, blocks, statements, expressions,
   patterns, and typed portable-test values;
7. check contract implementations and restricted dispatch;
8. prove match exhaustiveness and reject duplicate/unreachable patterns;
9. reject constant and callable dependency cycles; and
10. collect capabilities while checked types and operations are visited.

Independent errors accumulate where continuing cannot corrupt later reasoning.
Invalid structural identities and incompatible IR majors stop checking because
node identity can no longer be trusted. Diagnostics are sorted by source then
stable code before being returned.

## Types and scope

Aliases normalize transitively to their non-alias target. `Named(NodeId)` must
resolve to a record, enum, or alias; `Contract(NodeId)` must resolve to a
contract. Contract views are permitted only as direct function or method
parameters. They cannot be stored in records/containers, returned, constructed,
or compared for equality.

Parameters, immutable `let` bindings, bounded-loop bindings, and pattern
bindings receive stable symbol identities derived from their source node ID.
Shadowing within a function is rejected to keep frontend and backend lookup
unambiguous.

Every successfully checked expression has a normalized portable type. Aggregate
constructors and values must supply every declared field exactly once. Calls
check owner, receiver, arity, and assignability. A concrete record is assignable
to a contract parameter only when an explicit checked implementation exists.

## Control flow and patterns

Non-`Unit` functions and methods must produce their return type through a block
result or an unconditional explicit return. Return values are checked against
the enclosing callable. Statements or block results after an unconditional
return are rejected.

`if` branches must agree after accounting for branches that always return.
Matches accept only booleans, enums, `Option`, and `Result`. They must cover
both booleans, every enum variant, both option cases, or both result cases unless
a wildcard covers the remainder. Duplicate arms and arms following a wildcard
are unreachable. Enum payload patterns bind every field exactly once.

## Intrinsics and purity

All 69 v0 intrinsics have explicit arity, operand, and result rules in the
checker. Integer operations require an unambiguous common width. Operations
named `Checked` retain their value result type; their runtime failure becomes a
structured evaluator outcome in M05. Checked and wrapping arithmetic are
separate capabilities.

`StringScalarLength` and `StringUtf16Length` each have signature
`String -> I64`. The former counts Unicode scalar values. The latter counts
code units in the same scalar string's well-formed UTF-16 encoding, so BMP
scalars count as one and supplementary scalars count as two.

`ListIndexOf` has signature `List<T> × T -> Option<I64>` wherever `T` is
eligible for portable equality. It returns the zero-based first matching index
or `None` and uses structural/IEEE equality rather than target identity or
coercion.

`StringIndexOfLiteral` has the single signature
`String × String -> Option<I64>`. `StringSliceScalars` has the single
signature `String × I64 × I64 -> String`. Neither operation accepts bytes,
floating-point indices, target-native regular expressions, or an implicit
index unit.

`FloatTrunc` has signature `F64 -> F64`. It rounds toward zero while preserving
IEEE signed zero, NaN, and infinities. Portable-test expectation comparison is
separate from `Equal`: tests compare finite F64 bit patterns exactly, accept NaN
as an expected class, and recurse through aggregate values; program equality
continues to use IEEE semantics.

`FloatIsNaN` has signature `F64 -> Bool`. It accepts every binary64 NaN payload
and sign and rejects finite values, signed zeros, and both infinities. The
operation is serialized as `float_is_nan`; target runtimes may use their native
IEEE predicate, but cannot substitute a truthiness or domain check.

`FloatIsNegativeZero` has signature `F64 -> Bool` and is serialized as
`float_is_negative_zero`. It returns true exactly for raw binary64 bits
`0x8000000000000000`. It rejects positive zero, nonzero finite values,
subnormals, infinities, and every NaN payload/sign; it is therefore neither
ordinary IEEE equality nor a general sign-bit predicate.

`StringReplaceAll` has the exact signature
`(String, String, String) -> String`. Its second and third operands are a
literal needle and literal replacement; no target regular-expression or
replacement-template syntax is accepted.

`BytesReplaceAll` has the exact signature
`(Bytes, Bytes, Bytes) -> Bytes`. It scans immutable octets from left to right,
matches the needle literally, and replaces non-overlapping matches globally.
Replacement output is never rescanned. An empty needle inserts the replacement
at every byte boundary, including before the first and after the last byte.

`StringReplaceMany` accepts a source string followed by one or more
needle/replacement string pairs. The total arity is odd and at least three.
It scans the original source from left to right at Unicode scalar boundaries,
uses the first pair whose needle matches at the current boundary, and never
scans replacement output. An empty needle inserts its replacement at the
current boundary and advances by one source scalar; at end of input it inserts
once and terminates. These rules make ordered priority, overlapping needles,
and empty needles deterministic in every backend.

`StringStripPrefix` has signature `(String, String) -> String`. It removes one
exact leading substring when present. A missing or empty prefix leaves the
source unchanged.

`StringTrimStart` and `StringTrimEnd` each have signature
`(String, String) -> String`. The second string is a set of Unicode scalar
values, not a substring or regular expression.

The v0 IR grammar contains no I/O, mutation, global state, allocation hooks, or
unbounded loop node, so an impure operation cannot be represented by a valid v0
`Expression`. The only iteration form is bounded immutable-list iteration.
Direct and indirect recursion are rejected across functions, concrete methods,
contract dispatch candidates, and constant dependencies.

Semantic traversal is limited to 64 nested blocks/expressions/values. Inputs
over the limit produce `P0003` instead of recursing without a bound.

## Capability map

| Capability | Requiring portable feature |
| --- | --- |
| `Bytes` | `Bytes` types or byte operations |
| `CheckedIntegerArithmetic` | checked integer arithmetic, shift, or narrowing |
| `ContractDispatch` | contract types or method dispatch |
| `F64` | binary64 values or operations |
| `ImmutableList` | list types, constructors, patterns, or operations |
| `Option` | option types, constructors, patterns, or operations |
| `Result` | result types, constructors, patterns, or operations |
| `UnicodeScalar` | `Char`, `String`, or Unicode string operations |
| `WrappingIntegerArithmetic` | explicitly wrapping integer operations |
| `BoundedIteration` | `ForEach` statements |

Sets use ordered maps/sets. Program capability output is unchanged when
declaration insertion order changes, and every entry is traceable to its
requiring node and containing declaration.

## Diagnostic categories

M04 extends the stable registry with:

- `P0002` invalid IR structure and `P0003` complexity limit;
- `P0100` invalid identifier, `P0101` unresolved reference, `P0102`
  duplicate declaration, and `P0103` alias cycle;
- `P0207` type mismatch, `P0208` invalid invocation, `P0209` invalid
  control flow, `P0214` non-exhaustive match, and `P0215` unreachable pattern;
- `P0220` contract nonconformance and `P0221` invalid contract position;
- `P0230` invalid portable test; and
- `P0301` impure operation and `P0302` recursive dependency.

Target reserved names, target capability support decisions, evaluation,
optimization, and code emission remain outside this checker.
