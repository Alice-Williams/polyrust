# M35-03A-02M-03 — Checked Rust truncation integration

- Status: complete
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

## Implementation sequence

1. Establish a private canonical-HIR witness against the pinned compiler's
   actual standard-library owner, not a crate-name string. The published Rust
   source places inherent trunc in std; verify that with the pinned compiler
   rather than copying the earlier core-owned abs anchor.
2. Register executable mappings in both typed builders; materialize one
   receiver and use the already certified C/Java target foundations.
3. Carry certificate-derived C system-library requirements into a new manifest
   schema only where nonempty, preserving older bundle bytes. Reconstruct and
   verify the metadata at import/publication; include it in byte reservations.
4. Add original three-crate Rust inputs, native value/trace oracles, AST/private
   witness probes, single-fault compile-negative registrations, unsupported
   source/metadata rejection controls and exported actual examples.
5. Run the full Linux release/lint gate and independent review loop before
   marking complete or checkpointing.

## Implementation and proof

FloatingTruncation is an executable builder slot in both backends. Its private
TruncationInput authenticates canonical HIR, original TypeckResults, primitive
owner/signature and a nonlocal begin_panic language-item standard-library crate
anchor. The pinned compiler confirms that the inherent trunc method is std-owned.
Both source-call inventory walks preserve nested ordinary receiver calls.

The C mapping produces the existing FloatTruncate catalogue call. Java produces
a typed sign comparison and MathCeil/MathFloor conditional. Both materialize the
original receiver once. No runtime file, raw target fragment or handwritten
production import is added. C schema 9 carries certificate-derived local and
transitive system-library requirements; older empty-library encodings are
unchanged.

The focused probe tree eb41326f780d221a18349a31dde2ce1cefa13a1b passed all
14 single-fault mapping compile negatives, both target AST/private-witness
observations, 48 atomic unsupported-source checks and fixture Clippy.
Invocation: 6802fde9-dd81-47ae-a328-1d8ef35474fa.
The AST proof reconstructs receiver dataflow independently, rejects copied HIR
and wrong Typeck context, and detects disconnected results even when original
call declarations remain present. Probe-enabled and production output bytes
must match.

Two native harness issues were fixed rather than weakening checks: the shared
clients now accept the explicit expanded oracle corpus, and the deliberately
substituted C standard-call mutant includes its own math header. The original
transitive-only middle crate must still have no math header. The native proof
on f6dbc5e8acc8578fcc967cdb2e42341bdcf06ec9 passes 1,170 independent expected
results per Rust/C/Java run (146 input bit patterns, eight operations and two
literal results). GCC14/Zig O0/O2 and Java21 strict lint compile original owners
and consumers separately. Seven compiled value/trace faults are detected;
ordinary-function substitution and duplication preserve observed values but
change the independent call trace. Omitting the metadata-derived math link
fails under GCC with builtins disabled.

Manifest probes remove required Math or inject it into a no-library package,
then require both binding reconstruction and owner verification to reject.
They check the exact 48-byte bound contribution and actual encoded size.
A schema-9 fixture also includes Boolean and unit-result signatures.
Serialized consumer controls reject missing, empty, unknown, duplicate and
extra library requirements.

The first complete release run exposed a missing backend-independent positive
capability probe. The new witness/capability signature use was added without
suppressing strict unused/dead-code checks. That first full run passed 855 tests;
only the three targets using this probe failed to build.

## Completion

Reviewed implementation tree: 64a164449233a5b3d4e11c6229da427fc448be9a.
The full Linux Bazel release gate passes all 858 test targets, including Rust
formatting/Clippy, Bazel lint, source policy, existing generated-language and
real-world regression suites. Invocation 512bc639-7dda-4b1f-8ee2-a683cec435bd,
109.181 seconds (7 executed, 851 cached); no disabled tests.

Two independent reviews found no core production or required-proof defects.
The proof review checked the final exact implementation tree, including the
positive-probe correction, schema-9 Boolean/unit signatures and test-only mutant
header. No findings were waived as optional features.

All 138 recorded file hashes across 18 earlier C/Java source bundles match;
the 19 preserved ownership-work file hashes are unchanged. Actual unmodified
Rust inputs, C/Java packages and handwritten consumers are exported locally at
`generated/examples/floating-truncation-64a16444/README.md` (ignored artifacts,
not committed generated output). Truncation parity is complete; broader scalar
and runtime replacement parity remains open.

This is a locally tested checkpoint, not a claim that unpublished GitHub CI
has passed. Publishing remains subject to the previously reported approval block.
