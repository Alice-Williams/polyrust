# M34A-11-01S — Encode the measured C scalar ABI

- Status: in-progress
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

Fresh immutable review and hosted confirmation remain open. Contextual C AST,
resource certification and capability mapping work have not been implemented
by this scalar foundation.
