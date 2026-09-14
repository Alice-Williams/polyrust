# M35-01E-04D — Bounded atomic Java bundle publication

- Status: complete
- Parent: [M35-01E-04](M35-01E-04-java-crate-bundles.md)
- Depends on: M35-01E-04C

## Implementation contract

Execute focused checkpoints in order:

1. [M35-01E-04D-01](M35-01E-04D-01-bounded-directory-trees.md): reusable atomic staging with bounded nested paths.
2. [M35-01E-04D-02](M35-01E-04D-02-java-bundle-inventory.md): typed Java manifest projection and pre-render resource reservation.
3. [M35-01E-04D-03](M35-01E-04D-03-java-bundle-proof.md): production CLI/Bazel integration and atomic/native proof.

- Render every certified owner before publication. Keep each canonical Java
  package path and public compilation-unit basename; never rename a certified unit.
- Reuse the existing bounded staging and Linux atomic no-replace directory
  publication mechanism. No credentials, partial destination or overwrite path.
- Admit 1..1024 owners, at most one Java source and one descriptive owner manifest
  per owner plus one bundle manifest (2N+1 payload files), and at most 256 MiB total bytes.
  Reject duplicates, unsafe paths and limit overflow before writing the destination.
- Canonical Java package directories are separate bounded staging metadata:
  at most N+6 directories below the stage for N owners, with depth at most seven.
  Track only transaction-created files/directories for cleanup; do not traverse or
  recursively remove an existing destination.
- Descriptive inventories expose roots, aliases, docs, owned/public functions and
  used dependency owners. Serialized JSON is never accepted as a certificate.

## Definition of done and tests

- Exact path/count/byte boundaries and malicious duplicate/unsafe paths reject.
- Existing and late-created destinations survive unchanged; concurrent publishers
  have one complete winner. Failed staging leaves no partial published bundle.
- Four-crate Java source bundles compile/run independently. Illegal private
  consumers reject and public API/docs/alias inventories match compiler facts.
- Full migration gates and fresh broad independent review pass before parent E04
  completes; E05 still owns final native closure, examples and committed-tree proof.

## Closure

D01, D02A/B and D03 are complete with clean independent reviews. Production
CLI/Bazel bundle equivalence, complete compiler-backed public/private JSON
inventory, native Java/Rust results and atomic failure/race tests pass in final
gate `49c0e966-89a7-4dc4-8b25-640149f83724` (370 tests, 470 targets).
The final fresh D03 reviewer found no core findings after the documented proof
correction. E05 remains required; no committed-tree or push claim is made here.
