# M35-03A-05A-04C-03 — Original Java canonical-owner imports

- Status: planned
- Parent: [Java canonical owner](M35-03A-05A-04C-java-type-owner.md)
- Depends on: [owner certificate](M35-03A-05A-04C-02-java-owner-certificate.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Contract and implementation

Replace blanket source-root dependency identities with full typed owner keys.
Retain exact immutable certificate authority; clones share it, independently
certified lookalikes do not. Source-only APIs use an optional checked source root.
Keep existing source-crate overlap checks as well as full-owner checks.

Import original family, success constructor/accessor and each error enum constant
through consumer-scoped typed bindings. Enum constants are values, never zero-arg
Error constructors or ordinal lookups. Preserve signature phases, original
membership and all direct unused imports through transitive/diamond closure.

Compiler manifests and graph publication remain source-only until 04D and must
explicitly reject canonical dependencies rather than emit a fake core root.

## Definition of done and tests

- Original family/value/member identities survive source relays; same-core
  distinct instances coexist, competing authorities and namespace collisions do
  not. Wrong enum role, foreign subtype and copied-family references reject.
- Unused owners still pay registration, traversal, source and classfile budgets.
  Exact/one-over closure tests exercise the new keys.
- Existing source-only dependency/function/constant behavior and output bytes
  remain unchanged. Manifest negative controls cover unused canonical imports.
- Full Linux Bazel/lint, independent GPT-6-SOL review, protected-WIP audit and
  scoped commit/push. Rust Result source admission remains closed.
