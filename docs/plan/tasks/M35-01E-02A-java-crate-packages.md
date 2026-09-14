# M35-01E-02A — Typed Java crate package paths

- Status: complete
- Parent: [M35-01E-02](M35-01E-02-java-source-identity.md)
- Depends on: completed M35-01E-01 and M35-01D

## Goal

Replace Java's single hard-coded namespace assumption with a typed, bounded
crate-derived package identity, without changing the legacy package.

## Implementation contract

- JavaPackage retains Generated and adds RustCrate(u64), using the stable rustc
  crate ID as naming data. It is not a compiler/source-validity certificate.
- Rust crate spelling is org.polyrust.generated.r followed by sixteen lower-case
  hexadecimal digits. Paths derive mechanically from the typed package.
- Each nonempty target package contains exactly one Java namespace. Empty
  certificates remain valid and render no files or namespace. Mixed namespaces
  reject before certification, preserving existing package-access assumptions.
  Separate Rust crates will have separate certificates joined in M35-01E-04.
- Runtime role/placement/shell remain confined to their existing exact identity;
  do not relocate or copy Runtime into a Rust crate namespace, including under
  PublicApi or Implementation roles. The top-level Runtime name is reserved
  globally; future source declarations with that spelling must be mapped.
- Renderer-qualified paths and every class-name/resource calculation use the
  actual typed package. No additional raw source/import pathway.
- RustCrate files have a provenance-neutral generated header; package naming
  alone must not claim checked CoreIR or Rust compiler provenance.

## Definition of done and tests

- Legacy name/path behavior is unchanged; crate IDs 0, 1 and u64::MAX produce
  distinct valid deterministic spellings and exact path roots.
- Wrong package/path, nested output paths, mixed namespaces and misplaced
  Runtime reject through existing unresolved/post-link verification.
- Structural fixtures with the same class name in distinct separately certified
  packages compile together under pinned Java 21 with all warnings denied.
- Actual-package descriptor/name resource boundaries have exact/one-over tests.
- Three-render determinism, all historical Java tests, full migration/lint/docs
  gate and fresh review pass.

## Scope boundary

This is target syntax/layout/resource support. RustSource provenance/member
authentication and compiler HIR lowering are not claimed by this child.

## Completion evidence

- Linux DevContainer full release/frontend/C/shared/Java/docs gate
  82f037bb-21af-4863-a296-47011f5917f6: 343/343 tests, 413 targets,
  226.481 seconds. Rust and Bazel lint and historical examples remain enabled.
- Native Java 21 oracle compiles registered constructors/types with identical
  class names in Generated and crate IDs 0, 1 and u64::MAX; three renders agree.
- Regression cases reject wrong paths, mixed namespaces and copied Runtime
  shells under Runtime, PublicApi and Implementation roles. Empty output is
  explicit; RustCrate headers make no CoreIR provenance claim.
- Actual-prefix record descriptor and binary-name capacity exact/one-over
  tests pass. Independent resource tests do not claim filesystem support for
  65-KiB source filenames.
- Original review findings were evaluated and repaired. Fresh Sol Extra High
  java_packages_fresh_review reported no blocking or non-blocking findings.
- Documentation/Buildifier follow-up b4eebeea-96db-4e5c-b5d1-3942709bd8a8:
  2/2 pass. No tests disabled; no commit/push before migration closure.
