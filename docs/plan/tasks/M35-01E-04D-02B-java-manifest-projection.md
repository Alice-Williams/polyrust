# M35-01E-04D-02B — Typed manifests and complete Java bundle reservation

- Status: complete
- Parent: [M35-01E-04D-02](M35-01E-04D-02-java-bundle-inventory.md)
- Depends on: M35-01E-04D-02A

## Implementation contract

Follow the [versioned Java bundle manifest specification](../../specification/typed-generation/languages/java/rust-hir-bundles.md).

- Project descriptive roots/modules/exports, owned/public functions, docs and
  used-owner edges from the exact complete compiler graph and its certificates.
- Keep metadata data types, JSON serialization and bundle reconciliation in
  separate modules. No metadata reconstruction grants callable authority.
- Reserve all owner sources, owner manifests and bundle index before rendering;
  enforce the parent task's owner/file/directory/byte ceilings with checked math.
- Reconcile canonical files, actual byte totals and exact used-owner witnesses
  after rendering and before passing the entire payload to atomic publication.

## Definition of done and tests

- Missing/replaced/duplicate owners, declarations, dependencies and manifests
  reject. Limits test exact/one-over/overflow. Private descriptions are not APIs.
- Compiler facts match typed inventories and deterministic serialized bytes;
  aliases, private helpers and docs survive relocation and record reordering.
- Backend/source-policy/compiler graph tests and linters pass; full gate and
  fresh broad review close parent D02 before production publication in D03.

## Progress

- Added certificate-borrowed JavaSourceDescription/JavaSourceTarget enums and
  JavaDependencyApi::source_descriptions. The backend retains its private
  registration table; consumers receive typed declaration paths, exact source
  origins and function/record/field descriptions, including private metadata.
- Descriptions are stable-ID ordered and cannot be promoted into public
  JavaDependencyFunction handles. The defining certificate outlives every
  returned borrow; collection uses an explicit owner lifetime.
- Focused gate `71710635-3038-481f-95ba-3d91c4d1d773`: 5/5 targets pass,
  including Java tests/compile-fail, Rustfmt/Clippy and opaque-source policy.
- Current-tree full migration/release gate
  `debc410b-b981-4547-a39c-04b084f74412`: all 367 tests across 464 targets pass;
  no tests disabled. This includes the new descriptive API and its tests.
- At that initial API-only checkpoint, projection/serialization/reconciliation
  and proof were still required; the closure below supplies those results.

## Implemented bundle projection

- Added independently cached first-party `java_bundle` library with separate
  borrowed manifest types, projection/reconciliation, checked budget, structural
  JSON sinks, complete payload assembly and focused tests; no new dependencies.
- Exact dependency package/function witnesses reconcile against the whole input
  graph. Compiler check mode now preflights metadata/source reservations without
  rendering or publishing. Used-owner substitution with equal stable IDs rejects.
- Real four-owner compiler probe emits all nine bundle files plus its independent
  observation file. Java 21 separate compilation, 131,264 Rust/Java result values,
  private method rejection, manifest/API aliases/docs and byte-identical graph
  reordering/relocation pass. Private record/field/function origins are now also
  compared directly with rustc IDs, scalar types, visibility and doc attributes.
- Unit mutations reject missing/duplicate/replaced declarations, missing module
  metadata, wrong manifest names and missing/duplicate/replaced payloads. Exact
  1024-owner/2049-file limits, unused members, escaping, arithmetic overflow and
  exact/one-over byte counters are covered.
- Full current-tree gate `4736ed11-1a44-4aa1-8108-c647d2c65d94` passed
  369 tests across 467 targets. The subsequent direct private compiler-origin
  probe passed with Rustfmt/Buildifier in `8bf91299-7caf-4e66-bd1b-9fab898ab921`.
- Initial fixture/probe compile errors were corrected using public typed
  signatures and the existing compiler normalization API; no checks disabled.
- Final full current-tree gate `bed1a0f2-79fa-4e2d-92cb-7e00693d29bb` passed
  all 369 tests across 467 targets, including final compiler-origin and directory
  inventory checks. No tests disabled. This is not committed-tree/push proof.
- Fresh Sol Extra High `java_bundle_projection_review` found no core defects
  after reviewing the final implementation/proof snapshot. The private compiler
  fact coverage gap identified during review was closed by the direct source
  probe and rerun above. A second fresh final-snapshot Sol Extra High reviewer,
  `java_manifest_final_review`, independently returned clean with no core findings.
- Optional suggestion: add a bundle-level cyclic module-alias serialization
  fixture. Deferred as extra regression coverage, not a correctness blocker:
  existing graph tests cover cycles, and the serializer emits typed module
  binding edges flatly without recursive expansion. No issue-count cap was used.
