# M35-03A-02F-02B-05C-07 — Compiler lowering and alias-aware publication

- Status: complete
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-04, M35-03A-02F-02B-05C-06
- Specifications: [shared](../../specification/typed-generation/rust-constant-reexports.md),
  [C](../../specification/typed-generation/languages/c/rust-constant-reexports.md),
  [Java](../../specification/typed-generation/languages/java/rust-constant-reexports.md)

## Contract

Connect rustc's checked foreign constant inventory to the existing typed C/Java
producer witnesses. Preserve every finite module/name binding while registering
each defining constant once. Publication must reconstruct the same owned/foreign
partition from target certificates; a descriptive DefId or manifest is not
authority. Do not enable production source admission until both target lowering
and both publication paths pass.

## Implementation order

1. Extend the C API manifest with a distinct certified foreign-constant binding
   inventory. Reconcile graph/root/header metadata and original imported objects
   even when there are no owned declarations. Encode alias-bearing owner schema
   version 6; retain existing versions for packages without aliases. Unsupported
   standalone foreign-owner publication remains rejected.
2. Extend Java bundle projection to read the selected explicit package graph
   without requiring a first source description. Reconcile each foreign binding
   with JavaForeignConstantExport and original producer identity. Encode
   alias-bearing owner schema version 4; retain previous versions otherwise.
3. In each schema distinguish expression-used constant imports from export-only
   aliases. A constant may occur in both lists. Dependencies cover the complete
   union. Preserve exact type, lossless value, original owner, and defining target
   symbol/path. Visit identical checked inventories for reservation and encoding.
4. Extend preflight and exact reconstruction: complete producer closure, original
   member certificate equality, finite bindings, byte budgets and output filenames.
   Reject omitted/replaced owners, swapped alias evidence, unknown schemas and
   incomplete inventories before the atomic output transaction starts.
5. In both compiler assemblers use the extended checked export inventory. Merge
   expression reads and foreign exported DefIds deterministically, authenticate
   each through the existing PublicConstantImports mapping (compiler
   identity/type/value versus certificate), and retain the frozen scope/registry.
   Alias-only roots need no body, owned field, method or object.
6. Keep unsupported foreign module/function/type/macro exports diagnosed. Only
   enable production foreign-constant exports after the above proofs are present.
   Retain entry-point policy separately from public-package selection.

## Definition of done and tests

- Both publication paths pass alias-only, mixed, renamed/repeated aliases, local
  module cycles, direct and transitive two-producer graphs, including zero owned
  declarations and UTF-8 documentation.
- Exact JSON tests assert schema selection, all module/name bindings, disjoint
  owned/foreign inventories, original producer symbols and lossless bool/i32/i64
  values. Existing non-alias schema golden expectations remain unchanged.
- Negative controls reject deleted/swapped/extra/coupled alias metadata, missing
  or independently recertified producers, wrong DefId/type/value/namespace/kind,
  duplicate output/owner identities and inadequate byte reservations.
- Native Rust fixtures prove export-only registration without any expression
  reads. Separately compile generated C and Java files and consumers; no runtime,
  copied alias storage, proxy or wrapper is introduced.
- Atomic publication failure leaves no partial output. Existing compiler-owned
  constants, functions, private/local constants and graph tests stay enabled.
- Full Linux Bazel release and lint gates, fresh independent Sol Extra High
  review, exact-tree evidence, scoped commit and push.
- The following end-to-end closure checkpoint still owns complete source mutation,
  Bazel invalidation, schema tamper, generated examples and parent closure proof.

## Verification evidence

- Exact implementation tree `18cdc3d1d0f556c3b06b40869c8a1e0a878bef7b`
  passed Linux dev-container Bazel `//... //:release_gate`: 1,110 targets,
  **741/741 tests**, 2 executed / 739 cached, 53.367 seconds; invocation
  `ff8f5e6e-6b83-444f-b2ee-f32a36995380`. All prior tests and Rust/Bazel lint
  gates stayed enabled.
- Initial exact tree `6a964a393508cbc0cf0203d7aaa1dc1e76c3bd10` passed the six
  focused native/publication/Java/lint targets, 76.308 seconds; invocation
  `d47c63f7-63b3-436d-84f0-5d568c4cc201`. Its full run passed 739/741:
  new macro argument documentation and an obsolete unsupported-function diagnostic
  expectation were corrected, without disabling the negative test.
- Review-strengthened tree `4265d70ab4f8748f2fb55cc8c4d8bfbdb15427af` passed
  740/741, including overlap/dedup, reversed discovery order and combined-union
  4096 acceptance / 4097 rejection. The remaining new documentation assertion
  expected literal Unicode in C comments; it was corrected to the existing
  specified printable UTF-8 byte presentation, retaining literal Unicode in Java.
- Real two-producer, alias-only and mixed/transitive Rust crates generate separate
  native packages. Independent Rust truth and GCC/Zig O0/O2 plus Java 21 strict
  compilation agree. Every generated C header compiles separately. Java reflection
  proves no alias fields/methods; producer mutations fail unchanged truth after
  readers and consumers are recompiled.
- Exact schema-6 C alias arrays (order and raw field layout), schema-4 Java
  arrays, every finite module/name binding, original paths/owners, lossless
  scalar values, exact COMPUTED import/export overlap, disjoint owned inventories
  and UTF-8 crate/nested-module docs are asserted.
- Atomic proof checks deterministic reordered inputs and byte-identical Bazel
  bundles, alias-only roots, missing owners, seven C manifest mutations, ten
  producer-authority mutants across mixed/empty-owned roots and absent/existing
  destinations, stale metadata rejection and fresh compiler metadata reconstruction.
  Failed writes preserve sentinel output and leave no staging/scratch files.
- Generated bundles and independent consumers are exported as test artifacts and
  copied to ignored `generated/examples/constant-aliases-18cdc3/`, with C/Java
  alias-only and mixed-root examples. No generated files are committed.
- The initial independent Sol Extra High review found no production defect. Its
  required-proof findings (exact C order/read identity, UTF-8 documentation and
  combined union boundaries) were accepted and fixed; follow-up found no
  remaining core findings. A fresh independent Sol Extra High reviewer examined
  final implementation tree 18cdc3d1 and found no findings. The final
  documentation-only checkpoint is revalidated before publication.
- All 20 protected unrelated ownership-file hashes remain unchanged.
  [Child 08](M35-03A-02F-02B-05C-08-end-to-end-closure.md) still owns full Bazel
  invalidation and parent closure. Legacy runtime retirement remains incomplete.
