# stdlib is-negative-zero 0.2.3 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[stdlib-js/math-base-assert-is-negative-zero](https://github.com/stdlib-js/math-base-assert-is-negative-zero)
0.2.3 at lightweight tag commit
`766200b9eeea46b7f827ac7d63effa6bea65d896`. The JavaScript entry point and
implementation, TypeScript declaration and declaration test, JavaScript and C
runtime tests, C implementation/header, package metadata, README, NOTICE, and
Apache-2.0 license are retained under
`third_party/stdlib-is-negative-zero/`. An offline Bazel test verifies all 12
files against their exact upstream Git blob IDs.

The declaration admits exactly `isNegativeZero(x: number): boolean`, so the
portable API is `is_negative_zero(value: F64) -> Bool`. The retained
JavaScript tests also pass a string, booleans, null, undefined, arrays,
objects, and a function. Those values are useful evidence about untyped
JavaScript coercion, but the retained declaration rejects them and they are
not silently claimed by the PolyRust numeric port.

At selection time on 31 August 2026, the package reported 274,168 npm
downloads for the measured week ending 29 August 2026.

## Portable implementation

M33 adds the reusable `FloatIsNegativeZero(F64) -> Bool` intrinsic. Its result
is true exactly when the input's raw IEEE-754 binary64 representation is
`0x8000000000000000`. Ordinary equality cannot express this because positive
and negative zero compare equal. A generic sign test is also incorrect because
negative normal/subnormal values, negative infinity, and negative NaNs must
return false.

The checked package model is one function containing one use of that general
intrinsic. It contains no stdlib-specific operation or target branch. Each
target translator owns its lowering:

- Rust compares `to_bits` values with no import or helper;
- TypeScript and derived JavaScript use `Object.is(value, -0)` with no import;
- Python uses `math.copysign` through its optional F64 helper closure;
- Go compares `math.Float64bits` through its optional F64 helper closure;
- Java compares `Double.doubleToRawLongBits` with `Long.MIN_VALUE` and adds no
  import;
- C++ combines equality with `std::signbit` in the execution helper that
  already owns `<cmath>`; and
- C calls `poly_f64_is_negative_zero` through the mapping-owned optional F64
  runtime root, whose source fragment owns `<math.h>`.

These are dependency-complete target-language fragments under the normative
[compositional architecture](../language-ir-architecture.md). There is no
fixed import inventory, directive-bearing body string, post-render repair,
package-specific lowering, or JavaScript fork from the TypeScript runtime.

## Equivalence evidence

The permanent M33 suite proves:

- all distinct official typed numeric assertions, plus signed
  normal/subnormal boundaries, both infinities, and signaling/quiet NaNs with
  both signs, through 22 exact-bit portable vectors;
- all 22 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 86,018 deterministic exact-bit inputs agree with the retained JavaScript
  implementation. The corpus covers both signs, every exponent, five
  mantissa boundaries per sign/exponent pair, and 65,536 seeded full-width
  samples reconstructed losslessly with `DataView`;
- an external C consumer verifies the public scalar ABI under strict C17,
  ASan, and UBSan, while Java and C++ generated public consumers also pass;
  and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazel test //examples/real-world/stdlib-is-negative-zero:all --nocache_test_results --test_output=errors
```

The suite contributes 17 permanent targets to `//:release_gate`. At the local
implementation checkpoint, the uncached repository-wide gate passes 250/250
tests and the uncached release gate passes 227/227 tests, including
Buildifier, Rustfmt, Clippy, all earlier compatibility ports, dependency
boundaries, source-policy/fault-injection checks, public consumers, and C/C++
sanitizers.
