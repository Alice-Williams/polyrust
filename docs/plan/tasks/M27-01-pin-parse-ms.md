# M27-01 — Pin parse-ms v3 provenance and scope

- Status: complete

## Goal

Retain an immutable, license-auditable parse-ms release whose complete typed API
fits the current portable type system.

## Definition of done

- Version 3.0.0 is pinned at commit
  `49dab09236deeea5d2c082182e2c73e7a79763a8`.
- The implementation, declaration, type test, official tests, package metadata,
  README, and MIT license are retained under `third_party/parse-ms/`.
- Every retained file is checked against its upstream Git blob ID offline.
- The `number -> TimeComponents` v3 API is fully in scope; v4 `bigint` and
  dynamic non-number calls are explicit boundaries.

## Tests

- `bazelisk test //third_party/parse-ms:provenance_test --test_output=errors`.
- Deliberately changing any retained byte must fail the provenance target.
