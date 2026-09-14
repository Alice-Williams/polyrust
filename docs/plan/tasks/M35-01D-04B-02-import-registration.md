# M35-01D-04B-02 — Registered imported C calls

- Status: complete
- Depends on: [M35-01D-04B-01](M35-01D-04B-01-dependency-api.md)
- Parent: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)

## Definition of done

Bind exact certified dependency functions into a consumer registration context.
Reuse existing typed callable/signature/expression nodes; retain original
dependency authority and keep owned declarations distinct from imports.
Ordinary missing definitions, cross-registry substitution and private imports
must still reject. No stub body or raw external declaration is permitted.

## Tests and proof

- Positive authenticated scalar calls and exact consumer/original ownership.
- Duplicate/conflicting imports, wrong certificate/reference/signature and
  private/deleted dependency bindings reject, including coordinated mutations.
- Existing registry/context/ownership negative tests remain enabled.
- Imported calls are not render-ready until shared linking and composed safety
  obligations in 04B-03/04B-04 are implemented and tested.
- Linux Bazel target/lint gates and fresh independent review pass.

## Evidence

- Consumer-branded imported functions retain exact original references,
  signatures, public headers and opaque certificate identity. Imports have a
  separate read-only inventory; no new output file or local body is fabricated.
- Tests cover i32/bool argument shape, duplicate/conflicting registration,
  cross-registry use, header collisions, deleted/private/retargeted references,
  altered signatures/contracts and substitution of an independent certificate
  sharing the same original function registration. Projection remains closed.
- Review identified callback-role widening through general function lookup.
  Accepted and fixed: function addresses, indirect-call contracts, callable
  members and every interface callback role remain owned-only. Dedicated tests
  prove each rejection and preserve the corresponding owned positive case.
- `4088381a-82cd-4c3a-9e4b-8489ce40b013`: post-fix focused 8/8 Bazel targets
  pass in the Linux container, including 688 ordinary C tests, compile-negative
  tests, Rust formatting/Clippy, buildifier and documentation/source policies.
- Earlier full gate `f1cc1915-a31d-408a-8333-7ddaf1a9415b` was interrupted by
  Docker shutdown; it is not completion evidence.
- `12f6551b-1499-4f24-9a2e-134131290023`: fresh complete release/compiler/
  C/shared/Java gate passes: 351 targets, 311/311 tests (804 seconds; 71 executed,
  the remainder cached), including all five capacity proof partitions.
- Fresh independent Sol Extra High post-fix review found no substantiated core
  errors. All tests remained enabled and no push was made.
