# Checked Rust wrapping signed addition

- Status: planned
- Plan: [02Q](../../plan/tasks/M35-03A-02Q-wrapping-addition.md)
- Targets: [C17](languages/c/rust-wrapping-addition.md), [Java21](languages/java/rust-wrapping-addition.md)

## Semantics and scope

Admit the actual core i32::wrapping_add and i64::wrapping_add inherent operations,
in method or direct associated-function form. For width W, the result is the
signed representative of (left + right) modulo 2^W. The operation is total and
does not depend on debug/release overflow checking. See the official
[Rust wrapping_add contract](https://doc.rust-lang.org/std/primitive.i32.html#method.wrapping_add).

Do not admit ordinary potentially overflowing +, wrapping_sub/mul, saturating or
checked arithmetic, other widths, mixed signedness, explicit casts, generic
callees, borrowed/autodereferenced receivers or user-defined lookalike methods.
Previously supported ordinary functions retain normal direct-call behavior
regardless of spelling. This is one operation, not broad integer parity.

## Compiler boundary

A separate WrappingAddition capability has a private authenticated input with
canonical source expression, original ordered left/right HIR children, actual
core DefId and an exact I32/I64 width enum. Check the original TypeckResults and
HIR identity, safe nongeneric builtin signature and primitive implementation
identity; both operands and result must have the same unadjusted signed type.
A name filter may exclude unrelated calls, but never grants admission.

Each backend registers an executable mapping through its consuming builder;
Supports<WrappingAddition> exposes that mapping, not a marker. Missing,
duplicate, wrong-capability/input/context/output bindings fail compilation.
Private input construction is unavailable to callers; copied nodes and wrong
typechecking owners reject in the witness probe.

Materialize left completely, then right, once each, before target arithmetic.
Preserve original producer identities, source-owned public/private declarations,
module/crate boundaries, documentation and dependency manifests.

## Required proof

The target foundation must be certified and reviewed before compiler admission.
Use independent arbitrary-precision modular truth, boundary/carry/borrow cases,
deterministic full-width samples and actual native Rust behavior. C additionally
requires path-sensitive arithmetic/cast safety and sanitizer evidence.

Addition is commutative: final values alone cannot detect operand reversal.
Actual per-input Rust/C/Java traces and typed AST/dataflow probes must prove
ordered once-only original operands. Detect dropped/duplicated/reversed calls,
wrong-width/truncation and safe wrong-operation/carry faults.

Export real packages and original inputs outside Docker. Keep unsupported-source
atomic diagnostics, all earlier release gates and legacy corpus paths enabled.
No runtime source, wrapper package, helper catalogue call or new dependency is
introduced. Each checkpoint has a full isolated Bazel/lint gate, preserved
older outputs/WIP, clean independent review and its own commit/push.
