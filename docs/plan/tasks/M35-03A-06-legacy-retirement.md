# M35-03A-06 — Corpus cutover and legacy runtime retirement

- Status: planned
- Parent: [M35-03A](M35-03A-runtime-free-parity.md)
- Depends on: M35-03A-02 through M35-03A-05 and M35-02

## Contract

Migrate all affected C/Java examples, conformance, public consumers, CLI and
benchmarks to the new system with feature parity. Only then remove legacy
runtime code and the obsolete generation architecture that requires it.

## Definition of done and tests

- Every inventory row has native replacement evidence and all affected corpus
  cases pass; retain license provenance and existing golden behavioral vectors.
- Public entry points have an explicit tested migration path, not an unnoticed
  disappearance of C/Java from the registry or a hidden old-generator fallback.
- Delete C runtime templates/string generator and Java custom runtime builders,
  helper catalogues, special runtime AST/file/render paths when no longer used.
- Remove obsolete build/dependency declarations, fixture-only adapters, policy
  exemptions and stale current guidance; retain portable independent tests.
- Search/code-boundary tests detect residual mandatory runtime dependencies;
  generated artifacts are ordinary target packages and are not committed.
- Keep shared typed AST, certification, provenance and required ownership code.
  Preserve unrelated unfinished M34 edits and historical completion evidence.
- Full native/lint/release gates, fresh review and exact-tree tested commit/push.
