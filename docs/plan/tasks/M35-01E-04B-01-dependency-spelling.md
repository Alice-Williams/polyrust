# M35-01E-04B-01 — Typed dependency spelling policy

- Status: complete
- Parent: [M35-01E-04B](M35-01E-04B-java-imported-callables.md)
- Depends on: M35-01E-04A

## Implementation contract

- Replace the mandatory import field in DependencyCallableSpec with a closed
  DependencySpelling enum: FixedImport(ImportKind) or Qualified(QualifiedName).
  The dialect still reconstructs the entire spec from its opaque witness.
- Reuse the existing reference planner/resolver policies. A qualified dependency
  always carries its typed qualified name and produces no import allocation.
- Fixed native symbols remain globally reserved against other fixed imports and
  owned bindings, including unused registrations. Qualified symbols collide by
  their complete typed qualified name, not by the unqualified member name.
- Keep exact catalogue, per-reference and original-package post-link rederivation.
  A spelling policy, name or owner changed after linking must fail verification.
- C explicitly selects FixedImport and preserves its current output/diagnostics.
  No Java imported-call support is claimed by this shared checkpoint alone.

## Definition of done and tests

- Existing C/shared fixed-import, collision, no-alias, missing-symbol and tamper
  regressions pass unchanged in meaning.
- Independent shared-dialect tests cover two qualified owners with identical
  member names, overlap with an owned name, zero import directives, exact native
  qualified references and deterministic repeat linking.
- Reject qualified-path collisions and coordinated post-link policy/name/owner
  substitutions; unused registrations do not produce directives.
- Shared codegen/C Bazel targets, compile-negative contracts, Rustfmt, Clippy,
  Buildifier and docs gates pass. Review the shared change before B-02 uses it.

## Completion evidence

- Focused Linux Bazel gate `3e309208-e5e9-49d2-96b8-1b0ad2206a05`:
  15/15 test targets passed, including shared codegen (114 unit tests), C,
  Java, compile-negative contracts, Rustfmt, Clippy, Buildifier and docs.
- Fresh read-only Sol Extra High review found no core defects or material test
  gaps. Symmetric fixed-to-qualified tamper coverage was optional redundancy;
  exact witness and original-catalogue reconstruction already reject that swap.
- No Java consumer binding support is claimed by this shared checkpoint.
