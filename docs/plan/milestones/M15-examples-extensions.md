# M15 — Deliver examples, author guide, and external backend proof

- Status: complete
- Phase: 5
- Depends on: M06, M10, M11, M12, M13, M14

## Outcome

Demonstrate the intended user workflow and prove backend extensibility outside the
core workspace's concrete target crates.

## Implementation checklist

- End-to-end `models-and-validation` Rust generator example producing all four
  target packages without hand edits. It includes constants, a restricted
  contract and implementation, concrete and abstract dispatch, and at least ten
  portable tests.
- Author guide covering builder API, semantics, diagnostics, capabilities,
  regeneration workflow, and unsupported features.
- Backend author guide and template using only public APIs.
- External toy backend fixture built as a separate workspace/package.
- Generated-code review guide for each target's public API conventions.

## Required exit evidence

- One documented command generates all four example packages.
- One test command runs the common tests through the evaluator and all generated
  native frameworks.
- Fresh generation passes all native checks and conformance tests.
- Deleting generated output and regenerating restores identical bytes.
- External toy backend registers, preflights, emits a manifest, and passes the
  shared backend contract tests without editing core target-name branches.
- Guide clearly distinguishes Rust host code, PolyRust portable code, and
  generated Rust target code.

### Verification

- Documentation snippets compile/run as doctests or scripted tests.
- Clean temp-directory walkthrough follows every author-guide command.
- Example generated packages expose and run every portable test natively.
- External backend builds against the public versioned API with no path to
  private crates/modules.
- Link checker validates internal documentation links.

### Completion gate

A new contributor can follow the guide from a clean environment, regenerate and
test all four packages, and create the toy backend; all snippets are automated;
no step relies on undocumented local state; and screenshots/manual edits are not
required for correctness.

## Scope boundary

Publishing packages, production SDK generation, IDE plugins, and alternative
authoring syntax.

## Exit evidence

- `//examples/models-and-validation:generate` builds one checked Rust-authored
  module and emits 23 files across fresh Rust, TypeScript, Python, and Go
  packages. The model includes a constant, two records, a restricted contract
  and implementation, concrete and abstract dispatch, and ten portable tests.
- `//examples/models-and-validation:all` passed all five proofs: the reference
  evaluator, every generated native test framework and linter, byte-identical
  deletion/regeneration, the external backend contract, and documentation/link
  checks. The literal clean generation walkthrough also produced all 23 files.
- The separately rooted `examples/external-backend` package depends only on
  public versioned crates and proves registration, explicit preflight, manifest
  emission, and `check_backend_contract` without a core target-name branch.
- The author, backend-author, and generated-review guides cover builder use,
  semantics, diagnostics, capabilities, unsupported features, regeneration,
  extension requirements, and all four targets' public API conventions. The
  guide explicitly separates Rust host, PolyRust portable, and generated Rust
  code, and its links and required command snippets are tested.
- Cargo Rustfmt and warning-denied Clippy passed. The authoritative
  `bazelisk test //...` gate passed all 31 tests across 66 targets, including
  Buildifier, Bazel Rustfmt/Clippy, and all pinned native toolchains.
