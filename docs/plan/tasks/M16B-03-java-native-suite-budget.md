# M16B-03 — Java native suite execution budget

- Status: in-progress
- Parent: [native Linux CI](../milestones/M16B-native-linux-ci.md)
- Depends on: M16B-02 compatibility-job fix

## Failure and scope

Hosted run 35099743690 executed all 684 Bazel tests: 683 passed and
//crates/backend-java:portable_backend_java_test timed out at 300 seconds.
Its 330 Rust test cases include repeated native javac/java invocations,
positive executions, invalid-program controls and compiling semantic mutants.
The BUILD target still inherited the default medium-test timeout.

Classify this compiler-heavy suite as large, giving it Bazel's bounded
900-second default timeout. Do not change the global timeout, exclude cases,
ignore failures, reduce assertions, add retries or disable cached test results.
This is an execution-budget correction, not a claim that every observed timeout
is harmless. The complete suite must finish successfully before push.

## Definition of done and tests

- Bazel resolves the target to size large and timeout long (900 seconds).
- An uncached container execution completes all 330 cases without failures or
  ignored tests; also exercise four Rust test threads, matching the hosted
  runner's constrained concurrency.
- The unchanged full workspace/release suite, Rustfmt, Clippy, Buildifier,
  documentation and CI policy targets pass in the Linux development container.
- Independent review finds no unresolved core defect.
- Commit and push only the scoped BUILD/documentation changes.
- Verify the replacement hosted run, including native test execution,
  conformance/determinism and successful cache save, before closing M16B.

## Evidence

- Exact isolated Git tree 993c817f47c546d174e23a51306cc3de46a8c5ad was
  verified against all 2,582 archived Git blobs and executable modes.
- Bazel query with default attributes resolves size large and timeout long.
- Uncached execution with RUST_TEST_THREADS=4 passed all 330 cases in 133.29s:
  zero failures, ignored tests or filtered cases. Invocation
  bbfa1a41-144b-4c23-8048-9c0c0f83976a.
- The full container command bazelisk --batch test //... //:release_gate passed
  all 684 tests across 1,001 targets in 158.398s. Four test targets executed;
  unchanged results were cached. The Java suite reran with default test-thread
  settings. Invocation 03918176-e04b-4265-a80d-6462ae209187.
- Independent review confirmed that the hosted partial log contains 329 passing
  cases and no assertion failures before the 300-second timeout; no required
  tests, filters, assertions or release commands have changed.

The earlier hosted failure is recorded in
[M16B-02](M16B-02-rust-compatibility-boundary.md). Final staged-documentation
verification and review precede push. Hosted completion remains pending; this
local proof does not claim that the replacement GitHub run is green.
