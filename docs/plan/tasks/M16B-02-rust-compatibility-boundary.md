# M16B-02 — Rust compatibility and native test ownership

- Status: complete
- Parent: [native Linux CI](../milestones/M16B-native-linux-ci.md)
- Depends on: M16B-01 local implementation

## Failure and contract

Hosted run 35025443248 at commit 37531f3 failed both Rust compatibility jobs:
Cargo executed C native tests without Bazel's test directories and declared
compiler runfiles. Ten tests failed; the dependent release gate never ran.
Local Bazel success was not evidence of hosted workflow success.

The compiler-version matrix must compile and link all workspace tests with
all features and the selected Rust version using the locked dependency graph,
without executing tests outside their authoritative Bazel environment.
Keep test execution mandatory in the release script's unfiltered Bazel test
//... invocation. Do not add ignored tests, environment-based early returns,
skip filters, continue-on-error or a successful fallback for missing tools.

## Definition of done and tests

- Pinned and stable compiler jobs run cargo test --no-run --workspace
  --all-features --locked with their explicitly selected toolchain.
- A Bazel policy test accepts the checked-in workflow/release split and rejects
  mutants that execute Cargo tests, omit compile coverage, skip Bazel execution
  or make compatibility/release failures optional.
- Container reproduction compiles the complete Cargo test suite with the pinned
  toolchain; authoritative Bazel native tests and the full local gate pass.
- Workflow syntax/actionlint, Rust/Bazel linters and documentation pass.
- Independent review, scoped commit and push; verify the exact pushed SHA's
  hosted run, including native release execution and cache save, before claiming
  CI is green. Record local and hosted evidence separately.

## Local evidence

- Actionlint 1.7.12, shell syntax and the CI boundary policy pass. The policy
  rejects 25 regressions, including step-level skips, an extra executing Cargo
  test step, optional failures, missing compile coverage and filtered Bazel tests.
- Initial isolated tree 0a75faf2136609e8366eaa1f7f5fc62def8f6e6b passed all
  684 Bazel workspace tests across 1,001 targets in 119.010 seconds, invocation
  386b4ac2-f0d9-4eba-b665-4c2135e7339d. Native tests remain enabled.
- Container Cargo reproduction explicitly removed TEST_TMPDIR, TEST_SRCDIR,
  RUNFILES_DIR and TEST_WORKSPACE. Pinned1.98 compiled/linked all 46 workspace
  test executables in 4m56s using --no-run --workspace --all-features --locked
  --offline. The stable alias passed in 0.66s using cached outputs; both installed
  aliases currently resolve to rustc1.98.0, so this is not a claim of two distinct
  local compiler versions.
- cargo audit --deny warnings passed against 1,246 advisories and 63 dependencies.
  The explicit conformance/determinism command passed 50 cases across all eight
  targets, invocation 0ec43adf-3c95-424e-b58b-d3356c134996.
- Independent review accepted the workflow split and found two policy gaps:
  step-level skips and extra executing Cargo tests. Both findings were accepted
  and fixed, with corresponding negative controls. Re-review identified early
  exit, syntax-only shell selection, matrix exclusion and comment-only dependency
  checks as additional policy gaps. These were accepted: the policy now validates
  a closed compatibility/release recipe, not arbitrary YAML or shell semantics.
  Job-level fields are checked across the entire mapping, including fields after
  the steps list. Required steps explicitly select Bash. A fresh review and final isolated gate
  cover this stronger policy before push.

## Hosted evidence

Run [35099743690](https://github.com/Alice-Williams/polyrust/actions/runs/35099743690)
at a3934c85c424ca41411a9417f150b01b795c777b passed both Rust compatibility
jobs, the Rust/Bazel linters, both determinism jobs, the manifest comparison and
the Windows contract. The native release job executed all 684 Bazel tests:
683 passed, but the 330-case Java backend suite reached its inherited 300-second
timeout. No assertion failure was reported in that suite's partial output.
The downstream conformance command and cache save did not run. Follow-up
[M16B-03](M16B-03-java-native-suite-budget.md) addresses the test classification.

Replacement run 35107402373 at 081a796ce28ef429ba46ab787df284b7e88694d1
passed every job, including native release execution and cache save. See the
[milestone evidence](../milestones/M16B-native-linux-ci.md#hosted-completion--2026-09-16).
The final independent CI-boundary review approved the pushed fix with no
remaining core findings.
