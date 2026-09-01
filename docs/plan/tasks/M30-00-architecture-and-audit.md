# M30-00 — Freeze the strict architecture and audit every target

- Status: complete

## Goal

Turn compositional dependency ownership into a normative contract and measure
the shared codegen layer plus every supported target against it before migration.

## Definition of done

- The architecture defines fragments, helper nodes, units, files, groups, and
  renderers with explicit MUST/MUST NOT rules.
- Source-role bypasses, parallel dependency scans, fixed runtime inventories,
  and raw directive text have unambiguous compliance outcomes.
- Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C each have an
  evidence-backed baseline row.
- No row is marked compliant merely because generated code compiles.
- The migration tasks close every failed row and require executable evidence.

## Tests

- `bazel test //tools/docs:documentation_test`
- Manual evidence audit using repository-local source paths recorded in the
  compliance ledger.
