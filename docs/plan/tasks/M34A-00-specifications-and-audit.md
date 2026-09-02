# M34A-00 — Accept specifications and baseline audit

- Status: complete
- Depends on: M34-02

## Goal

Make every shared phase and every supported output language an explicit,
reviewable contract before production migration starts.

## Definition of done

- ADR-0004 is accepted and names the complete typed-generation specification.
- Nine shared layer specifications define inputs, outputs, ownership,
  invariants, forbidden escapes, and required proof.
- Rust, TypeScript, derived JavaScript, Python, Go, Java, C++20, and C17 each
  have a complete per-language specification covering capabilities, AST/type
  mapping, interfaces, symbols, helpers, files, rendering, validation, tests,
  and legacy deletion.
- A normative coverage matrix has no missing language/layer owner.
- The new compliance ledger records the shared surface and all eight outputs as
  failing, with source evidence, rather than inheriting M30 pass labels.
- Conflicting legacy architecture/ABI documents are explicitly historical.
- M34-03 is formally blocked and its uncommitted partial package is excluded.
- M34A has one dependency-ordered implementation task per shared layer and per
  language, each with a definition of done and executable tests.
- Documentation tests pass; the checkpoint is committed and pushed.

## Tests

- `bazel test //tools/docs:documentation_test --nocache_test_results --test_output=errors`
- `git diff --check`
- Manual link/coverage audit recorded in
  `docs/specification/typed-generation/coverage.md`.

## Commit gate

Commit as `M34A-00: specify typed target-AST pipeline` and push only after the
tests above pass in the Linux development container. Do not include
`examples/real-world/stdlib-abs`.
