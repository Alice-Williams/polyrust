# M35-03A-02F-02B-05C-08 — Constant alias end-to-end closure

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-07
- Specifications: [shared](../../specification/typed-generation/rust-constant-reexports.md),
  [C](../../specification/typed-generation/languages/c/rust-constant-reexports.md),
  [Java](../../specification/typed-generation/languages/java/rust-constant-reexports.md)

## Contract

Close the bounded bool/i32/i64 public constant import/re-export proof. Reuse the
compiler, target-certificate and atomic-publication gates from child 07. Do not
add a second lowering path or treat manifests as authority. Parent closure must
identify concrete evidence for every promised control, including the source
mutation and actual Bazel dependency invalidation that direct adapter tests
cannot prove.

## Implementation order

1. Map parent exit criteria to existing compiler inventory, target certificate,
   bundle reconstruction, native execution and mutation tests. Fill specific
   missing cases rather than duplicating complete suites.
2. Exercise a real two-producer, alias-only intermediate and mixed root graph
   through Bazel. Record an unchanged warm result, change one producer source in
   an isolated checkout, rebuild and prove affected Rust metadata, C/Java bundle
   and native truth actions invalidate. An unrelated producer action should stay
   cached. Restore the source and prove the original truth passes again.
3. Ensure constant identity/type/value checks, unsupported foreign kinds and
   private exports, missing/replaced owners, alias binding tampering and schema
   rejection have active negative controls. Do not invent a manifest parser if
   production does not accept manifests; test the actual supported boundary.
4. Export real generated bundles and independent consumers into ignored local
   examples. Check file separation, docs, one producer definition per constant
   and no runtime files, alias copies, wrappers or synthetic source functions.
5. Run fresh independent Sol Extra High review and full Linux Bazel release/lint
   gates. Record exact trees/invocations and the bounded supported subset.
6. Reconcile and close 05C, 05, 02B and 02 only where every required exit criterion
   is proved. Leave broader parity/runtime retirement tasks open.

## Definition of done and tests

- Existing child 07 strict native Rust/GCC/Zig/Java tests and publication mutation
  tests remain enabled and passing.
- Recorded Bazel invalidation distinguishes affected actions from unrelated
  cached work; fresh generated values fail the old independent oracle and pass
  the updated oracle. Restoration passes the original oracle.
- Every parent proof obligation links to a concrete gate or an explicit remaining
  blocker; no aggregate completion based only on text comparisons.
- Generated originals and handwritten consumers are inspectable locally but not
  committed. No caches, compiler outputs or image archives enter Git.
- Independent review has no unresolved core finding. Full release, Rust and
  Bazel lint gates pass on the exact scoped tree before commit and push.

## Proof map

All short target names below are in `//experiments/rustc-frontend`.

| Obligation | Active evidence |
| --- | --- |
| Checked foreign identities, diamonds, aliases, visibility, unsupported kinds | `foreign_export_inventory_test`, `foreign_export_private_test` |
| Combined expression/export identity union and bounded discovery | `constant_import_limits_test`: overlap, reversal, 4096/4097, traversal/depth |
| Original producer authority and certified target alias views | C and Java backend unit/compile-fail suites; child 04 and child 06 receipts |
| Exact schema, all alias bindings, original paths, scalar values, docs | `constant_export_native_test`, `java_bundle_test`, C manifest mutation probe |
| Missing/replaced owners, type/value/declaration faults, atomic outputs | `constant_export_publication_test`, `constant_import_publication_test` |
| Readonly public values, no alias storage or fake source methods | `public_constant_native_test`, `public_constant_bundle_native_test`, `constant_export_native_test` |
| Local/private constants, shadowing, original binding ownership | `constant_ast_test`, `local_constant_ast_test`, their contract/native matrix tests |
| Actual Bazel invalidation, unrelated cached producer, independent old/new truth | Manual Linux `test/constant_export_cache_proof.py` against an exact Git archive |
| Regression, native toolchains and linters | `//... //:release_gate` |

## Schema boundary clarification

Bundle manifests are generated descriptions, never accepted as compiler or target
authority. Production accepts checked Rust source/configuration and original
in-memory producer certificates; it has no API for loading an arbitrary bundle
manifest into those certificates. Versions are selected by closed serializers.
Unknown emitted versions must fail the independent artifact validator, but an
imaginary input-manifest parser is neither needed nor claimed. Certificate and
retained-projection tampering must fail the existing actual publication boundary.

The manual cache gate retains Bazel build events, relevant execution logs,
source-output hashes and a compact summary in an ignored caller-selected receipt
directory. It changes only files inside a freshly extracted temporary archive,
restores the source and independent oracle in a finally block, and leaves the
working checkout unchanged. Cache hits are allowed throughout: invalidating an
action does not prohibit reusing a previously cached result for its new key.

## Completion evidence

- Exact implementation tree `5c6a15524ef3c57c8e1a920db4504ee6dbdcbf47`
  passed Linux dev-container Bazel `//... //:release_gate`: 1,110 targets,
  **741/741 tests**, 3 executed / 738 cached, 30.302 seconds; invocation
  `dd1cfac8-e756-4174-9d78-e0d815670d80`. Earlier implementation
  `fbfdfc179467a85c6c3866fb9b0dcdd0e125a11f` passed the same 741 tests,
  4 executed / 737 cached, 69.359 seconds; invocation
  `12de5116-a828-47bb-858a-baf14edc801a`.
- The manual nine-phase Bazel cache proof passed on three exact archives:
  `8a1b2c72a0babd9e4b75b2223fb6b78c51fdc9ee` (initial proof),
  `fbfdfc179467a85c6c3866fb9b0dcdd0e125a11f` (schema controls),
  and `5c6a15524ef3c57c8e1a920db4504ee6dbdcbf47` (repeatability).
  Receipts live in ignored `generated/m35-checkpoints/alias-cache-08-first/`,
  `alias-cache-08-final/` and `alias-cache-08-repeatable/`.
- In every run, unchanged warm builds produced no selected actions and warm
  native tests were cached. Changing the defining Rust initializer 62 to 17
  invalidated three dependent metadata actions and four generated bundle actions.
  The independent second producer stayed cached and byte-identical. Every
  affected metadata artifact changed and both target bundles changed.
- The old independent truth failed uncached. Updating only the handwritten
  expected values made Rust, GCC/Zig O0/O2 and Java 21 native proof pass,
  including generated-producer mutation controls. Restoring source and oracle
  restored every generated output hash and reused the original passing test cache.
  The final repeated run also reused the valid changed-source test cache.
- Output-validator negatives explicitly reject unknown bundle-index and alias-
  owner schema versions in both languages. Production certificate/alias mutation
  and atomic-publication negatives remain the gates recorded in child 07; no
  input-manifest authority API was invented.
- Actual generated C/Java alias-only and mixed packages and independent consumers
  remain inspectable under ignored `generated/examples/constant-aliases-18cdc3/`.
  The producer/compiler code is unchanged from the reviewed child 07 checkpoint.
- Fresh independent Sol Extra High review of exact tree 5c6a155 found **no
  findings**, including repeatable cache evidence, temporary-archive safety,
  mutation semantics, schema controls and bounded status wording. The final
  documentation-only tree is release-gated again before scoped commit/push.
  Existing ownership edits remain excluded. No legacy runtime or old corpus
  gate is deleted.
