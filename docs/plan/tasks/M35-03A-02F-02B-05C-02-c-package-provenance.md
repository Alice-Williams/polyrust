# M35-03A-02F-02B-05C-02 — Independent C source-package provenance

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-01
- Specification: [C constant re-exports](../../specification/typed-generation/languages/c/rust-constant-reexports.md)

## Contract and scope

Store the selected Rust crate's finite export/documentation graph as a distinct,
private-field CSourcePackage registration associated with the exact public header.
It is unresolved provenance, not a certificate or a source declaration. Freeze
it with the registry and reconstruct its checks during target certification.

Production public Rust-to-C lowering registers this metadata, including
constant-only packages. Existing declaration-derived C fixture/caller inputs
remain compatible when no explicit registration is present. When present it is
authoritative: no fallback may hide a disagreement. Reconcile every owned source
registration, not just the first definition, and route module docs without
inventing a function/object origin.

This checkpoint does not admit alias-only render-ready packages, certify foreign
exports, alter bundle schemas or remove legacy runtime files. Those require the
next target-export evidence and compiler/publication checkpoints.

## Definition of done and tests

- Typed registration rejects wrong file roles, cross-registry references and
  replacement; read-only frozen metadata retains exact graph and file identity.
- Projection/certification rejects conflicting roots, owned origin graphs,
  missing/foreign file ownership, malformed/unreachable/over-budget module graphs
  and inconsistent ancestry/docs. Existing graph/documentation budgets apply.
- Documentation normalization works with an empty owned-declaration inventory,
  but the full profile still rejects empty/foreign-export-only publication.
- Existing nonempty function/constant packages certify and render unchanged;
  declaration-derived compatibility stays covered. Public source lowering uses
  explicit metadata with no raw C syntax, synthetic object or runtime added.
- Native GCC and Zig O0/O2 consumer tests, compiler frontend regressions, complete
  Linux Bazel release/lint gate and fresh independent review pass.
- Record exact staged-tree evidence, preserve unrelated work, commit and push
  this checkpoint alone.

## Evidence collected

- Focused Linux C unit/native and typed compile-fail Bazel targets passed on
  staged tree `62d0662da627f7f4da148d6526393342dadaa40d` (2/2 targets,
  invocation `a35f96ff-f345-4bc7-8f6e-aa317d4801f8`).
- The full-gate C unit suite on implementation tree
  `a077de7bc8980a8db44c66d07163dc86f45aa240` passed 753 cases with zero
  failures/ignored cases; five existing capacity cases run in separate targets.
  This includes GCC and Zig O0/O2 native consumers, changed-value negative
  oracles, ten new provenance/graph/reconstruction tests, and unchanged-output
  comparisons for explicit versus declaration-derived packages.
- Actual generated header/source/independent-consumer examples are exported to
  ignored host directory `generated/examples/c-source-package-a077de7/`,
  under `owned-constants/ConstantsOnly` and `owned-constants/Mixed`.
- First independent Sol Extra High review found no core defects. A proposed
  shared-Arc repeated deep-comparison defect was evaluated and withdrawn:
  `RustCrateExports: Eq` selects Rust's identity-short-circuiting Arc equality;
  production lowering retains the same allocation. See the
  [standard library implementation](https://doc.rust-lang.org/src/alloc/sync.rs.html).
  Distinct equal graph allocations, eliminating a repeated registration scan,
  and adding a separate function-only fixture are optional improvements, not
  unproven ownership/certification paths. Mixed and real compiler packages
  already exercise function mappings.
- The next [file-requirements task](M35-03A-02F-02B-05C-03-file-requirements.md)
  records the necessary source-to-header edge for alias-only packages.
  It is planned, not implemented or enabled by this checkpoint.
- Complete Linux Bazel `test //... //:release_gate` passed on implementation
  tree `a077de7bc8980a8db44c66d07163dc86f45aa240`: 1093 targets,
  738/738 tests (93 executed, 645 cached), 919.357 seconds; invocation
  `316be814-f9b9-469a-9073-60bb044b5339`. Rust/Bazel lint and all native,
  frontend, legacy conformance and capacity gates stayed enabled.
- A fresh independent Sol Extra High review found no core errors or material
  proof gaps. Its optional body-local-origin mutation suggestion is redundant
  with the exhaustive registration traversal and existing unused-owned-function
  negative; it is not a missing proof branch.
- All 20 protected ownership files retain their original SHA-256 hashes.
  Generated examples remain ignored and are not committed.
