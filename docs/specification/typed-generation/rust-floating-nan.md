# Checked Rust binary64 NaN classification

- Status: implemented and verified for standard inherent f64::is_nan
- Plan: [M35-03A-02K](../../plan/tasks/M35-03A-02K-floating-nan.md)
- Targets: [C17](languages/c/rust-floating-nan.md), [Java21](languages/java/rust-floating-nan.md)

## Source contract

Admit the standard inherent f64::is_nan by-value method and associated-call
forms. Return bool, true exactly for NaN, false for either infinity and every
finite value including signed zeros and subnormals. Evaluate the original
receiver exactly once. The contract follows the
[Rust primitive API](https://doc.rust-lang.org/std/primitive.f64.html#method.is_nan).

FloatingNaN is an executable capability, not a marker. Its private NaNInput
retains canonical HIR source/receiver, actual compiler DefId and original
TypeckResults ownership. Authentication precedes type queries. Prove the
callee belongs to the pinned core crate using a compiler language-item anchor,
is an inherent nongeneric f64 method, and has exactly safe Rust ABI (f64)->bool.
Reject implicit receiver/result/callee adjustments, trait methods, custom
same-named methods and other floating widths. Ordinary free functions with the
same spelling remain ordinary DirectCalls; name matching never grants support.

## Target-independent obligations

Each mapping validates the witness against its Reader, lowers the receiver
once and stores it in a typed local. Both comparison operands read that exact
local. The primitive target expression is local != local, with a bool result
(normalized from C int). No custom runtime, bit cast, boxed wrapper, library
import, detached callable identity or raw source fragment is necessary.

This contract observes returned classification and source evaluation order,
not NaN payload/sign or floating exception flags. The existing default
nontrapping binary64 environment remains required; fast-math is excluded.
is_finite, is_infinite, sign queries, bit conversions, general floating
arithmetic and floating constants are not enabled by this capability.

## Required proof

Independent integer exponent/fraction masks determine classification.
Compare original Rust and separately compiled generated C/Java over finite
boundaries, signed zeros, subnormals, infinities and both NaN signs including
quiet/signaling encodings. Trace receiver calls and reject zero/double
evaluation faults. Wrong equality and constant-false mutants must fail.

Read-only compiler/target probes authenticate method identity, original
receiver dataflow, exact F64/Double local reuse, bool result and target
precedence. Deliberately disconnect a call result without removing the call;
the independent dataflow oracle must reject it. Instrumented output bytes
must match production. Missing/duplicate/wrong mapping signatures and forged
input fail exact compile contracts. Unsupported valid Rust rejects atomically,
preserving existing directories. Retain original cross-crate certificates and
export actual examples outside the container. Full Linux release/lint gates
and fresh independent review precede the scoped commit/push.
