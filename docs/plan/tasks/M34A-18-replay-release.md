# M34A-18 — Delete legacy paths, replay history, and release

- Status: planned
- Depends on: M34A-10 through M34A-17

## Goal

Prove the replacement architecture is exclusive, behavior-preserving, and
green on clean local and hosted systems before resuming M34.

## Definition of done

- The shared and all eight language compliance rows are **Pass**, with exact
  source and test evidence.
- Legacy `LanguageFragment` executable APIs, target `*Code` string builders,
  executable `RawText` paths, runtime source constants/includes/marker parsing,
  manual dependency attachment, and paired JavaScript generation are deleted.
- Production source policy rejects every known escape while allowing only
  path-scoped handwritten upstream snapshots, native consumers, metadata,
  documentation, and renderer-private literal presentation.
- The backend author/API/architecture/portable-language/C-C++ ABI docs describe
  the implemented ADR-0004 API rather than migration notices.
- The external backend example uses the sealed typed adapter and proves a new
  dialect cannot bypass phases.
- Every M17-M33 real-world package regenerates in all eight outputs and passes
  evaluator, native, retained upstream differential/oracle, dependency/helper
  minimality, public-consumer/sanitizer where applicable, and three-generation
  tests.
- Clean cache-cold and cache-warm repository/release gates pass, including
  Buildifier, Rustfmt, Clippy, every language linter/static checker, compilers,
  and native tests.
- M34 is changed from blocked to in-progress at M34-03; its partial legacy
  generated package is rebuilt through ADR-0004 rather than reused.
- The final checkpoint is committed, pushed, and hosted CI is green.

## Tests

- `bazel test //tools/policy:typed_generation_source_policy_test --nocache_test_results --test_output=errors`
- `bazel test //examples/real-world/... --nocache_test_results --test_output=errors`
- `bazel test //... --nocache_test_results --test_output=errors`
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
- Repeat the release gate cache-warm; verify `git status --short` contains no
  generated output or Bazel links.
- Inspect the pushed GitHub Actions run and record its URL/SHA in the milestone
  and compliance ledger.

## Commit gate

Commit and push `M34A-18: complete typed target-AST migration` only after all
local gates pass. Mark M34A complete only after hosted CI is green, then resume
M34-03 in a new checkpoint.
