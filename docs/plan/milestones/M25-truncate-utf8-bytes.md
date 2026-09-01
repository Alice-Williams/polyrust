# M25 — Port truncate-utf8-bytes with UTF-8 budget proof

- Status: complete
- Phase: 6 compatibility continuation
- Depends on: M23, M24

## Outcome

Reimplement the complete representable typed behavior of the MIT-licensed
`parshap/truncate-utf8-bytes` 1.0.2 package at revision
`4212839ea184e74fb81f1e4e633e1db794ebe4f4`, then generate equivalent packages
in all eight supported languages.

## Implementation checklist

- Pin the immutable MIT license, package entry point, implementation, official
  tests, DefinitelyTyped declaration, and upstream-pinned naughty-string corpus.
- Specify a reusable UTF-8 byte-budget string truncation intrinsic whose budget
  retains the upstream `number` behavior.
- Implement and test the intrinsic in the checker, evaluator, and all eight
  generated runtimes without splitting a Unicode scalar.
- Carry every official fixed assertion and the complete pinned naughty-string
  corpus into permanent differential evidence.
- Add boundary, fractional, negative, infinity, NaN, combining, astral, NUL,
  and large-input vectors beyond the upstream suite.
- Run native format, lint/static, compile, sanitizer, and test gates for every
  generated package and prove deterministic regeneration.

## Required exit evidence

- The representable public signature `truncate(string, byteLength): string` is
  modeled as `String × F64 -> String`; integer, fractional, infinite, and NaN
  budgets match the pinned JavaScript oracle.
- Every valid-Unicode input in the official fixed and naughty-string corpus
  matches upstream for every byte boundary needed to reach the original string.
- Output is always a source prefix, is valid Unicode, never exceeds a finite
  nonnegative byte budget, and never splits a Unicode scalar.
- Three complete generations are byte-identical and every generated package
  passes its native gates.
- `bazelisk test //... --test_output=errors` and
  `bazelisk test //:release_gate --test_output=errors` pass in the Linux
  development container, including Buildifier, Rustfmt, and Clippy.

## Scope boundary

PolyRust `String` is a sequence of Unicode scalar values. JavaScript strings can
also contain isolated UTF-16 surrogate code units, which cannot be represented
as PolyRust strings and are excluded explicitly. All valid Unicode strings are
in scope. Dynamic non-string arguments are outside the pinned DefinitelyTyped
signature; all generated targets enforce their native string and floating-point
types.

The operation is a general string/UTF-8 primitive. It must not be named for this
package, branch on a target, or embed target-language source in checked IR.

## Completion evidence

- Seven retained upstream files pass an offline Git-blob provenance test. The
  exact MIT license, 1.0.2 code, official test, typed declaration, and all 483
  naughty-string entries are pinned immutably.
- `StringTruncateUtf8Bytes` is the 61st serialized v0 intrinsic. Its checked
  signature is `String × F64 -> String`; invalid shapes are rejected, and the
  evaluator covers exact, split, fractional, NaN, and infinity behavior.
- Thirty portable vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages.
- The pinned JavaScript oracle passes 25,303 input/budget comparisons over 486
  strings. It includes every integer boundary for the complete official corpus,
  special F64 budgets, upstream fixed cases, and two large inputs.
- Three generated manifests are byte-identical. All native format, lint/static,
  compile, test, public-consumer, C/C++ style, C ownership, and sanitizer gates
  pass.
- The port filled bit-exact C F64 portable-test emission, dynamic Go test imports
  for F64, and unsigned Go IEEE-bit decoding as reusable backend features.
- `bazelisk test //... --test_output=errors` passes all 153 tests in the
  complete repository suite.
- `bazelisk test //:release_gate --test_output=errors` passes all 131 release
  tests, including Buildifier, Rustfmt, and Clippy.
