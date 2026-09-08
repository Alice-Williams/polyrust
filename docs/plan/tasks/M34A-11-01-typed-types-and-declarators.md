# M34A-11-01 — Implement typed C identifiers, types and declarators

- Status: complete
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

## Checkpoint evidence (2026-09-08)

- `4a4ac669-6dd1-475f-ac03-9890674f0872`: C unit/doctest targets,
  Rustfmt, Clippy, Buildifier and documentation all pass.
- `14e24036-bc6b-4f23-88bf-316fe02e170a`: all 436 tracked rules build;
  all 311 tests pass, including native C consumers and sanitizers.
- `deaea5d8-f72d-4f3b-9ef6-6b1713ed9cbc`: all 248 release tests pass.
- `c6190365-bff1-4f9b-b068-330283e84863`: 50 cases and one portable test;
  evaluator/eight-target agreement and repeated-manifest determinism pass.
- Supplementary Linux `cargo +1.98.0 test -p polyrust-backend-c
  --all-features --locked`: 21 unit tests, eight compile-fail doctests and
  one positive doctest pass. Native proof is supplied by Bazel, not Cargo.

The first runs caught a leading blank line and the source-policy assumption
that legacy import assertions were still inside an inline test module. Both
were corrected without relaxing production policy; the complete rerun passed.
New production modules remain small and tests are excluded from the library
source set. Legacy generation remains untouched except test organization.

The independent design review also requested a closed grammar inventory and
identified contradictory nested-interface equality wording; both specifications
are corrected here. Its shared typed-record equality counterexample is accepted
for a separate prerequisite repair before C contextual implementation. Passing
this type-shape checkpoint does not establish typed entry totality or C Pass.
