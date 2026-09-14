# M35-01D-04B — Certificate-backed C dependencies

- Status: complete
- Depends on: [M35-01D-04A](M35-01D-04A-compiler-crate-identity.md)
- Parent: [M35-01D-04](M35-01D-04-c-crate-linking.md)
- Contract: [crate dependencies](../../specification/typed-generation/languages/c/rust-hir-crate-dependencies.md)

## Goal

Import exact public C functions through existing typed AST/certification layers,
without treating a manifest or caller-written summary as proof.

## Ordered implementation slices

1. [04B-01 — Certified dependency API](M35-01D-04B-01-dependency-api.md).
2. [04B-02 — Registered imported calls](M35-01D-04B-02-import-registration.md).
3. [04B-03 — Shared external linking](M35-01D-04B-03-dependency-linking.md).
4. [04B-04 — Composed dependency proof](M35-01D-04B-04-dependency-proof.md).

Each slice has focused source/test modules. A read-only dependency API alone
does not enable imported calls or waive any current package verification.

## Definition of done

- A private-constructor dependency API requires a certified C package and
  exposes only exact public scalar functions, signatures, symbols and headers.
- Explicit certified-import registration retains original package/function
  authority and consumer binding. Ordinary missing definitions still reject.
- Extend existing shared linking/reconstruction to exact external dependencies;
  imported C names cannot be silently renamed, and includes are witness-derived.
- Compose existing scalar effects and live call/resource bounds with dependency
  evidence. No zero-cost external leaves, purity flags or raw source stubs.
- Preserve separate owned/imported inventories and independent package output.

## Tests and proof

- Positive two-package typed fixtures and private-constructor compile failures.
- Missing/private/extra/wrong-registry/signature/owner/header/name substitutions
  reject at the appropriate phase, including coordinated graph mutations.
- Missing/understated dependency frames, cycles, arithmetic overflow and actual
  call-path boundary cases reject; native frames/stack verify admitted cases.
- Exact include/owned/imported inventories and separate native compilation.
- Full C/shared/Java regression, format/lint/policy gates and fresh review pass.

## Completion evidence

All four ordered slices are complete. The final composed proof gate
`ffe5aa4d-fe13-4b46-82a4-2c9be14ad48a` passed all 311 tests, including
historical capacity and compile-negative tests; fresh Sol Extra High review
found no actionable core errors. See 04B-04 for native separate-object, exact
symbol, checked-stack and rejected-mutation evidence. Authentication against
actual compiler-loaded Rust dependency metadata remains 04C, not a claim of
this target-level certificate milestone.
