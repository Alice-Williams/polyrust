# M35-03A-02F-02B-05C-07 — Compiler lowering and alias-aware publication

- Status: planned
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
