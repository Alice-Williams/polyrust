# M35-01C — Preserve Rust documentation attributes in C

- Status: complete (local migration gate; push held)
- Depends on: certified capability checkpoint in [M35-01B](M35-01B-c-hir-typed-bridge.md)
- Contract: [documentation and proof](../../specification/typed-generation/languages/c/rust-hir-documentation-and-proof.md)

## Goal

Carry resolved doc-attribute text through typed C lowering without recovering
ordinary source comments or replacing rustc's HIR.

## Definition of done

- Collect compiler-resolved doc text in attribute order on admitted owners.
- Preserve original source metadata and normalize generated text through CComment.
- Extend typed owner attachments for declarations/members where needed; retain
  them through linker reordering and primary-declaration placement.
- Support module, function, type and field docs in the admitted subset.
  Interface-method docs become mandatory when interface mapping is enabled.
- Declare included documentation files as Bazel inputs or diagnose unsupported
  include forms; rendering performs no source-file reads.
- Leave ordinary comments and Rustdoc presentation semantics out of scope.

## Tests and proof

Implementation order: first add the typed projection attachment inventory and
compiler module metadata; then exercise declared includes, owner placement and
hostile comments in compiler/native fixtures; finally complete independent
review and the release gate. The current single-.c profile keeps all concrete
record documentation in its implementation. Public-header primary placement
and visibility enforcement close together with M35-01D; this task cannot claim
those future package-boundary tests from the single-file example.

- Equivalent /// and doc attributes, inner docs, empty/multiline/repeated text,
  resolved text macros, distinct owners with same-spelled field names.
- Correct attachment after declaration reordering and no unwanted duplication.
- CComment hostile-text cases compile under strict C17; documentation cannot
  create tokens or affect program behavior.
- Native parity with and without documentation; ordinary comments absent.
- Three-render determinism; docs/Rustfmt/Clippy/Buildifier gates in Linux/Bazel.
- Independent review; visible ignored generated example and recorded evidence.

## Implementation and review evidence

- Typed projection attachments use registered module/file, struct, member and
  function owners. Whole-graph reconstruction checks attachment contents and
  ordering; renderers only place normalized CComment values.
- Compiler provenance retains original ordered attributes and shared Arc module
  ancestry. The sharing regression uses a 1 MiB module document and a 256-field
  record: all 258 owners share metadata before certification and three renders.
- Declared input checks cover text/source includes, out-of-line modules,
  environment attributes, missing files, readable undeclared files and the
  special `-` source spelling. Failures neither create nor overwrite output.
- Independent Sol Extra High review is clean after repairing duplicate primary
  prototypes, source-owner collisions, disconnected crate roots and repeated
  module-text allocation. Native tests also exposed nested C comment openers;
  CComment now safely normalizes both opening and closing delimiters, including
  overlaps. No accepted finding was deferred or test disabled.
- Linux/Bazel invocation `5b5fab5b-2b04-4652-82bd-d4a8db955560` passed seven
  documentation/lint targets, including the large sharing and extraction-limit
  regressions. `9882219c-6f3d-49f4-9fbb-46760968f012` passed all nine expanded
  documentation, compiler-boundary, baseline proof and Buildifier targets.
- `eb855386-c458-4434-b864-12866081c098` passed the focused backend documentation
  unit tests after the final identity/resource/Arc changes.
- Full invocation `ca25eea0-381e-4843-82bd-97ed62083cd7` passed 285 of 286
  targets; the combined C test action timed out at 300 seconds with two capacity
  tests still running. Its four expensive capacity/native checks now have
  explicit independently cached large Bazel actions. The historical
  `portable_backend_c_test` name is the complete suite, retaining all cases;
  each partition rejects a missing/renamed test rather than accepting zero
  matches. The ordinary binary is `portable_backend_c_unit_test`, still covered
  by Clippy/Rustfmt, and the complete suite is explicitly in release_gate.
- All 619 C tests then passed in `d98967d5-abd7-4652-bade-23850e208b63`;
  that invocation's only failure was Buildifier formatting, subsequently fixed.
  The storage and native partitions completed in 647.44 and 435.82 seconds.
  These are test execution timeouts, not increased language resource budgets.
- Review identified substring skip filters as a future coverage risk. Exclusions
  are now exact, and c_test_partition_contract_test compares the complete test
  inventory with the disjoint ordinary/expensive partitions without deduplication.
  It also proves the pinned libtest applies exact matching to skip filters.
  Independent follow-up review found no remaining issue.
- Final Linux/Bazel invocation `15987dd0-e406-4be3-8729-77fee9a68790` passed all
  291 release/frontend/C/codegen/documentation targets. Eight actions executed;
  the remaining valid test results were reused from cache. Clippy, Rustfmt,
  Buildifier, all historical language gates and every capacity test remained
  enabled. No compiler/verifier assertion was weakened.
- Visible ignored artifacts under `experiments/rustc-frontend/output/` are
  model.c, alternate.c, scopes.c, mapping_inventory.c and documentation.c.
  Each was copied from and compared with its actual Bazel generated artifact.

## Commit gate

Keep this checkpoint separate from owned-allocation work. Hold pushes until the
complete C/Java migration gate passes, as required by the latest user instruction.
