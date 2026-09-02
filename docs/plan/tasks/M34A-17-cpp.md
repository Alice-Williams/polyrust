# M34A-17 — Migrate C++20 to typed generation

- Status: planned
- Depends on: M34A-16

## Goal

Generate C++20 through a complete typed C++ AST with explicit ownership,
header/source resolution, and composition-based type erasure.

## Definition of done

- C++ owns all type/declarator/qualifier, expression, statement,
  declaration/definition/member, special-member, heritage, file/package, and
  template enums in its specification.
- Every CoreIR feature has an exhaustive C++ strategy; exact admitted `std`
  types/functions/methods/operators have typed ownership/lifetime/signature
  catalogue entries.
- Sequencing, conversions, arithmetic UB avoidance, overloads, value categories,
  cv/ref/lifetimes, copy/move/drop, complete types, ODR, and visibility are
  verifier checked.
- Portable interfaces use owning flat type-erased value/operation tables, not
  abstract bases; multiple conformance, nested interface values, dispatch,
  copy/move/drop, and delegation pass.
- Optional `CppHeritage` accepts only a final one-edge external adapter and
  rejects generated/multiple/virtual/reuse chains.
- Includes, forward declarations, namespaces, declarations/definitions,
  helpers, and files are resolver-derived.
- Runtime declarations/definitions are C++ AST and strict templates render
  resolved views.
- `CppCode`, raw runtime source/section parsing, manual includes/helpers,
  abstract-base portable contracts, and the legacy pipeline are deleted.
- The C++20 compliance row moves to **Pass** with exact evidence.

## Tests

- `bazel test //crates/backend-cpp:all --nocache_test_results --test_output=errors`
- C++ AST/verifier/catalogue/include/helper/placement/overload/template matrices.
- Hermetic C++20 strict compile/native tests, separate public-header consumer,
  negative fixtures, interface ownership/heritage corpus, and three-generation
  determinism.
- GCC 14.2.0 AddressSanitizer and UndefinedBehaviorSanitizer gates.
- All M17-M33 C++ historical port targets.

## Commit gate

Commit and push `M34A-17: migrate C++20 to typed AST` only when all C++ and
shared typed-generation gates pass in the dev container.
