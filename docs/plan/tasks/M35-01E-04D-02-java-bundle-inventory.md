# M35-01E-04D-02 — Typed Java bundle inventories and resource reservation

- Status: complete
- Parent: [M35-01E-04D](M35-01E-04D-java-bundle-publication.md)
- Depends on: M35-01E-04D-01

## Implementation contract

Execute [D02A source reservation](M35-01E-04D-02A-java-source-reservation.md)
before [D02B inventory projection](M35-01E-04D-02B-java-manifest-projection.md).

- Consume only a complete CheckedGraph and its immutable Java owner certificates.
  Reconcile every used dependency reference with the exact owning graph member.
- Project root/module/export aliases, owned/public function identities, docs and
  used-owner edges into typed descriptive manifest data. Keep serialization in a
  separate module. Metadata must not become a new callable or certificate input;
  private declaration descriptions must never create public dependency handles.
- Retain each canonical Generated.java path. Use one deterministic owner manifest
  per owner and one bundle index; require 1..1024 owners, 2N+1 payload files,
  at most N+6 canonical directories and 256 MiB aggregate UTF-8 payload bytes.
- Reserve a conservative source-output byte bound before rendering. Charge repeated
  identifier/qualified-name occurrences, syntax/indentation and escaped docs with
  checked arithmetic. Reject unsupported reservation shapes explicitly rather than
  using a small fallback estimate. Add manifest/index reservations before rendering.
- After structural rendering, recheck exact output paths/counts and actual bytes
  against the reserved inventory. Do not serialize authority addresses.

## Definition of done and tests

- Exact/one-over owner/file/directory/byte boundaries and arithmetic overflow reject.
- Rendered bytes never exceed reservations for independent small/large/deep/name/doc
  fixtures, repeated foreign calls and all source graph fixtures.
- Altered/missing/duplicate owner, declaration, dependency or manifest inventories
  fail reconciliation. Private metadata remains impossible to import as a callable.
- Deterministic manifests match compiler/API facts; no output text is parsed to
  invent source identity. Backend/resource/source-policy tests and linters pass.

## Closure

D02A and D02B are complete with independent clean reviews. D02B's final
current-tree gate `bed1a0f2-79fa-4e2d-92cb-7e00693d29bb` passed all 369 tests
across 467 targets. Production publication remains D03; committed-tree proof
and milestone commits remain held for E05 migration closure.
