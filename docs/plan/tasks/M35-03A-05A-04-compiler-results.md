# M35-03A-05A-04 — Checked scalar-result source integration

- Status: planned
- Parent: [05A](M35-03A-05A-scalar-results.md)
- Depends on: [Java result transport](M35-03A-05A-03-java-results.md)
- Specification: [shared](../../specification/typed-generation/rust-scalar-results.md)

## Contract

Add compiler-authenticated nominal instance and variant facts to the bounded
source/dependency model. Executable typed bindings support signatures, moves/
copies, local bindings, Ok construction, Err reconstruction from an existing
opaque error, and exhaustive two-arm match/returns in the closed instance.
Materialize the scrutinee once and preserve exactly the selected arm's effects.

The Rust native driver may supply an error made with standard checked narrowing;
this driver is outside translated code. Production narrowing remains 02Y-04.
Thus result integration can precede conversion admission without a new cycle.
Do not enable arbitrary error construction, error formatting, generic calls,
other Result/Option instances or heap payloads.

## Tests and definition of done

Multi-crate Rust/C/Java prove success and error paths, equal-valued distinct
nominal instances, opaque error forwarding, guard-scoped payloads, original
identity/imports/docs/privacy and function result transport. Typed variant/
instance/callee/scrutinee forgeries and unsupported source reject atomically.
Measure real producer-change invalidation/restoration, export actual packages,
preserve old outputs/WIP, pass full Linux release/lint and fresh broad reviews.
