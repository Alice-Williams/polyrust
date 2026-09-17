# M35-03A-02F-02B-05C-08 — Constant alias end-to-end closure

- Status: planned
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
