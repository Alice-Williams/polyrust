# M35-03A-02F-02B-05C-01 — Typed compiler constant re-export inventory

- Status: complete
- Parent: [cross-crate export closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05B
- Specification: [shared re-export contract](../../specification/typed-generation/rust-constant-reexports.md)

## Contract

Preserve checked compiler DefIds alongside the finite public binding graph.
Introduce a private typed foreign module constant descriptor, separate from
owned local declarations. Extended classification resolves direct/renamed/
transitive aliases to original defining identities without target strings.

Retain the existing strict production constructor and all foreign-export
rejection tests. This checkpoint supplies source facts only: neither C nor Java
publication gains an unchecked or partially implemented export path.

## Definition of done and tests

- Real checked producer/intermediate/root fixtures prove constant-only aliases,
  renamed/transitive/diamond bindings, owned/foreign separation, nested modules,
  original stable/DefId identity and repeated cached inventory agreement.
- Foreign functions/modules/types/associated constants and private exports reject
  at compiler or inventory boundaries as appropriate.
- Combined owned/foreign declaration counts obey the existing 4096 bound;
  finite local module cycles remain finite and existing traversal limits pass.
- Private fields cannot be constructed by external probe code. Scalar value/type
  admission remains separate from classification.
- Existing production C/Java foreign-export tests remain enabled and rejecting.
- Focused compiler tests, Rust/Bazel linters, full Linux release gate, fresh
  independent review and scoped commit/push precede completion.

## Implementation and review evidence

The compiler export collector now retains exact DefIds alongside its finite
stable-ID graph. Cache publication installs graph and definition map together.
Both local and foreign selected declarations are checked against the retained
compiler definition. A dedicated private ForeignConstantDeclaration prevents a
foreign DefId from being passed off as an owned LocalDefId.

The extended classification constructor admits foreign ordinary module constant
bindings, while the existing production constructor remains owned-only.
Classification accepts a foreign string constant as a source fact without
claiming a scalar target mapping. Default production rejection is asserted in
every positive foreign-export probe.

Six focused targets passed: real compiler inventory, private construction,
existing inventory/private construction, Rust formatting and Bazel lint.
The pre-hardening tree dfc73286720dd4d8e4d1c8608a4b66a08b827707 passed all
738 tests / 1092 targets (254.054 seconds; invocation
f6cce954-cab8-411e-821b-9c817643e555).

The first reviewer requested an explicit local DefId cross-check in addition to
stable identity. This invariant was accepted and implemented. The suggested
specific unsupported-export/non-exported-body collision was not demonstrated:
the existing effective-visibility check rejects a non-exported body owner, and
the export collector detects two exported definitions sharing a stable ID.
Nevertheless direct DefId equality is clearer and closes the cross-map
consistency obligation without relying on those indirect checks.

Test-only retained-map corruption now swaps one DefId while leaving the binding
graph unchanged. Local and foreign probes require their exact mismatch
diagnostics; all six focused tests pass on tree
d26391e32c451a37b6a1011f4b519a5041d088d6. No production warning was suppressed.
The strengthened tree d26391e32c451a37b6a1011f4b519a5041d088d6 passed the full
Linux dev-container Bazel `//... //:release_gate`: 1093 targets, 738/738 tests,
55 executed and 683 cached, 244.698 seconds. Invocation:
1d56bbfe-1148-4c27-b463-b457d3570e98.

The original reviewer rechecked the fix and found no remaining core error.
A fresh independent Sol Extra High reviewer found no concrete core defect or
material proof gap after tracing both identity paths, mutation controls,
visibility/kind checks, finite graphs, counts and strict production rejection.
Its optional module-node hash-collision hardening is outside this declaration
inventory step. Extra macro and repeated-alias boundary fixtures are non-blocking:
namespace rejection, finite graph bounds, alias deduplication and combined
unique-declaration limits already have direct coverage. These are not claims
that future target export mappings are implemented.

Completion documentation is included in the final isolated exact-tree gate
before the scoped commit and push. No unrelated ownership changes or generated
artifacts are included.
