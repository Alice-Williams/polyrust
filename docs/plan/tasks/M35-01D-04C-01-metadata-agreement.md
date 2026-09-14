# M35-01D-04C-01 — Pinned compiler metadata agreement

- Status: complete
- Depends on: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)
- Parent: [M35-01D-04C](M35-01D-04C-rust-crate-dependency-driver.md)

## Definition of done

Probe the pinned compiler's stable crate identity and content hash in local
analysis, emitted metadata and a consuming analysis using explicit extern paths.
Reuse the closed compiler configuration. Establish same-process sequential
compiler invocation behavior before designing the production driver around it.
The test probe is not a public certificate constructor or new generation path.

## Tests and proof

- Matching local analysis and loaded metadata agree for unchanged source/config.
- Same crate/item names and signatures with changed function bodies do not
  satisfy content agreement; inspect stable identity separately from content.
- Source documentation and explicit crate key changes are tested separately.
- Explicit aliases resolve to the defining foreign crate and declaration.
- Repeated analysis is deterministic under the supported input/configuration;
  document any source-path sensitivity instead of silently ignoring it.
- Compiler analysis still rejects invalid source/private access; no arbitrary
  user compiler flags are introduced by a test-only metadata emitter.
- Bazel probe, compiler Clippy, Rust formatting, buildifier and docs pass.

## Scope boundary

Production dependency graph parsing, metadata actions, typed import registration
and atomic multi-package output remain the following slices. No generated C
claim is made by this compiler-only experiment.

## Completion evidence

- Pinned metadata/local/loaded probe and lint/format/docs gate
  `7b308974-6f7d-4749-82c8-d045e70c8b32`: all four targets passed.
- Distinct-crate sequential compiler invocation and frontend gate
  `d4b0adcc-6639-4c89-a52c-6381a94c49aa`: all 48 tests passed.
- Full gate `c79cd70a-be6a-45e4-8e9e-2139babc31f9`: all 313 tests
  passed across 354 targets with test caching enabled.
- Fresh broad read-only Sol Extra High review found no actionable core errors.
- Follow-on shared configuration library refactor preserved all 48 frontend
  tests in `a869c5b6-893c-4322-8a77-e70b36c099a9`.

The experiment discovered that crate hashes include output mode and logical
source filenames. Matching metadata mode and explicit source-directory remapping
produce agreement across processes/sandboxes. Body/doc mutations keep the
declaration signature/identity but alter the content hash; key changes alter
crate identity. A logical filename change also alters content identity and is
not silently ignored. The normative dependency contract records these results.
