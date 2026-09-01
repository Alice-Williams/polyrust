# truncate-utf8-bytes 1.0.2 compatibility port

## Provenance and scope

This port reproduces the complete representable typed public behavior of
[parshap/truncate-utf8-bytes](https://github.com/parshap/truncate-utf8-bytes)
at revision `4212839ea184e74fb81f1e4e633e1db794ebe4f4`. That revision contains
the published 1.0.2 implementation plus the upstream change that explicitly
offers the MIT license; the code is unchanged from the 1.0.2 tag.

The DefinitelyTyped declaration is pinned at revision
`451dc8fc19383bc12af59522020e571957f1684e` and defines the public API as
`truncate(string: string, byteLength: number): string`. The implementation,
official tests, package metadata, MIT license, declaration, and the
`big-list-of-naughty-strings` corpus at upstream's pinned submodule revision
`5f5a11b34b86f811e9888e32f3053d8cb1466325` are retained under
`third_party/truncate-utf8-bytes/`. An offline Bazel provenance test verifies all
seven retained files against their Git blob IDs.

All 483 entries in the official corpus are valid Unicode strings; 479 are
unique. PolyRust strings cannot represent isolated UTF-16 surrogate code units,
so those JavaScript-only values remain an explicit general string-model
boundary, but no official corpus entry is excluded. Dynamic non-string inputs
are outside the pinned declaration.

The package received 8,395,462 npm downloads in the measured week ending
29 August 2026.

## Portable implementation

M25 adds the general `StringTruncateUtf8Bytes` intrinsic with signature
`String × F64 -> String`. It scans the source once at Unicode-scalar
boundaries, tracks the cumulative UTF-8 byte length, and returns the longest
prefix permitted by the upstream comparison rules. It never slices inside a
scalar.

The floating-point budget intentionally retains JavaScript `number` behavior:

- an exact cumulative byte count includes that scalar;
- a fractional count returns the prefix before the first cumulative count that
  exceeds it;
- a negative finite value or negative infinity returns the empty prefix for a
  nonempty string;
- positive infinity and NaN return the complete string; and
- an empty input always returns empty.

The intrinsic is serialized in PolyIR, type-checked, evaluated by the reference
interpreter, and implemented in Rust, TypeScript, derived JavaScript, Python,
Go, Java, C++20, and C17. It is not named for the upstream package and contains
no target source in checked IR.

The port also closed two general backend gaps. C portable tests can now emit
bit-exact F64 arguments. Go test files emit `math` as a dynamic per-file import
only when a test value contains F64, and Go JSON decoding now parses the full
unsigned 64-bit IEEE representation rather than losing high-bit values such as
negative zero. The C runtime returns an owned prefix, validates UTF-8, zeros
failed outputs, and has direct forced-allocation-failure ownership coverage.

## Equivalence evidence

The permanent M25 suite proves:

- 30 portable vectors cover every UTF-8 width, exact and split boundaries,
  combining sequences, astral scalars, NUL, negative zero, fractional values,
  infinities, NaN, large budgets, and the fixed upstream examples;
- all 30 vectors pass in the evaluator and natively generated Rust,
  TypeScript, JavaScript, Python, Go, Java, C++, and C packages;
- the exact retained JavaScript implementation agrees for 25,303 input/budget
  comparisons over 486 strings, including every official naughty-string entry,
  every integer byte boundary, special F64 budgets, and two large inputs;
- every fresh package passes its formatter/style, linter/static checker,
  compiler, tests, and applicable C/C++ sanitizer gates;
- direct C ownership tests cover successful prefix allocation, forced
  allocation failure, zeroed failure output, invalid UTF-8, and safe cleanup;
- three complete generations are byte-identical; and
- the full repository and release gates pass, including Buildifier, Rustfmt,
  and Clippy.

Reproduce the port-specific proof in the Linux development container:

```sh
bazelisk test //examples/real-world/truncate-utf8-bytes:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port.
