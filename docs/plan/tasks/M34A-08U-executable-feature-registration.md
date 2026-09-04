# M34A-08U — Register executable typed feature mappings

- Status: in-progress
- Depends on: M34A-08T
- Blocks: M34A-10U and every remaining typed-language migration

## Goal

Replace empty target-support witnesses with consuming plugin-builder
registrations which carry executable, typed mappings, and complete the typed
frontend's PolyIR v0 intrinsic surface.

## Definition of done

- `Supports<F>` exposes an associated registered mapping rather than acting as
  an empty marker.
- A consuming typestate plugin builder changes exactly one `Missing` slot to
  `Implemented<M>` through `.support::<F>(mapping)`.
- Registration requires `M: FeatureMapping<D, F>` and duplicate, missing,
  wrong-feature, wrong-dialect, and wrong-output registrations do not compile.
- `Supports<F>` is derived only from an `Implemented<M>` slot; backends do not
  write manual support implementations.
- Runtime capability presence is derived from the same registration catalogue
  and retains shape-specific fallible checks.
- The typed frontend exposes `Char`, `Bytes`, `List<T>`, `Option<T>`, and
  `Result<Ok, Error>` without untyped vectors or erased generic values.
- Every PolyIR v0 intrinsic has a typed constructor with exact operand/result
  types and its own inferred feature family.
- Replacement pairs and homogeneous lists use recursive typed structures;
  invalid list elements and malformed replace-many calls cannot be expressed.
- No mapping returns source strings, tokens, imports, helpers, or unchecked
  target AST.

## Tests

- Compile-pass one expression for every intrinsic and replay the resulting
  program through checker and CoreIR verification.
- Compile-fail wrong integer width, string/bytes confusion, heterogeneous list,
  list element mismatch, option fallback mismatch, result-branch mismatch,
  malformed replacement list, and private proof construction.
- Compile-pass a plugin containing a required subset.
- Compile-fail missing, duplicate, wrong-feature, wrong-dialect, and
  wrong-target-AST registrations.
- A catalogue inventory test proves every public constructor has a feature
  marker and every PolyIR v0 intrinsic is covered exactly once.
- Run Rustfmt, strict Clippy, Buildifier, documentation tests, full tracked
  Bazel tests, and the release gate in the Linux development container.

## Commit gate

Commit and push the specification checkpoint before implementation. Commit and
push `M34A-08U/M34A-10U: register executable Java mappings` only after every
shared and Java proof passes. Hosted CI for the exact implementation SHA must
be green before either task is complete.

## Exit evidence

Pending implementation.

