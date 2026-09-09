# M34A-11-02D-03C — numeric flow

- Status: planned
- Depends on: M34A-11-02D-03B

## Goal

Convergent actual-graph range/relational flow with effect invalidation and point-local safety checking.

## Definition of done

- Private authenticated numeric storage keys/state; edge refinement uses typed polarity/cases/loop identity.
- Conservative monotone joins/widening, no timeout-as-success or giant loop enumeration.
- All evaluated expression children checked at converged inputs; short-circuit and conditional guard-local refinement.
- Writes and opaque effects invalidate aliases/globals while preserving never-address-taken local storage facts.
- Use actual proved loop evidence for counter<bound and safe increment; maintain nonwrapping-size/index obligations for storage/call composition.
- Known/generated calls contribute only sound return ranges with outstanding safety obligations; no summary is manufactured.
- Compose with ContextFacts and constants; dynamic pointer validity/ownership still 04/05.

The detailed numeric contract is in
[numeric range proof](../../specification/typed-generation/languages/c/numeric-range-proof.md).
The exact loop form remains [counted loops](../../specification/typed-generation/languages/c/counted-loops.md).
These slices implement the parent task; none weakens its complete-stage gate.

## Tests and proof

- Guard reversal/nondominance/staleness, joins, loops and unknown parameters; arithmetic bounds before execution.
- Allocation products distinguish numeric modulo from checked extent, with wrong/missing preguard controls.
- Zero-bound dead paths vs unknown iteration, short-circuit division guards and float NaN guards.
- Full cached tracked/release/lint/eight-target plus final independent review; parent03 closure checklist.

## Commit gate

Record focused and full cached Linux Dev Container evidence and an independent
uncapped Sol Extra High review. Commit and push M34A-11-02D-03C separately. Keep
parent 02D-03 open until every slice and its combined checklist pass. No partial
numeric or loop fact is a C safety, allocation or rendering certificate.
