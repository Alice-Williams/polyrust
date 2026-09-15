# Checked Rust scalar constant reads

Status: implemented and verified for the bounded M35-03A-02F-01 scope.

## Shared boundary

ScalarConstants consumes a privately constructed ConstantInput. A constant
path must resolve through TypeckResults to Const or a nongeneric inherent
AssocConst DefId. Reject generic arguments, trait-associated unresolved
identities, adjustments, nonconstant paths and values other than bool/i32/i64.
The input retains its compiler-origin expression and resolved definition.

For associated constants, also normalize the defining inherent impl's self type.
Require a nongeneric nominal type or primitive owner. A concrete impl can have
zero generic parameters while its nominal self type still carries instantiated
type/const/lifetime arguments. Reject those semantic arguments, including
omitted defaults; checking only the expression's node_args is insufficient.

Evaluate the definition with the pinned rustc constant evaluator. Require the
evaluated scalar's exact width/type before decoding; failure is a diagnostic,
not an unwrap, truncation or target approximation. A closed value enum carries
the resulting literal to the backend. Constant names are never dispatch keys.

The fixed compiler invocation forbids long_running_const_eval, so source
attributes cannot opt out of its termination guard. Compiler rejection and
constant-query errors must happen before package publication. This is not a
claim of a universal wall-clock bound on arbitrary Rust compilation.

See [rustc constant evaluation](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html#method.const_eval_poly)
and [the constant-evaluation lint](https://doc.rust-lang.org/rustc/lints/listing/deny-by-default.html#long-running-const-eval).
These describe the APIs; the pinned toolchain and executable tests are authoritative.

## C17 mapping

Map checked Bool/I32/I64 values through the same ordinary CLiteral and
CSignedLiteral nodes as source literals. Keep exact minimum spelling,
dependency-driven headers and platform constraints. No static runtime, macro
interpreter, addressable backing object or source fragment is introduced.

## Java 21 mapping

Map checked values to primitive boolean/int/long literal AST nodes with exact
TypePlan types. Do not box or parse a textual number. Use the same verified
minimum-value representation and resource checks as ordinary literals.

## Declaration boundary

This first step folds scalar reads only. A public constant is an API member:
silently dropping its exported name is forbidden. Existing public-export
certification must reject that unmapped shape until 02F-02 implements real
constant declarations/references and documentation. Block-local const items,
generic constants, trait dispatch, constant borrows and other types remain
unsupported in 02F-01. Evaluating a constant initializer does not enable its
operations in generated runtime function bodies.

## Evidence

Independent native truth must cover values beyond floating-point precision,
computed and same-spelling constants, bools, import-name aliases, primitive/inherent
associated values and two-crate function use. Actual mapper probes retain
source identity and check exact literals; production artifacts stay identical
with probes enabled. Compile contracts and atomic errors guard the input,
registration, unsupported declarations and evaluation budget.

The completed [02F-01 evidence](../../plan/tasks/M35-03A-02F-01-constant-reads.md)
records native/AST/mutation/contract proof, 68 atomic rejection checks, the
post-review generic-owner repair and the isolated full Bazel/lint gate.
