# M30-04 — Migrate every target plugin

- Status: in-progress

## Goal

Apply the proven fragment and helper-graph contract to every supported target.

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
