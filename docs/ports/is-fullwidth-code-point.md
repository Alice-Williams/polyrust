# is-fullwidth-code-point 3.0.0 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[sindresorhus/is-fullwidth-code-point](https://github.com/sindresorhus/is-fullwidth-code-point)
3.0.0 at revision `80e5e314d86e5f76bd1b0573aa9d33e615a372db`.
The implementation, TypeScript declaration, type test, runtime test, package
metadata, README, and MIT license are retained under
`third_party/is-fullwidth-code-point/`. An offline Bazel test verifies all seven
files against their Git blob IDs.

Version 3 declares the complete public API as `number -> boolean` and contains
its Unicode range table directly. PolyRust therefore models the whole typed API
as `F64 -> Bool`. Dynamic non-number JavaScript inputs are outside that
declaration. Later upstream versions delegate the decision to a separately
versioned Unicode-width package and consequently define a different
multi-upstream evidence case; this port makes no claim about their newer table.

At selection time on 1 September 2026, the package reported 265,568,331 npm
downloads for the measured week ending 29 August 2026 and 51 GitHub stars.

## Portable implementation

M28 adds the reusable `FloatIsNaN` intrinsic with checked signature
`F64 -> Bool`. It recognizes every IEEE-754 NaN payload and sign and rejects
finite values, signed zeros, and infinities. The checked model composes that
predicate with ordered comparisons, equality, and short-circuit Boolean
operators to express all 15 accepted intervals and the upstream U+303F hole.
There is no package-specific classifier intrinsic.

Every target translator lowers `FloatIsNaN` through language IR. Required math
dependencies belong to the relevant language unit and are resolved by the
package renderer; generated files contain no model-authored import block. The
C runtime supplies the predicate behind its normal runtime unit boundary.

The unusually large canonical model and test document also exposed two general
compiler limits. Java now renders long serialized documents as `String.join`
over bounded chunks, and C++ builds them from bounded `std::string` chunks.
The C++ language unit requests `<string>` only when that lowering needs it, so
the import remains dependency-derived. Focused 100,000-byte regression tests
protect both fixes.

## Equivalence evidence

The permanent M28 suite proves:

- all six official assertions plus every accepted-range edge, the U+303F hole,
  fractional boundaries, signed zero, subnormals, signed NaNs, infinities,
  maximum finite values, and out-of-Unicode-domain inputs through 89 portable
  vectors;
- all 89 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 22,409 deterministic numeric inputs agree with the exact retained JavaScript
  implementation, including every boundary neighborhood, a broad lattice, and
  20,000 seeded random values;
- every fresh package passes its formatter/style, static analysis, compiler,
  tests, and applicable C/C++ sanitizer gates; and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazelisk test //examples/real-world/is-fullwidth-code-point:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port. At the implementation checkpoint, the uncached
full-repository gate passes 183/183 tests and the uncached release gate passes
161/161 tests in the Linux development container; both include Buildifier,
Rustfmt, and Clippy. Hosted
[CI run 33547403633](https://github.com/Alice-Williams/polyrust/actions/runs/33547403633)
also passes for the implementation commit, including cache-cold and cache-warm
complete release gates.
