# Rust scalar results in Java21

- Status: planned
- Contract: [shared](../../rust-scalar-results.md)

Use an ordinary source-derived sealed result interface with an immutable
success record carrying primitive int and a payload-free error implementation.
Register the exact permits/implements, constructors, fields and dependencies
through typed Java AST APIs. The error implementation represents only the
admitted opaque standard error observations, not a general singleton type.

Each canonical source instantiation owns distinct target symbols. Do not share
Runtime.Result, use raw Object/null as a variant, throw to represent Err, or
add unchecked payload casts. Match lowering uses certified variant knowledge
and evaluates the scrutinee once and only the selected arm. Foreign null is not
a Rust result value and must follow the established public boundary policy.

Incidental Java object allocation is not support for Rust-owned heap payloads.
Prove closed variants, immutable exact payloads, original import authority,
private implementation details, JVM capacity and strict separately compiled
normal/interpreted consumers before source admission.
