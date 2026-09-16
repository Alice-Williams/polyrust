# M35-03A-02F-02B-04B-01 — Shared typed public export inventory

- Status: complete
- Parent: [public constant compiler integration](M35-03A-02F-02B-04B-public-source-constants.md)

## Contract

Replace duplicated C/Java public function selection with one compiler-owned
inventory. Resolve stable declaration identities into a closed Function/Constant
kind enum; retain the exact finite module/alias graph. Keep construction private
and derive the inventory from checked compiler definitions and the origin cache.
Reject foreign/unmapped exports, unsupported declaration kinds, ambiguous
identities, unexported declarations and excessive inventory size.

Constant classification is not scalar constant admission or target support:
the forthcoming PublicConstants mapping must still validate types and evaluate
values. Both production backends retain an explicit function-only gate until
the complete constant mappings and publication path are implemented.

## Definition of done and tests

- C and Java consume the same stable, deduplicated function inventory.
- Compiler probes exercise function-only, constant-only and mixed graphs;
  aliases, re-exported hidden-module declarations, cyclic module bindings, docs
  and private same-spelled declarations preserve their identities.
- Compiler inputs with unsupported public kinds, foreign exports or no public
  declarations are rejected. Production constant packages still reject before
  publication; existing private/local constant behavior is unchanged.
- No synthetic function, target AST object or name-based resolution is used.
- Focused compiler/native regression tests and full Linux Bazel/release/lint
  gate pass. Independent review and a scoped tested commit/push complete this
  checkpoint, without claiming completion of its parent or runtime retirement.

## Verification evidence

- Production C/Java public selection uses the shared private inventory; the
  duplicated C selector and Java selector were removed.
- The compiler probe passes five positive graphs, including the 4,096 limit,
  plus twelve unsupported/empty/foreign/over-budget rejection controls.
- The private-construction compile-negative probe requires Rust E0451.
- The production constant rejection matrix covers 76 executions, including
  direct and re-exported constants-only crates in C and Java with both missing
  and pre-existing output destinations. Rejection preserves output bytes.
- Exact candidate tree 94f449d7e7b36e989dd401ecde56948092002a0e passed
  `bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test
  //... //:release_gate --noshow_progress --noverbose_failures
  --test_output=errors --test_summary=terse --keep_going` in the Linux
  development container: 1,004 targets, 686/686 tests passed. The final run
  executed the three invalidated tests and reused the other 683 results.
  Invocation: 477df52c-6cf8-4c48-b817-ca9d432ac8c2.
- An independent Sol Extra High review identified the missing constants-only
  production rejection proof. That finding was accepted and covered; its
  follow-up reported no unresolved core findings. The exact 100,000-owner
  stress boundary is not tested; the loop limit was reviewed directly.
- A fresh independent Sol Extra High review reported no actionable correctness,
  regression, test or specification findings in that exact staged implementation.
- The completion documentation is included in the final isolated full gate before
  committing this checkpoint. Runtime retirement and public constant output
  remain unfinished in the parent tasks.
