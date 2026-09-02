# M32 — split-on-first 3.0.0 equivalence port

- Status: complete
- Phase: 6
- Depends on: M31

## Outcome

Port the complete typed behavior of MIT-licensed `split-on-first` 3.0.0 into
one checked PolyRust model and generate equivalent Rust, TypeScript,
JavaScript, Python, Go, Java, C++, and C packages.

The portable API is
`split_on_first(input: String, separator: String) -> List<String>`. It returns
an empty list when either input is empty, the separator is empty, or no match
exists; otherwise it returns exactly two strings containing the prefix and
suffix around the leftmost literal match. This preserves the runtime result
shape more honestly than the upstream declaration, whose
`[string, string?]` return type excludes the officially tested empty result.

## Architecture contract

M32 adds only two target-independent operations:

1. `StringIndexOfLiteral(String, String) -> Option<I64>` searches
   case-sensitively for the leftmost exact substring and reports a Unicode
   scalar offset. The empty needle returns `Some(0)`.
2. `StringSliceScalars(String, I64, I64) -> String` returns a half-open scalar
   range after clamping each endpoint to `[0, scalar_len(input)]`; an endpoint
   order with start greater than or equal to end returns the empty string.

The checked model guards the upstream empty-input/empty-separator cases, finds
the separator, and derives the two slices. No split-on-first-specific intrinsic,
raw backend source, target semantic branch, or direct runtime import is allowed.
Every backend must lower these operations through dependency-complete
compositional language fragments as required by M30.

The runtime result is an empty or two-element `List<String>`. M32 therefore
also completes C17's existing `List<String>` ABI by adding allocator-safe
`ConstructList` expression lowering. This is a backend completeness gap, not a
new portable semantic operation; all other backends already construct lists.

JavaScript indexes and slices by well-formed UTF-16 code units. PolyRust uses
Unicode scalar offsets for both operations. For admitted well-formed Unicode
strings, a match starts and ends on scalar boundaries, so this internal choice
produces the same prefix and suffix even when astral scalars precede or form the
separator. Lone surrogates remain outside the PolyRust `String` domain.

## Implementation checklist

- Retain implementation, declaration, declaration test, runtime test, package
  metadata, README, and MIT license from tag `v3.0.0`, annotated tag object
  `3f4a69e8e1715e4e60060d2f04ccc68a1305c96f`, commit
  `d6bf86163df4e6490b134c303477644a52736997`.
- Verify every retained byte against its upstream Git blob ID without network
  access at test time.
- Implement, serialize, check, evaluate, and lower both general string
  operations in every supported backend.
- Complete C17 dynamic `List<String>` construction with single-evaluation,
  partial-initialization cleanup, and allocation-failure proof.
- Retain all six valid official result assertions and the invalid-input oracle
  evidence; add empty, absent, boundary, overlap, combining, BMP, astral, NUL,
  CR/LF, and repeated-separator vectors.
- Differentially compare the model with the exact retained JavaScript
  implementation over a deterministic cross-product corpus.
- Generate, format, lint, compile, and natively test all eight packages,
  including public Java/C/C++ consumers and C/C++ sanitizers.

## Local completion evidence

- The retained implementation passes provenance verification for all seven
  pinned blobs.
- The six official assertions plus 26 boundary vectors pass in the evaluator
  and all eight generated packages. The deterministic differential oracle
  agrees on all 58,274 admitted comparisons.
- The port suite passes 17/17 targets, including three-generation
  determinism, public consumers, strict native toolchains, and C/C++
  sanitizers. The external C allocation harness also passes every observed
  allocation-failure edge.
- The complete uncached repository gate passes 233/233 tests. The complete
  uncached release gate passes 210/210 tests, including every earlier
  native/differential port, Buildifier, Rustfmt, Clippy, both source-policy
  targets, public consumers, and sanitizers.
- Hosted workflow
  [33592407791](https://github.com/Alice-Williams/polyrust/actions/runs/33592407791)
  passes implementation commit
  `84a81eb92f54cfbc37a4dd6013bee036c14d4939` across both determinism hosts,
  cross-host manifest comparison, pinned/stable Rust, fast checks, and
  cache-cold/cache-warm complete release gates.

## Required exit evidence

- Provenance tests pass for all retained upstream blobs.
- Checker and evaluator tests cover both new operations, invalid signatures,
  empty needles, absent matches, clamping, reversed endpoints, and Unicode
  boundaries.
- Every backend has focused positive and negative lowering coverage for both
  operations.
- Every official and expanded portable vector passes in the evaluator and all
  eight generated targets.
- The differential oracle reports every admitted comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.
- The completed milestone is committed, pushed, and green in hosted CI before
  another repository is selected.

## Scope boundary

M32 pins version 3.0.0 because its complete public input domain is
`String × String`. Version 4.0.0, released later, adds arbitrary JavaScript
`RegExp` separators. That is a different API version and remains a future
candidate for a specified portable regex subset; M32 does not claim v4
compatibility. Invalid JavaScript argument types are retained as oracle
evidence but fall outside v3's declared input domain.
