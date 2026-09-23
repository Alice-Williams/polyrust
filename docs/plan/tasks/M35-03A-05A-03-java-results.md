# M35-03A-05A-03 — Certified Java scalar-result transport

- Status: planned
- Parent: [05A](M35-03A-05A-scalar-results.md)
- Depends on: [C result transport](M35-03A-05A-02-c-results.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-scalar-results.md)

## Contract

Use ordinary source-derived closed result types and immutable primitive success
payloads. Keep each canonical source instantiation distinct; do not use shared
Runtime.Result, raw Object, null, unchecked casts or exception sentinels.
Use existing typed declaration, interface, constructor and dependency APIs.

## Tests and definition of done

Strict separately compiled Java21 producer/relay/consumer packages match tags,
payloads, fallback and ordered effects in normal and interpreted execution.
Wrong variant/type/owner/imports, illegal payload access and unregistered
subtypes reject; external consumers cannot invent new variants. Prove resource
and JVM limits, original source provenance and no runtime artifact. Full Linux
release/lint, fresh review and separate commit; no compiler admission.
