# M35-01D-04B-03A — Package-derived symbol catalogues

- Status: complete
- Depends on: [04B-02](M35-01D-04B-02-import-registration.md)
- Parent: [04B-03](M35-01D-04B-03-dependency-linking.md)

## Definition of done

- Add a fallible dialect hook that derives the exact symbol catalogue from the
  checked unresolved package. The default retains existing static catalogues.
- Link through this hook and independently reconstruct it in post-link
  verification. A self-consistent modified linked catalogue is not authority.
- Require exact dialect-value agreement with the original package at both
  boundaries before invoking a potentially different configuration's hooks.
- Keep catalogue validation/reconstruction and new tests in focused modules.
- Do not enable C dependency calls or add raw-name dependency constructors.

## Tests and proof

- Existing C/Java static catalogues preserve output and complete regression gates.
- A package-aware test dialect changes its catalogue from actual package input;
  link and post-link verification must agree and deterministic repeats match.
- Hook failures propagate without a linked/certified package.
- Missing/extra/changed metadata and coordinated catalogue/import/reference
  substitutions reject, including unreferenced entries and default catalogues.
- Linux Bazel shared/compile-negative/C/Java and lint gates pass; fresh independent
  review finds no remaining substantiated core defect.

## Evidence

- Extracted the existing linker trait and catalogue validation into focused
  `linking/dialect.rs` and `linking/catalogue.rs` modules. Added the default
  package-derived hook, exact post-link reconstruction and fail-closed handling
  even when a failing hook supplies an empty diagnostic vector.
- Six focused tests cover actual package-dependent names, deterministic repeats,
  static compatibility, failure propagation, unused/extra/changed metadata and
  coordinated catalogue/import-origin mutation across shared physical aliases.
  The coordinated fixture explicitly passes the older per-reference consistency
  checks before the new original-package check rejects it.
- `531d5bd7-c3dc-4dbc-b1ad-4fb458cec350`: C/Java, compile-negative and lint/
  policy targets passed (9/10); a new coordinated-alias test fixture needed a
  correction. Production code was unchanged by that correction.
- `cd61a064-2afc-426a-bdf0-92b5f321d406`: corrected shared test and format
  targets pass (2/2), including all 90 shared unit tests. Complete regression
  and fresh independent review remain before completion.
- Independent review found a stateful-dialect mismatch between validation and
  catalogue derivation. Added entry checks and positive/negative configuration
  matrices. The superseded full run `17fa6504-2525-48a4-8b0f-8f17e9e762cb`
  was deliberately interrupted; its partial results are not completion proof.
- `855bf09e-756f-4922-a816-c072776a99f2`: repaired shared test/format gate
  passes, including 91 shared unit tests.
- `9cb8a982-bf89-41c9-99ad-269a241896c9`: full Linux release/frontend/C/shared/
  Java/documentation gate passes all 311 tests (351 targets), including all five
  C capacity partitions, compile-negative tests and Rust/Bazel linting.
- Fresh Sol Extra High review reports no substantiated core defects. Optional
  invocation-counter and third-repeat assertions are not blockers: entry checks
  precede hooks by construction, replacement matrices reject, and deterministic
  reconstruction/repetition is already covered. No optional feature is treated
  as a core correctness failure.
