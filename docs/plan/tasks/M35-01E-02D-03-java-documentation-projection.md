# M35-01E-02D-03 — Typed Java documentation projection and rendering

- Status: complete
- Parent: [M35-01E-02D](M35-01E-02D-java-source-documentation.md)
- Depends on: completed M35-01E-02D-02

## Implementation contract

- Extend ResolvedJavaFileItem with a typed documentation attachment collection.
  Owners distinguish registered symbols, source record fields with their nominal
  owner, and source modules. Payloads are only JavaDocComment values; no raw
  target strings or caller-provided file paths enter the projection.
- Derive declaration attachments by walking actual file-item declarations and
  joining their registered origins. Interface implementation methods and aliases
  must not copy the primary interface/declaration documentation.
- A single emitted synthesized PackageEntryPoint owns crate/module presentation.
  Crate-root attributes describe that facade. Other canonical module attributes
  appear once as explanatory block comments, ordered by depth then stable identity;
  they do not become public Java declarations. Require the facade when nonempty
  module docs need presentation; do not silently drop or duplicate them.
- Preserve attribute order per owner and component/declaration placement. Keep
  empty-doc historical output byte-identical. Render normalized text using fixed
  delimiters and prefixes, never by interpreting attributes as templates or code.
- Existing post-link exact file-item rederivation must include attachments.
  Private linked-package state remains private; no mutation backdoor for tests.
- Account for attachment count and normalized presentation bytes, including
  comment delimiters, line prefixes and indentation, before certification.
  Input is bounded by the shared metadata checker before normalization.
- The initial presentation policy permits at most 100,000 attached owners and
  64 MiB of rendered comment text per package. These are target-resource limits,
  not a claim about javac's intrinsic comment capacity.

## Definition of done and tests

- Crate, nested/private module, type, callable and component attributes appear at
  the correct generated owner, once, with ordered multi-attribute text.
- Conflicting/missing facade ownership and misplaced/extra/missing attachments
  reject. Mutation tests exercise the same exact projection/equality used by
  post-link validation; compile-fail tests preserve private linked state.
- Exact and one-over count/presentation budgets use the production implementation.
- Hostile text passes through production lowering, certification and rendering;
  generated Java 21 compiles with warnings denied and runs a separate scalar
  consumer. Injection controls and reflection detect added executable members.
- Three renders agree; empty-doc legacy fixtures are unchanged. Existing private
  access and native C/Rust/Java regressions remain enabled.
- Full gate and fresh independent review pass before parent completion.

## Scope boundary

This completes target documentation infrastructure. Compiler HIR mappings and
cross-crate Java output remain M35-01E-03 through M35-01E-05. It does not claim
that arbitrary Rust programs can already be translated to Java.

## Completion evidence

- Full container gate `1d9bfd30-4a69-4830-9406-7dca023ca0db` passed 343/343
  tests across 413 targets; the Java suite passed all 248 tests with none ignored
  or filtered. Rust/Bazel lint, template policy, docs, native and compile-negative
  regressions all remain enabled.
- Fresh Sol Extra High review found no core defects in owner projection,
  presentation accounting, post-link equality, private linked state or native
  injection controls.
- The preceding full gate `eddb722c-6448-4734-973b-9aa6290ba576` passed 342 tests
  but caught three wildcard renderer matches. Replaced them with exhaustive
  enum variants; no policy was suppressed and no rendering behavior changed.
