# M35-03A-05A-04E — Checked Result HIR operations and equivalence

- Status: planned
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [owner graph](M35-03A-05A-04D-type-owner-graph.md)
- Specification: [scalar results](../../specification/typed-generation/rust-scalar-results.md)

## Contract

Open only the specified Result<I32, standard narrowing-error> source domain,
using checked compiler-session instance/variant/field facts and frozen target
owner bindings. Support signatures, moves/copies, local bindings, Ok construction,
forward/reconstruct an existing Err, returns and exhaustive two-arm matches.
Forward the full original error-kind value using the 04A-03 witness and version-2
target profiles, never recreate a payload-free or default error. Public input
values need not originate from the narrowing operation used to identify the type.

Evaluate the scrutinee exactly once and only the selected arm. Error creation
in a native Rust test driver does not admit narrowing in translated code.
Reject other instances, arbitrary opaque-error construction/inspection, generic
traits, formatting, reference/heap payloads and unsupported match forms.

## Implementation and definition of done

1. Specify each new checked capability input, output and executable C/Java binding;
   use enum variants and private typed witnesses, not string operation IDs.
2. Extend bounded source facts and cross-crate signature reconciliation. Check
   guard-scoped payload bindings and original instance/variant ownership.
3. Implement structured target lowering through existing typed family operations;
   no renderer semantics, copied runtime, raw Object or unchecked payload casts.
4. Native multi-crate Rust/C/Java agree for both variants, successful zero, extrema,
   opaque error forwarding, cross-producer arguments/returns and measured effects.
5. Reject forged variants, fields, scrutinees, callee signatures, stale owners,
   unsupported source and resource overflows before atomic output publication.
6. Include compiling fault controls, actual generated examples and producer-change
   invalidation/restoration. Preserve prior outputs and unrelated WIP.
7. Full Linux Bazel release/lint plus independent broad GPT-6-SOL reviews for both
   lowerings precede scoped commit/push and closure of the compiler-result parent.

This closes one no-heap Result instance, not general enums, collections or the
whole Rust-source migration.
