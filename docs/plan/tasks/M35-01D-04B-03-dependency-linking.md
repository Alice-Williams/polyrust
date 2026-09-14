# M35-01D-04B-03 — Typed external C linking

- Status: complete
- Depends on: [M35-01D-04B-02](M35-01D-04B-02-import-registration.md)
- Parent: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)

## Definition of done

Implement in two ordered, independently tested slices:

1. [03A — Package-derived catalogues](M35-01D-04B-03A-package-catalogues.md).
2. [03B — Certified external C symbols](M35-01D-04B-03B-c-dependency-symbols.md).

Extend shared symbol linking with exact certificate-backed dependency entries,
preserving static catalogue behavior for existing languages. Reconstruct the
dependency catalogue and references from original checked package authority,
including post-link verification. Keep imported C native names fixed; derive
and deduplicate public-header imports from actual registered calls.

## Tests and proof

- Positive multi-symbol/shared-header and independent dependency cases.
- Missing/extra/retargeted imports, catalogue mutations, private headers, aliasing
  of fixed native names, symbol/path collisions and unused invented edges reject.
- Renderer only spells resolved witnesses; no source-text import construction.
- Complete shared/Java/default-catalogue and C import/compile-fail regressions.
- Resource certification still refuses imported calls until 04B-04 evidence exists.
- Existing C local checks remain mandatory. 03B feeds their scalar-call analysis
  exact certified effects so consumers can reach linking; 04B-04 still composes
  transitive stack bounds and enables final certification, not 03B.
- Linux Bazel style/policy gates and fresh independent review pass.

## Completion evidence

03A and 03B are complete. Final 03B full invocation
`c9da358d-2579-463a-bb65-25930383e883` passed all 311 tests across 351
targets. Both repair and fresh Sol Extra High reviews found no remaining core
error. See the child tasks for regression counts and explicit review decisions.
Dependency resource certification remains deliberately closed until 04B-04.
