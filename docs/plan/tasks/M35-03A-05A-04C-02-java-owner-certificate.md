# M35-03A-05A-04C-02 — Strict Java canonical type-owner certificate

- Status: complete
- Parent: [Java canonical owner](M35-03A-05A-04C-java-type-owner.md)
- Depends on: [six-state family](M35-03A-05A-04C-01-java-error-family.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Contract and implementation

Add a closed version-2 Java profile and deterministic namespace derived from the
complete canonical instance key. Retain all original instance/error-kind facts
and exact local family/value selections in the immutable package certificate.
Source-owner and canonical-owner metadata must be mutually exclusive. No fake
core exports or source root, first-consumer identity or handwritten runtime.

The profile accepts exactly Generated.java containing its fixed facade and
Outcome/Success/Error family, with canonical names, modifiers, constructor and
constant inventory. Reject unrelated files, functions, fields, dependencies or
caller documentation that would change canonical output. Ordinary Java source
profile checks remain intact; dependency publication rejects this owner until
04C-03.

## Layer changes

1. AST metadata: replace the source-only optional file-owner field with one
   optional `JavaPackageMetadata` enum containing Source or Canonical variants.
   Retain `JavaSourcePackage` as the source branch's existing description.
   A private-field `JavaCanonicalTypePackage` stores profile, complete original
   facts and exact local family/constant selections. The enum makes simultaneous
   source/canonical metadata unrepresentable; package-wide duplicates still reject.
2. Namespace: extend `JavaPackage` with a typed canonical-instance branch,
   containing the full instance key and closed `JavaCanonicalTypeProfile`.
   Its version-2 name is derived, never caller-supplied. Source namespaces and
   canonical namespaces remain disjoint. Preserve all existing source paths.
3. Verification: strict profile checking runs for both the unresolved package and
   independent reconstruction of resolved input. Factor the local family check
   into a read-only declaration/registration check shared by both paths; do not
   manufacture an intermediate certificate to verify unchecked input.
   Compare namespace, one canonical file/item, exact registered facade/family,
   constants, origin classes and empty dependency/documentation/helper inventory.
   Require a private, empty, parameterless facade constructor followed by
   Outcome, Success and Error in that order, including the exact final-input
   success constructor. Unknown registrations
   remain errors, not unused data to silently discard.
4. Publication: derive descriptive owner/fact access only from the retained
   certificate. Keep source-only dependency publication and compiler manifests
   explicitly closed to the new branch until their separate milestones.
5. Rendering/resources: use existing ordinary structural declarations, imports,
   enum heritage and deterministic paths. Measure normal source/classfile costs.
   No runtime helper, fabricated source exports or zero-cost bypass.

## Definition of done and tests

- Exact inventory/profile/namespace/facts survive projection and independent
  certification; malformed descriptors and source/type mixing reject.
- Maximum-width keys produce valid bounded namespace/path names. Permuted
  registration and unrelated packages cannot change the generated owner bytes.
- Strict standalone Java21 compilation; measured source/classfile costs remain
  charged even without executable source methods. Test exact/one-over policies
  and owner-specific malformed inventories without inventing arbitrary limits.
- Export actual generated files; preserve old outputs/WIP; full Linux Bazel/lint,
  independent GPT-6-SOL review and scoped commit/push.

## Completion evidence

- Public integration target: `//crates/backend-java:java_canonical_type_owner_test`.
  Normal/maximum descriptors survive certification; source-only publication
  rejects them. Independent registries, reversed registration order, unrelated
  owners and exact maximum-width spelling cover deterministic naming.
- Rejection cases cover absent/substituted/source metadata, caller documentation,
  wrong namespace/path, duplicate/swapped/wrong-type roles, modified constructors,
  extra fields and registrations, changed permits/constants and extra files/groups.
- Private resource tests use the same public fixture. All four actual javac
  classfiles are compared with their individual unchanged reservations. Ten
  retained type/value registrations and their name bytes reach exact/one-over
  counters, including the production 100,000/100,001 declaration boundary.
  General JVM capacity boundaries remain covered by the existing resource suite;
  this fixed profile cannot grow arbitrary methods/classes. Dependency-API source
  reservation is extended in 04C-03; aggregate graph reservations remain 04D.
- Strict standalone Java21 compilation uses an empty sourcepath/classpath and
  all warnings as errors. Actual normal and maximum sources are both 919 bytes,
  exported under ignored `generated/m35-java-canonical-owner/`; their SHA-256
  digests match the compiled Bazel artifacts. No handwritten source is exported.
- Initial GPT-6-SOL Extra High review found no production defect but identified
  the missing registration permutation proof; that test was added. Fresh review
  of `05cc71e7` found no correctness/contract/proof defect. Its optional unused
  generic-arena restriction was not adopted: such entries are not emitted file
  declarations, do not affect owner bytes and still undergo shared AST checking.
  This is not permission to accept extra declared types, values or members.
- Full Linux `bazelisk test //... //:release_gate --jobs=2 --keep_going
  --lockfile_mode=off --test_output=errors`: all 1,058 targets pass, including
  Rustfmt, Clippy and Buildifier. Initial tree `164bc1eb` executed 125 targets;
  strengthened/reviewed tree `05cc71e7` executed 15 with the remainder cached.
  Final code invocation: `2387a8d4-1272-4321-966e-ec235dcfc8d4` (672.821 s).
  Eight public canonical-owner cases and all 485 Java unit cases pass; the five
  native partitions run separately and are not disabled or ignored.
- 530 baseline output hashes and 45 protected WIP hashes remain unchanged.
  Source-only dependency APIs still reject canonical ownership; 04C-03 adds
  original typed imports, 04C-04 adds cross-producer proof. Rust Result admission
  remains closed until the subsequent graph and HIR gates.
