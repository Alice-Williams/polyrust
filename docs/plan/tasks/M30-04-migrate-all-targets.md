# M30-04 — Migrate every target plugin

- Status: in-progress

## Goal

Apply the proven fragment and helper-graph contract to every supported target.

## Ordered target tasks

Each target is an independently committed and pushed checkpoint. A later task
MUST NOT use an earlier target's renderer or syntax; it reuses only the shared
fragment and helper-graph algebra.

1. [M30-04A — Go](M30-04a-go-fragments.md)
2. [M30-04B — Python](M30-04b-python-fragments.md)
3. [M30-04C — TypeScript and derived JavaScript](M30-04c-ecmascript-fragments.md)
4. [M30-04D — Rust](M30-04d-rust-fragments.md)
5. [M30-04E — C++](M30-04e-cpp-fragments.md)
6. [M30-04F — C](M30-04f-c-fragments.md)

This parent task becomes complete only after all six target tasks are complete
and every corresponding compliance-ledger row is `Pass`.

## Definition of done

- Rust, TypeScript, JavaScript, Python, Go, C++, and C mappings return
  dependency-complete fragments.
- Runtime/support templates are split into helper nodes where a monolithic body
  forces unrelated imports or includes.
- No backend performs a separate feature scan whose only purpose is to repair
  imports after emitting syntax.
- JavaScript remains mechanically derived from TypeScript where syntax permits.
- All eight renderers remain unable to inspect `CheckedProgram`.

## Tests

- Per-target minimal/feature-bearing import matrices.
- Every backend unit, native, conformance, public-consumer, ABI, and sanitizer
  target.
- Three-generation determinism for every real-world example.
