# M33-01 — Pin stdlib is-negative-zero 0.2.3

- Status: pending

## Goal

Retain an immutable, license-auditable upstream oracle for the exact declared
numeric API and its JavaScript and C implementations.

## Definition of done

- The lightweight `v0.2.3` tag resolves to
  `766200b9eeea46b7f827ac7d63effa6bea65d896`.
- The JavaScript entry point/implementation, TypeScript declaration and
  declaration test, runtime/native tests, C implementation/header, package
  metadata, README, NOTICE, and Apache-2.0 license are retained byte-for-byte.
- Every retained file's exact Git blob ID is recorded and verified by an
  offline Bazel test.
- The provenance test also proves the package name, version, license, and exact
  `number -> boolean` declaration.
- No package-manager install or network access is required by permanent tests.

## Tests

- `bazel test //third_party/stdlib-is-negative-zero:provenance_test --test_output=errors`
