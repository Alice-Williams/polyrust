# M35-01E-05 — Native equivalence and migration closure

- Status: complete
- Parent: [M35-01E](M35-01E-java-rustc-retrofit.md)
- Depends on: M35-01E-04

## Goal

Prove the complete admitted Rust-source Java path and close the C/Java migration.

## Definition of done and tests

Execute the following focused closure checkpoints in order:

1. [E05A — Fixture matrix and examples](M35-01E-05A-java-fixture-closure.md).
2. [E05B — Isolated proposed commit trees](M35-01E-05B-migration-commit-trees.md).
3. [E05C — Verified milestone commits and push](M35-01E-05C-migration-checkpoints.md).

- Generate every admitted single-crate fixture and the real four-crate diamond
  into Java. Run the same 8,204-input corpus against native Rust, generated C's
  existing native matrix, and Java 21 with warnings treated as errors.
- Independently reconcile actual public/private declarations and binding
  inventories; reject every private helper/layout through handwritten consumers.
- Check all deliberate docs, alias identity, same-spelled declarations, field
  order, branch polarity, noncommutative nested calls and unchanged outputs.
- Exercise compiler negatives and target capacity/registry/witness/resource
  mutations, not only golden source text.
- Produce byte-verified ignored Java artifacts in the host workspace and
  document exact generation/test commands and remaining unsupported categories.
- Run the full release/frontend/C/Java/shared/lint/docs gates with no disabled
  tests, then fresh broad independent review loops until core findings close.
- Keep C and Java milestone commits distinct. Before any push, isolate
  unrelated dirty work and verify the actual proposed committed tree; dirty
  workspace tests alone do not certify that tree.

## Scope boundary

This closes the no-heap Rust-source migration, not arbitrary Rust translation,
heap/destructor support or a formal equivalence theorem for every program.

## Closure

E05A's complete fixture matrix/examples and fresh review loops passed. E05B
verified isolated exact C and Java trees, including all historical tests and
lint/documentation gates. E05C published separate commits with matching tree
identities and preserved the unrelated local ownership work. See those child
records for gate IDs, review outcomes, commit IDs and the verified remote ref.
