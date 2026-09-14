# M35-01E-04D-03 — Production Java bundles and independent proof

- Status: complete
- Parent: [M35-01E-04D](M35-01E-04D-java-bundle-publication.md)
- Depends on: M35-01E-04D-02

## Implementation contract

- Add production bundle publication to the Java graph adapter and a Bazel rule
  consuming the existing declared Rust crate graph provider. No host compiler or
  dynamic dependency discovery. Keep check mode strictly non-publishing.
- Complete compiler analysis, graph/resource validation and rendering before the
  transaction. Publish once with the shared atomic no-replace directory boundary.
- Extend the existing independent Java graph oracle to consume actual production
  bundles and manifests, not the test-only probe format. Preserve probe assertions
  that compare source/compiler facts with certificate structures.

## Definition of done and tests

- Real four-crate generated Java files compile separately with pinned Java 21
  --release 21 -Xlint:all -Werror; all 8,204 x 16 Rust reference results agree.
- Public aliases/docs, exact private/owned declarations and used dependencies
  match the compiler-backed inventory. Illegal private consumers fail.
- Reordering/relocation is byte-identical. Existing/late/concurrent destinations,
  failed members, overflow and staging failures never produce a partial bundle.
- Full C/Java migration gate and a fresh broad Sol Extra High review pass; assess
  all findings and repeat fresh review after core fixes. No tests are disabled.
- E05 remains required for final example artifacts and committed-tree proof.

## Progress

- Production `--bundle DESTINATION` performs full source graph analysis,
  exact-owner projection/resource reservation and rendering before one shared
  atomic tree publication. Check mode remains non-publishing.
- Added independently declared `rust_java_bundle` Bazel action using existing
  `RustSourceCrateInfo` records and pinned tools. The four-crate native oracle
  now consumes actual CLI bundle/manifests and compares all bytes with the Bazel
  action and separate compiler-fact probe. Consumer bindings come from production
  manifests, never the probe TSV.
- Added actual Java integration tests for failed compiler members, untouched
  inputs, permissions, existing and late destinations, helper failures and two
  complete staged bundles racing the native no-replace rename. Test helper
  injection uses a separate wrapper, not production runtime mutation options.
- Initial focused gate `1d4c1dc9-655e-413b-b1b3-470bcd076904`: native production
  equivalence, check-mode admission, Rustfmt, Buildifier and docs passed. The new
  filesystem test rejected the bad compiler member correctly but expected the
  wrong Rust diagnostic text; corrected its assertion to the actual pinned
  compiler E0277 and rerunning. No tests disabled.
- Full current-tree gate `f856b5d9-a61f-4f9e-8562-1a0647de8232` passed all
  370 tests across 470 targets. This includes production native equivalence,
  every private generated function/record, filesystem failure and race checks.
- Added a direct production bundle regression for declared-but-unused owners
  (four owners/nine files retained; no invented root used-dependency edges), plus
  a precise pre-publication malformed-graph diagnostic assertion. Final review
  remains pending. Post-addition publication/docs gate
  `ae44af82-0a7e-48da-81af-3ed9315de4c5` passed both targets.
- Produced `generated/m35-java-production-preview/` on the host from the actual
  Bazel tree, exactly nine payload files, and verified `diff -rq` byte equality.
  Its explanatory note is a sibling file, not an extra bundle payload. Both
  outputs are ignored; the experiment README documents reproduction commands.

## Review correction and final proof

- `java_production_bundle_review` found one core proof gap, accepted: the original
  native checker proved private API descriptions against rustc, but serialization
  could omit/change private JSON fields without an independent complete expected
  JSON inventory. Rendering a second observation bundle with the same serializer
  was insufficient. No production-code defect was found.
- Added a separate typed observation writer, not using the manifest projector or
  JSON encoder. It emits every source/target/module/binding and ordered doc fact.
  The four-crate compiler probe also requires exact equality between description
  IDs and its complete compiler Fn/Struct/Field inventory (all fixture declarations
  are retained). Thus an omitted private collector entry rejects as well.
- `java_manifest_assertions.py` compares complete public/private declarations,
  fields, records, modules, docs, visibility, locations, scalar signatures and
  fully decomposed Java target paths. Mutations remove each private kind and
  modules/bindings, duplicate entries, and change metadata/targets; every mutation
  must fail the independent oracle. Native privacy tests enumerate every private
  generated function/record only after this exact comparison passes.
- Corrected proof, Rustfmt and Buildifier pass in
  `f11d2028-a4d9-40e9-9bec-f04d7c56c2f8` (3/3). A fresh broad Sol Extra High
  reviewer `java_production_final_review` independently concluded no core findings
  after the correction. Final full gate `49c0e966-89a7-4dc4-8b25-640149f83724`
  passed all 370 tests across 470 targets. Nothing was disabled or pushed.
- Optional review suggestion, deferred with rationale: declare the action's
  ambient `/usr/bin/rmdir` explicitly. It only removes Bazel's precreated empty
  tree and is the existing C-rule convention on the constrained Linux action
  platform. It neither selects compilers/dependencies nor authorizes replacement
  publication. This is hermetic setup hardening, not a current contract defect.

This closes D03, parent D and E04. E05 still owns the remaining full fixture
matrix, exact proposed committed-tree proof, separate C/Java commits and push.
