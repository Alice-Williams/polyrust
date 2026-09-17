# M35-03A-02M-03 — Checked Rust truncation integration

- Status: planned
- Parent: [truncation](M35-03A-02M-floating-truncation.md)
- Depends on: 02M-01 and 02M-02

## Contract

Add a private canonical-HIR TruncationInput and executable FloatingTruncation
mapping in both capability builders. Authenticate the pinned compiler's actual
standard inherent f64::trunc identity, original typeck, exact signature, receiver
and lack of adjustments. Names alone grant no authority. Materialize the
receiver once; ordinary free functions named trunc remain ordinary calls.

C manifests must serialize the certificate-derived system-library closure.
Authenticate it on import/publication and include it in resource reservations.
Keep earlier no-library bundles byte-identical when introducing the new schema.
Native consumer build rules obtain required link options from this metadata.

## Definition of done and tests

Three original Rust crates produce C/Java packages with native independent
value/trace equivalence, typed mapper AST/dataflow probes, missing/wrong
capability compile failures and atomic unsupported-source rejections. Include
ordinary-function identity controls whose changed call traces make substitution
and duplication observable even when values coincide. Test transitive math
linking and malformed/missing/extra manifest requirements. Export actual
examples, preserve existing bundles and pass full Linux release/lint gates.
Evaluate findings and obtain a clean fresh independent review before completion.
