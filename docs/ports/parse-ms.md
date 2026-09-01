# parse-ms 3.0.0 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[sindresorhus/parse-ms](https://github.com/sindresorhus/parse-ms) 3.0.0 at
revision `49dab09236deeea5d2c082182e2c73e7a79763a8`. The source, TypeScript
declaration, type test, official runtime tests, package metadata, README, and
MIT license are retained under `third_party/parse-ms/`. An offline Bazel test
verifies all seven files against their Git blob IDs, including the upstream
test file's exact trailing blank line.

Version 3 exposes one typed function from JavaScript `number` to a seven-number
`TimeComponents` object. PolyRust models that complete API as
`F64 -> TimeComponents`, with `F64` fields for days, hours, minutes, seconds,
milliseconds, microseconds, and nanoseconds. Dynamic non-number calls are
outside the declaration. Version 4 adds JavaScript `bigint`; it is deliberately
deferred until PolyRust has a specified arbitrary-precision integer type.

At selection time on 1 September 2026, the package reported 24,963,960 weekly
npm downloads and 171 npm dependents.

## Portable implementation

M27 adds the general `FloatTrunc` intrinsic. Its checked signature is
`F64 -> F64`; it rounds finite values toward zero, preserves positive and
negative zero, propagates NaN, and preserves infinities. The model combines it
with existing floating division, multiplication, and truncating remainder to
mirror the upstream calculations directly.

Portable expectation equality is now distinct from language equality. The
language operations `Equal`, `NotEqual`, and `ListContains` keep IEEE behavior:
NaN is not equal to itself and the two zeros compare equal. Test comparison
recurses through lists, options, results, records, and enums, treats NaN as a
valid expected class, and distinguishes signed-zero bit patterns. This lets a
portable test specify exact observable outputs without changing program
semantics.

The evidence-driven port closed several reusable backend gaps:

- TypeScript and derived JavaScript stringify wide F64 bit payloads in both
  program and portable-test language IR before emitting JavaScript numbers.
- Python now implements IEEE division/remainder special cases without leaking
  Python `ZeroDivisionError` or `ValueError` behavior.
- Go now evaluates the complete F64 arithmetic family, recursively compares
  values, and converts runtime record maps into generated typed record results.
- C now lowers scalar-field record construction, selects named-record call
  result wrappers, exposes standard-math F64 helpers through the runtime, and
  links `libm`; `<math.h>` is owned by the runtime language unit's import set.
- Java and C++ now keep semantic equality separate from exact test equality.
- Rust and C portable tests recursively compare F64 values inside records.

## Equivalence evidence

The permanent M27 suite proves:

- all ten official positive assertions and the upstream negative symmetry
  behavior, plus signed zero, NaN, infinities, unit boundaries, fractions, and
  maximum finite binary64, through 30 portable vectors;
- all 30 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 10,105 deterministic inputs agree with the retained JavaScript implementation
  for all seven fields, totaling 70,735 exact component comparisons;
- every fresh package passes its native formatter/style, static analysis,
  compiler, and tests, including C/C++ sanitizer gates; and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazelisk test //examples/real-world/parse-ms:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port. At completion, the uncached full-repository gate
passes 168/168 tests and the uncached release gate passes 146/146 tests in the
Linux development container; both include Buildifier, Rustfmt, and Clippy.
