# M34-01 — Pin stdlib abs 0.2.3

- Status: complete

## Goal

Retain an immutable, license-auditable upstream oracle for the exact declared
numeric API and its JavaScript and C implementations.

## Definition of done

- The lightweight `v0.2.3` tag resolves to
  `fbdc5b76328d9f376ea1851c0e6c84bde50278bf`.
- The public JavaScript entry point/implementation, independent bit-level
  implementation, TypeScript declaration and declaration test,
  runtime/bitwise/native tests, C implementation/header, package metadata,
  README, NOTICE, and Apache-2.0 license are retained byte-for-byte.
- Every retained file's exact Git blob ID is recorded and verified by an
  offline Bazel test.
- The provenance test also proves the package name, version, license, exact
  `number -> number` declaration, public `Math.abs` path, and independent
  JavaScript/C sign-bit-clearing paths.
- No package-manager install or network access is required by permanent tests.

## Tests

- `bazel test //third_party/stdlib-abs:provenance_test --test_output=errors`

## Completion evidence

- Lightweight tag `v0.2.3` resolves to
  `fbdc5b76328d9f376ea1851c0e6c84bde50278bf`.
- All 17 retained files hash exactly to their immutable upstream Git blob IDs.
- The declaration is exactly `abs(x: number): number`, and both retained
  representation-level implementations clear only the binary64 sign bit.
