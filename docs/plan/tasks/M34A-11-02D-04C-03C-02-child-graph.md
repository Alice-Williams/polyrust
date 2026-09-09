# M34A-11-02D-04C-03C-02 — Finite exclusive child graphs

- Status: planned
- Depends on: M34A-11-02D-04C-03C-01

## Goal

Derive parent-child edges and complete-owner commits from actual AST actions.

## Definition of done

- Private local/child locations and finite allocation graph retain exact roots,
  source resets, provenance and children of uncommitted partial parents.
- Actual active payload and typed member/element roles establish required,
  optional and metadata obligations. Unannotated pointers fail closed.
- Commit requires complete exclusive children; reject duplicate parents,
  owning cycles, shallow aliases and incorrect allocation bases.
- Compressed fixed-array defaults do not enumerate bounds. Missing/unequal
  evidence, joins and fallback never discard outstanding child resources.
- Lift registration-only rejection only for implemented actual graph cases;
  unresolved transfer/destruction/helper effects remain diagnostics.

## Tests and proof

- Actual required/optional/nested/active-union/fixed-array positives and missing
  child/wrong arm/shallow copy/duplicate edge/cycle/premature commit mutations.
- Complete memory but invalid ownership, partial parent attached children,
  zero defaults and large fixed-array bounds; private graph/join controls.
- Full focused/tracked/release/lint/deterministic gates and uncapped review.

## Commit gate

Record evidence, commit and push; continue 03C-03 without closing parent 03C.
