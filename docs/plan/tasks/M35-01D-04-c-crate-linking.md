# M35-01D-04 — Separate-crate native proof

- Status: complete
- Depends on: [M35-01D-03](M35-01D-03-c-public-packages.md)
- Parent: [M35-01D](M35-01D-c-files-and-visibility.md)
- Contract: [crate dependencies](../../specification/typed-generation/languages/c/rust-hir-crate-dependencies.md)

## Goal

Prove that two compiler-checked Rust crates become two separate C implementations
which interoperate through their admitted public API.

## Ordered implementation slices

1. [M35-01D-04A — Explicit compiler identity](M35-01D-04A-compiler-crate-identity.md).
2. [M35-01D-04B — Certificate-backed C dependencies](M35-01D-04B-certified-c-dependencies.md).
3. [M35-01D-04C — Checked compiler dependency driver](M35-01D-04C-rust-crate-dependency-driver.md).
4. [M35-01D-04D — Separate-crate integration proof](M35-01D-04D-separate-crate-proof.md).

No child alone proves dependency-crate support. Keep source/target modules focused
and put tests at stable Bazel boundaries rather than growing a driver mega-file.

## Definition of done

- Accept explicit crate identity and compiler dependency inputs through pinned
  Bazel actions; no ambient host crates, source lookups or credentials.
- Extract each crate separately and retain its own checked package. Resolve
  dependency references through typed crate/declaration API inventories.
- Generate stable identity-derived C symbols across separate adapter processes.
  Aliases retain one implementation; foreign bodies are never copied into users.
- Derive dependency-header includes, check exact link/export inventory and reject
  missing, conflicting or private dependency bindings before writing output.
- Refresh visible ignored examples; document command lines, limitations and proof.

## Tests and proof

- Two-crate native Rust oracle versus separately compiled/linked C programs over
  boundary and differential input vectors, with strict compiler/sanitizer matrix.
- Same-spelled declarations in both crates remain distinct; public aliases resolve
  correctly; illegal private dependency calls fail rustc analysis.
- Mutation tests reject cross-crate merging/substitution, wrong manifests,
  private-layout leaks and mismatched dependency signatures.
- Deterministic output, declared-input cache invalidation, all historical release
  tests, style checks and fresh independent review pass.
- Close M35-01D and the pending M35-01B public-boundary obligations, then proceed
  to Java retrofit. Hold pushes until the complete C/Java gate is green.

## Completion evidence

All four child checkpoints are complete. The final
[04D proof record](M35-01D-04D-separate-crate-proof.md) records the actual
four-crate native Rust/C diamond, exact ownership/import/export/doc inventories,
private consumer negatives, atomic bundle publication and clean fresh review.
Final Linux/Bazel gate 5630aeea-7116-4b63-bfcf-5a07969cb6a2 passed all
343 tests, with both ignored bundle examples byte-verified. This completes the
admitted no-heap boundary, not shared-library or arbitrary Rust support.
