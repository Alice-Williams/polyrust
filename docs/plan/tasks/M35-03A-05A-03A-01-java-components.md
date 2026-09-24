# M35-03A-05A-03A-01 — Java synthesized component identity

- Status: complete
- Parent: [local Java results](M35-03A-05A-03A-java-local-results.md)
- Depends on: [C result transport](M35-03A-05A-02-c-results.md)
- Specification: [Java scalar results](../../specification/typed-generation/languages/java/rust-scalar-results.md)

## Contract

Represent a synthesized component by its actual generated owner and an enum role,
initially ScalarResultPayload with primitive int storage. The owner must be a
synthesized interface-adapter record, not a RustSource or legacy runtime type.
This metadata is descriptive until the ordinary verifier authenticates the
declaration, reference, signature, scope and assignment. It does not by itself
certify a complete Result family.

Support exact constructor assignment, same-nest field reads and public record
accessors. Do not permit name-only structural references to bypass this identity.
Keep final-field flow, private access, symbol discovery, output accounting and
existing source-origin checks in force. No copied runtime or text templates.
Dependency publication and compiler Result admission remain unchanged.

## Definition of done and tests

- Typed references reject wrong owners, roles, names, types and signatures;
  same-layout records are not interchangeable. Reject missing/duplicate assignment,
  reassignment and reads before constructor initialization.
- Exact generated constructors, copies, direct reads and accessors certify and
  compile under strict Java21 lint. Separately compiled consumers run signed
  extrema and ordinary values in normal and interpreted execution.
- External field access/private construction fail; public accessor reads work.
  Unsupported synthesized nominal dependency publication stays rejected.
- Existing capacity gates, all Linux Bazel release/lint tests, output/WIP
  preservation and broad independent GPT-6-SOL review pass before commit/push.

## Scope boundary

Complete sealed families, guarded success/error observations, imports and source
conversion belong to the remaining parent tasks. Do not call this Result support.

## Verification history

The first compiler pass caught two omitted exhaustive enum arms in JVM resource
accounting; both now follow the existing typed-field accounting path. The first
complete Java unit run passed (294.592 seconds overall, 258.5 seconds execution),
including the six initial new cases. After adding canonical dependency rejection,
pre-render privacy/substitution checks, exact 255/256 method capacity and native
classfile accounting, all ten focused cases passed (39.463 seconds overall,
4.3 seconds execution). These were followed by the full gate and review below.

The reviewer correctly identified that the initial general-fixture dependency
rejection could fail at its facade name rather than its nominal declaration. The
new canonical source-facade test reaches and asserts the record-specific rejection
for private and public synthesized records. This is accepted feedback, not a
change to production publication policy.

## Completion evidence

- The complete Linux Bazel `test //... //:release_gate` passes all 1,041 targets
  on code/test tree `42c786238117f476061fcbc71e7ea273435a7f8e`: 123 executed,
  the rest cached; 1,138.878 seconds overall. Rust Clippy/rustfmt, buildifier,
  source/documentation policy and all native partitions remain enabled.
- Java unit execution passes 446 cases with four separately executed native
  partitions excluded from that unit invocation. The ten new cases include
  complete signature/owner/type rejection, final assignment, structural bypass,
  canonical dependency rejection, privacy and exact method-slot boundaries.
- Strict Java21 separately compiled consumers pass six signed payload rows in
  normal and interpreted execution. Four external negative consumers reject.
  Both public/private producer variants have all three emitted classfiles checked
  against their same-AST resource reservations.
- Broad independent GPT-6-SOL extra-high review of the complete checkpoint is
  clean after the accepted publication-test improvement. No findings were waived.
- All 530 prior generated file hashes and four additional character-constant
  bundles are unchanged. The 45 unrelated WIP file hashes are unchanged.
- This closes component identity only. Local family certification, imported
  transport, measured branch traces and source admission remain separate work.
