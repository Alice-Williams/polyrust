# M34A-11-01 — Implement typed C identifiers, types and declarators

- Status: planned
- Depends on: M34A-11-00

## Goal

Establish the grammar foundation with small Rust modules, without publishing a partial certified backend.

## Definition of done

- Add ast/ and focused tests/ modules; keep tests excluded from the production Bazel source set.
- Use validated identifiers, closed C17 keywords, exact scalar enums, const-qualified object/pointer types, nonzero fixed arrays and exact non-variadic function prototypes.
- Separate void/function/object categories. Reject array/function returns and implicit parameter adjustment; represent function pointers and pointer/array nesting structurally.
- Parameter and return lists are unbounded recursive/container APIs, not arity-numbered helpers. Nominal registry/context checks arrive in the next slice.
- Expose no raw source/declarator string, renderer or claimed Supports capability. Existing CBackend remains explicitly legacy until cutover.

## Tests and proof

- Rust unit matrix: every primitive/type constructor; reserved words/prefixes; zero arrays; function/array pointer nesting; zero and many parameters; return and qualifier restrictions.
- Rust compile-fail examples: void object, function-as-object, wrong pointer category and invariant-wrapper construction.
- Bazel C Rust tests and doctests, Rustfmt, Clippy, Buildifier, docs, tracked/release gates; legacy native C and sanitizers remain green.

Test targets required by this slice must be added before it closes; proposed
future targets are not evidence of an existing implementation. All builds and
tests run in the Linux development container with pinned toolchains and normal
Bazel action/test caching.

## Commit gate

Record exact commands, counts, failures and dispositions here. Commit and push
this completed checkpoint with M34A-11-01 in the message. Keep the overall
C migration open until M34A-11-09; never substitute a partial slice for Pass.
