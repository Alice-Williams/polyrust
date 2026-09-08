# M34A-11-01S — Encode the measured C scalar ABI

- Status: complete
- Depends on: M34A-11-02A
- Required by: M34A-11-02B

## Goal and independence

Encode the exact already-measured scalar compatibility, integer promotions and
usual arithmetic conversions in c/abi-type-model.md. This is a small independent
foundation extraction from 02B, permitted while amended contextual contracts
undergo review: it adds no expression, callable, ownership, verification,
rendering or Supports API. C02B itself retains its 00R prerequisite.

## Definition of done

- Exhaustive closed scalar matches implement the pinned 13-type ABI, including
  Int/I32 and U64/Size compatibility while preserving original AST spellings.
- PlainChar remains distinct from I8 despite equal signed layout.
- Promotions and mixed signed/unsigned/floating conversions match the measured
  model. Returned types are actual C types, not portable semantic conversions
  or proofs of arithmetic range/absence of undefined behavior.
- Keep implementation and tests in separate focused modules; no production
  test sources, new dependencies or renderer/text path.
- 02B consumes these rules rather than creating a second conversion table.

## Tests and proof

- Independent explicit 13-by-13 expected arithmetic matrix, all 13 promotions
  and all 169 compatibility pairs, retaining negative plain-char controls.
- Existing native ABI probes under pinned Zig/GCC at O0/O2 plus rejected
  unsigned-char configuration, C unit/compile-fail, Rustfmt/Clippy/Buildifier.
- Expand the independent compiler oracle to all 169 arithmetic pairs, not only
  representative mixed-signedness examples. Its test-only C macros do not enter
  the production grammar or generated packages.
- Full cached tracked/release/eight-target gates in the development container.
- Fresh read-only Sol Extra High review and documented finding dispositions.

## Commit gate

Commit and push this bounded prerequisite separately with M34A-11-01S. Record
exact invocations and hosted evidence; never report it as C02B or C completion.

## Implementation evidence (2026-09-08)

The focused scalar_abi module uses exhaustive input matches and a closed
five-variant promoted-rank enum; no default rank silently admits a new scalar.
The independent Rust matrix and native _Generic oracle cover all 169 ordered
arithmetic pairs. The native oracle remains repository-only source.

- Initial focused 4e3117a5-9fea-46b1-9757-5b94682e59b8: all nine C unit,
  compile-fail, native ABI, Rustfmt, Clippy, Buildifier and docs targets pass.
- Final tracked 1c4c0037-969e-4c3b-aaad-e99752f00d22: 439 rules / 314 tests pass,
  including the expanded native oracle under both compilers and optimizations
  and the unsigned-char rejected configuration. 49 tests executed normally.
- Release 03276f45-95b6-458f-b1b7-1884e2767545: all 251 tests pass.
- Conformance 96b22e35-3a62-4c71-8386-ed52128bfac9: evaluator and eight targets
  agree on 50 cases / one portable test with byte-identical repeated manifests.

At this implementation checkpoint, fresh review and hosted follow-up were pending. Contextual C AST,
resource certification and capability mapping work have not been implemented
by this scalar foundation.

## Independent review

Fresh Sol Extra High read-only review of 476d95b6d9264cccab0820633b4cbe9fed2d6500
found no core defects in the exact scalar rules, independent matrices, native
compiler wiring, source ownership or phase claims. Root accepts that result.
The reviewer ran no builds or network checks.

Two optional suggestions were evaluated separately:

- An enum/ALL inventory-cardinality guard is future Stage 04 grammar-inventory
  hardening; today's complete 13-item list and exhaustive production matches
  are correct. It is not a missing current scalar case.
- Making is_compatible_with const is API convenience, not a semantic or
  certification defect. Retain ordinary derived enum equality without adding
  a second comparison representation solely for const-call support.

Hosted run 34246605876 is pending; no cancelled ancestor run is counted as
successful hosted proof.

## Completion scope

The complete local gates and fresh independent review satisfy this bounded
foundation's exit contract. In accordance with the user's local-container
per-step instruction, the recorded pending hosted run is an integration
follow-up, not a reason to stall contextual work after its separate design gate.
Stage 09 still requires exact-final-commit hosted success; no pending/cancelled
run is counted as passed. The C02B/00R prerequisites are otherwise unchanged.
