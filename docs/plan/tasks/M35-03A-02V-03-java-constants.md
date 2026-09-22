# M35-03A-02V-03 — Java signed-infinity constant foundation

- Status: planned
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [C foundation](M35-03A-02V-02-c-constants.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-infinite-f64-constants.md)

## Contract

Register typed Double infinity fields and exact primitive-double constant
inventory values. Preserve declaration/producer authority and structural
rendering. Do not generalize arbitrary initializer expressions or finite literals.

## Definition of done and tests

Native strict separate Java21 compilation observes both signs from owned,
imported and aliased fields/readers under normal/-Xint execution. Compiling
faults are detected after recompiling dependents. Incorrect shape/type/sign,
lookalike producer, resource and byte-bound checks reject. All old output
hashes remain unchanged. Full gate and fresh independent review pass before
separate commit/push. Compiler source admission remains disabled.
