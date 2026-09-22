# M35-03A-02U-04 — Checked finite-constant source integration

- Status: complete
- Parent: [02U](M35-03A-02U-finite-f64-constants.md)
- Depends on: [Java foundation](M35-03A-02U-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-finite-f64-constants.md)

## Contract

Extend the distinct compiler constant domain with FiniteBinary64. Check f64
type/eight-byte representation before decoding bits, retain all original
definition/context/export/owner checks and map through existing executable
constant capabilities. Preserve signed zero and exact original producer values.

## Definition of done and tests

Original multi-crate Rust and normal C/Java packages agree on exact bits for
private/public/local/inherent constants, aliases and imported reads. Test
compiler-evaluated finite expressions without widening runtime admission.
Original APIs/docs/privacy and declaration identities remain intact. Typed
probes and negative contracts reject wrong domains, types, values, owners and
lookalike dependencies. Nonfinite/f32/generic/trait/alias-type/borrowed cases fail
atomically. Actual Bazel producer changes invalidate affected consumers and
restoration restores output; include positive/negative zero changes. Detect
compiling value faults, export real examples and update partial-only inventory.
Full gate, preservation and fresh independent review must pass before commit/push.

## Implementation sequence

1. Add the finite witness variant to the closed compiler constant domain;
   decode only compiler-confirmed f64/eight-byte scalars and fail closed on
   nonfinite bits. Exhaustively extend existing constant mappings and typed
   probe matches; do not merge constant inputs with literal inputs.
2. Extend C and Java descriptive constant manifests with the same lossless
   representation: scalar `f64`, value `0x` plus sixteen lowercase hexadecimal
   representation digits. Keep old Boolean/integer encodings byte-identical.
   Manifest strings remain descriptions, never dependency authority.
3. Add original Rust fixtures for public/private/local/inherent finite constants,
   constant-evaluated expressions, identical values under distinct definitions,
   signed zeros and cross-crate re-export/import chains. Preserve source docs,
   privacy and defining identities in both generated package inventories.
4. Exercise actual target ASTs through the existing constant probes, extending
   their finite-value matches. Challenge wrong values (including zero sign),
   types and producer certificates at the join. Keep unsupported cases atomic;
   replace obsolete finite-f64 rejection fixtures with f32 boundaries and add
   separate positive coverage for the newly admitted forms.
5. Compile the original Rust and separately compiled generated C/Java packages;
   compare external raw-bit observations with the independent integer oracle.
   Recompile Java dependents after producer mutations. Test GCC/Zig O0/O2,
   GCC undefined-behavior sanitization and Java21 normal/interpreted execution.
6. Run an isolated archive-based Bazel cache experiment: warm baseline, finite
   producer-value and zero-sign changes, affected action/output evidence,
   old-oracle rejection, updated-oracle acceptance and restored cached baseline.
   Never mutate the developer checkout or checked-in fixtures for this proof.
7. Export actual generated examples to the ignored host examples directory;
   update the partial capability inventory, run the full gate and review loop,
   verify prior output/WIP hashes, then commit/push this checkpoint separately.

## Implementation and focused evidence

The checked constant domain now has F64(FiniteBinary64); both existing mapping
families exhaustively handle it. C and Java manifests retain all sixteen hex
digits and both zero signs. Ordinary generated source still owns declarations;
no runtime, renderer, dependency or support-only capability has been added.

Four original crates cover 22 original dependency constants plus one root-owned
constant, alias-only and mixed owners, equal values with distinct declarations,
private/local/inherent reads and six compositions with previously supported
floating operations. Focused native proof passes 34 Rust reads and 62 target
observations per configuration, plus three compiling producer-value faults.
Typed probes observe 23 imported reads, three folded reads, two local
declarations, and a separate four-public-declaration/five-read fixture per target.
Wrong declaration/type/value/owner certificates and stale zero-sign metadata
are rejected atomically; 80 unsupported-source publication controls pass.

The first full gate ran all 1,001 tests: 999 passed, while Buildifier found two
unnamed macros and an older integer fixture exposed an over-broad call-depth
assertion. Both are corrected without weakening or disabling the older tests.

Independent whole-scope review found one core defect: constant-only manifests
still selected pre-binary64 schema versions. Accepted and fixed by deriving the
version from typed owned/imported/exported constants as well as signatures.
The native inventory now independently asserts C 8 (9 with math linkage) and
Java 6 for every owner, with downgraded-version mutation controls.

The initial isolated cache experiment proved finite-value invalidation and
restored cached results. Its zero-sign run caught a test-control defect: a
fixed negative-zero corruption ceased to be a corruption after changing the
original producer to negative zero. The control now flips the expected sign
bit. The complete experiment is being repeated from a corrected archive.

The corrected full gate passes 1,001/1,001 targets, invocation
`855b6499-17d1-4b67-b4bf-73e17433689d` (70 executed, 931 cached). The final
negative-control/specification tree `d5538ff590486f569be4322846bb03d16c7d65c3`
also passes all 1,001 targets, invocation
`0f203eb4-6c40-4e91-a4f2-9f8e68894f12` (4 executed, 997 cached).

Fresh independent Sol Extra High review of all 59 changed files in that tree
found no further core defect. Its one optional hardening suggestion concerns
scanning registered rather than serialized-used C constant imports for schema
selection. This is deferred: current source imports are either body-used or
exported (both inventories are covered), there is no demonstrated source
counterexample, and choosing the newer descriptive schema cannot weaken opaque
producer authority. All 423 prior generated file hashes remain byte-identical.

Actual four-package C/Java artifacts and original Rust fixtures are exported to
the ignored host directory `generated/examples/finite-constants-d83f30ac/`.
All 39 exported generated files match their Bazel artifact hashes. All 38
unrelated WIP hashes still match their preserved baseline.

The isolated archive cache experiment on implementation tree
`d83f30ac3c8ed64307e57e623079550812e1279b` passes completely. Both finite-value
and zero-sign changes execute exactly seven affected metadata/bundle actions;
the independent second producer stays cached and byte-identical. Old expectations
fail through uncached native tests, updated independent expectations pass, and
both restorations recover every original output hash and reuse cached passing
native tests. Receipts are retained under the ignored directory
`generated/m35-checkpoints/finite-constant-cache-d83f30ac/`.

Documentation-only closure is gated again before the separate commit/push.
This completes finite-f64 constant integration, not wider scalar parity or
permission to remove legacy runtimes and consumers.
