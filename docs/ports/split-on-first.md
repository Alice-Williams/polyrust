# split-on-first 3.0.0 compatibility port

## Provenance and scope

This port reproduces the complete string-only typed behavior of
[sindresorhus/split-on-first](https://github.com/sindresorhus/split-on-first)
3.0.0 at revision `d6bf86163df4e6490b134c303477644a52736997`.
The implementation, TypeScript declaration, runtime and declaration tests,
package metadata, README, and MIT license are retained under
`third_party/split-on-first/`. An offline Bazel test verifies all seven files
against their exact upstream Git blob IDs.

The portable API is
`split_on_first(input: String, separator: String) -> List<String>`. Empty
input, an empty separator, or an absent match returns an empty list. A match
returns exactly two strings around the leftmost literal occurrence.

The upstream declaration says `[string, string?]`, but the retained
implementation and official tests return `[]` for every no-split case. The
portable type therefore models the exact runtime result shape instead of
claiming a tuple invariant the source does not satisfy. Version 4.0.0 later
adds arbitrary JavaScript `RegExp` separators; that is a different API and is
not claimed here. JavaScript strings containing lone surrogate code units are
outside the PolyRust scalar-string domain.

## Portable implementation

The checked model contains no split-on-first-specific operation. It composes
existing Boolean, option, prefix-removal, and list operations with two new
general string operations:

- `StringIndexOfLiteral(String, String) -> Option<I64>` returns the leftmost
  exact, case-sensitive match in Unicode scalar units, `Some(0)` for an empty
  needle, or `None` when absent.
- `StringSliceScalars(String, I64, I64) -> String` takes a half-open range,
  independently clamps both endpoints to the scalar bounds, and returns empty
  for a reversed or empty range.

For well-formed strings, JavaScript's UTF-16 lookup offsets and PolyRust's
scalar offsets identify the same substring boundaries. The model slices the
prefix and matched tail, then removes the known separator from that tail. No
backend observes or converts the index into its native storage unit outside
its own language mapping.

The port also closes three general output gaps it exposed. C now constructs
`List<String>` values with incremental initialization, deep-owned returns,
and complete failure cleanup. Go converts the admitted raw IR `List<String>`
result into a typed `PolyList<string>` and compares public containers through
their semantic runtime representation. Rust emits inference-safe assertions
for empty list expectations. None of these paths contains package-specific
logic.

Every new mapping preserves the M30 compositional target-language IR contract:
syntax, structured dependencies, and helper roots are carried together;
runtime closure is selected before rendering; renderers only spell resolved
directives; and JavaScript is mechanically derived from TypeScript.

## Equivalence evidence

The permanent M32 suite proves:

- all six official result assertions plus 26 boundary cases, covering absence,
  empty operands, boundaries, overlaps, repeated and multi-scalar separators,
  NUL, CR/LF, combining sequences, BMP, and astral scalars;
- all 32 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 58,274 deterministic admitted inputs agree exactly with the retained
  JavaScript implementation;
- an external C consumer injects failure at every allocation point and passes
  strict C17, ASan leak/double-free checks, and UBSan; and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazel test //examples/real-world/split-on-first:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port. Repository-wide gate counts and hosted-CI evidence
are recorded in the M32 milestone.
