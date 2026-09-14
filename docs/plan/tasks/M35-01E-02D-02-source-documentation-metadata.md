# M35-01E-02D-02 — Checked source documentation metadata

- Status: complete
- Parent: [M35-01E-02D](M35-01E-02D-java-source-documentation.md)
- Depends on: completed M35-01E-02D-01

## Implementation contract

- Put target-independent metadata coherence in the shared codegen source module,
  not the Java renderer. Return a privately constructed borrowed checked inventory
  of declaration owners, canonical module documents and the crate export graph.
- Accept one crate per inventory. Require declaration origins, unique declaration
  IDs, disjoint module/declaration identities, consistent root-to-owner ancestry,
  same-crate parents and visibility restrictions to an ancestor module.
- Require complete reachable local export modules, exactly matching module
  ancestry keys and type-namespace module edges. Preserve foreign edges and
  cyclic aliases without recursively expanding paths. Shared payloads must agree.
- Bound declarations, ancestry traversal, export entries, attributes and metadata
  text before comparisons/projection. Shared allocation fast paths may avoid
  repeated payload scans, but cannot skip coherence checks for distinct payloads.
- Invoke this check for all Java source registrations and record components at
  unresolved verification and the existing post-link reverification boundary.
- A synthesized Java facade uses PackageEntryPoint synthesis, not a fabricated
  Rust declaration for a module. Genuine types/functions/fields retain RustSource.

## Definition of done and tests

- Positive fixtures cover private ancestry, exported modules, foreign module
  edges, alias cycles, separately allocated equivalent metadata and empty input.
- Negatives cover missing/unreachable/foreign modules, wrong namespace, changed
  root/parent/owner, duplicate or mixed-kind identities, conflicting shared docs
  and inventories, non-ancestor visibility, body origins and mixed crates.
- Exact and one-over resource boundaries use the same production checker with
  reduced private test limits; production limits are asserted separately.
- Java package tests prove the shared check is mandatory, including source fields.
- Full container gate and fresh independent review pass.

## Scope boundary

This is metadata coherence, not proof of compiler analysis or Java renderability.
It does not add a parser, Rust AST, target syntax or renderer. Attachment routing,
normalized-output budgets and visible generated docs follow in the parent task.
The existing C metadata policy remains unchanged in this checkpoint.

## Review repairs

- Accepted the first review's eager-Java-walk finding. Metadata discovery now
  streams through validation; field traversal retains ancestor cursors rather
  than collecting every field or sibling. A lazy counting/panic-tail regression
  proves the declaration cutoff and a Java field regression proves early stop.
- Accepted mixed-kind export identities: exported declaration targets now cannot
  name root/private ancestry modules or identities also tagged as foreign modules.
  Positive declaration aliases remain valid and do not duplicate owners.
- Superseded full gate `02651ff8-9cac-4928-a2b5-8e5455d81138` was intentionally
  interrupted after review findings, with 341 passing tests and two skipped by
  cancellation. It is not completion evidence. No test was disabled; the repaired
  revision requires the complete gate again.
- Fresh Sol Extra High review found no further core defects in the repaired
  checker and Java integration. Focused gate
  `969191c7-0caf-4fb9-a41f-86a8515fc814` passed all five targets.
- Full gate `43f2e843-787e-4710-925a-a9e83ca0312d` passed 342 tests, including
  the C stress cases, but failed the Rust lint target. Focused reproduction
  `cdbbcf4d-3d02-4730-97a2-0adcfe35ac37` identified the Java crate's prohibition
  on wildcard production imports. Replaced the traversal's wildcard with explicit
  imports; no lint was suppressed. Completion still requires the repaired full gate.

## Completion evidence

- Repaired full container gate `c9463973-e3c2-443c-9cd1-dbc0034ff5ea` passed
  343/343 tests across 413 targets, including Rust/Bazel lint, compiler/C/Java
  regressions, private-construction compile-fail checks and documentation policy.
- Fresh independent Sol Extra High review found no remaining core defects.
  The final explicit-import repair changed no validation or traversal behavior.
