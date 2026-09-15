# Rust-source exact signed 64-bit values

- Status: implemented; bounded native, admission and compile-contract evidence recorded
- Task: [M35-03A-02C](../../plan/tasks/M35-03A-02C-i64-values.md)

## Shared compiler boundary

Rustc remains responsible for source parsing, resolution, type checking and
borrow checking. Source literal admission produces a private checked value
whose closed enum distinguishes bool, i32 and i64. Negative integer literals
are interpreted with a wider checked intermediate so that a valid signed
minimum does not first overflow its positive target type. No target emitter
reinterprets source token text or silently narrows a magnitude.

Object and signature mappings admit i64 alongside the existing scalar types.
Comparisons require built-in equal-width integer operands and a bool result;
adjusted references and overloaded operators do not acquire numeric permission.
Supported immutable places, scalar record construction, shared dereference and
direct calls retain their existing identity and evaluation-order contracts.

## C language mapping

Use CScalarType::I64, CSignedLiteral::I64 and registered typed places/signatures.
The shared graph has a distinct standard int64_t symbol whose owner is stdint.h.
Imports are derived from typed dependencies and physically deduplicated.
Structural rendering obtains the type spelling from that resolved symbol.

Render minimum signed values through the target's valid typed literal policy;
do not rely on an out-of-range positive signed C literal or implementation-defined
narrowing. Retain the pinned Linux LP64 ABI. Discover i64 use by traversing
typed source nodes with the dependency pass, and add its two layout checks only
to packages that use it. Ignore existing platform assertions when discovering
that need: an extra assertion cannot authorize its own presence. Both files of
a header/implementation pair contribute to the header-owned requirement. Keep
existing i32/bool-only platform inventories and capacity measurements unchanged.
The actual C package profile requires identical scalar operand kinds for
comparisons and rejects numeric conversions to or from i64. Preserve the
pre-existing Bool/Int/I32-only C normalization nodes (including identity casts
used by capacity proofs); these do not enable Rust source casts. Scalar-call
analysis proves only local-storage effects, not numeric/render admission: its
evidence cannot bypass this package profile. No arithmetic or pointer write is
authorized merely by accepting the scalar type.

## Java language mapping

Add an explicit I64 representation plan that maps to primitive long and typed
JavaLiteral::I64. Do not use boxed Long, floating values, decimal strings or a
runtime wrapper. Preserve the plan through shared-reference erasure and imported
call binding. Declarations and manifests distinguish int from long exactly.

The dependency body grammar admits long literals, immutable locals, parameters,
scalar fields and comparisons. It continues to reject mutable integer locals and
writes. Its comparisons require identical operand types and a Boolean result,
even though Java syntax itself permits int/long promotions. Our general Java
AST already requires exact operand types: public tests must reject mixed widths
there, while direct private body tests independently exercise the dependency
guard without relying on the earlier check. Source/classfile resource accounting includes the long literal spelling,
constant-pool representation and two-slot long parameters/local values; prior
limits cannot be reused as if long occupied one slot.

## Proof and exclusions

Native consumers must exercise the generated public libraries, including
separate Rust-crate dependencies, not only inspect literal text. Require exact
wide-integer output, signed boundary comparisons, strict Java lint and two C
compilers at O0/O2. Keep compile-negative capability/input privacy checks and
atomic unsupported-source rejection. Use corruption controls for width metadata
and the independent oracle so a consistently narrowed implementation cannot pass.

This step does not enable arithmetic, shifts, casts, unit, unsigned integers,
constant declarations, general aliases, associated constants, heap ownership or
public record-valued signatures. Their capability tasks remain separate.
