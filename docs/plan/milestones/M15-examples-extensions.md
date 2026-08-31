# M15 — Deliver examples, author guide, and external backend proof

- Status: planned
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
