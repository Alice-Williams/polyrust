# normalize-newline 5.0.0 compatibility port

## Provenance and scope

This port reproduces the complete typed value behavior of
[sindresorhus/normalize-newline](https://github.com/sindresorhus/normalize-newline)
5.0.0 at revision `bc6982d73ebd62de3729435d9baf8731ca274f7a`.
The implementation, TypeScript declaration, runtime test, package metadata,
README, and MIT license are retained under `third_party/normalize-newline/`.
An offline Bazel test verifies all six files against their Git blob IDs.

Version 5 declares two type-preserving overloads: `String -> String` and
`Uint8Array -> Uint8Array`. PolyRust has no overload resolution, so the model
exposes these as the explicit functions `normalize_newline` and
`normalize_newline_bytes`. Together they admit every value in the typed API,
including arbitrary invalid-UTF-8 byte sequences. Dynamic JavaScript calls with
values outside the declaration are not part of this compatibility claim.

PolyRust `Bytes` values are immutable. The generated APIs therefore promise the
same returned bytes, not JavaScript typed-array object identity or post-return
mutation. Upstream always returns a fresh `Uint8Array`, so this boundary loses
no admitted result value.

At selection time on 1 September 2026, the package reported 88,373 npm
downloads for the measured week ending 29 August 2026.

## Portable implementation

The text overload composes the existing `StringReplaceAll` operation to replace
CRLF with LF. M29 adds the reusable `BytesReplaceAll` intrinsic with checked
signature `Bytes x Bytes x Bytes -> Bytes` for the binary overload. Replacement
is literal, global, left-to-right, and non-overlapping, and replacement output
is never rescanned. An empty needle inserts at every byte boundary, including
both ends. There is no newline-specific primitive.

The evidence-driven port also closed three reusable backend gaps:

- C portable tests now construct and compare byte arguments and expectations
  through the generated runtime's ownership-safe byte views.
- Python's byte-replacement helper returns canonical immutable `bytes` rather
  than a tuple.
- Go portable byte literals use `NewPolyBytes`, including an explicit non-nil
  empty value, preserving the generated runtime representation.

Every target lowers the operation through its language IR. JavaScript is
mechanically derived from the TypeScript package. The current Java runtime unit
still declares a coarse, hard-coded whole-runtime dependency inventory; M30's
compositional language-IR milestone records the required fragment dependency
graph and the policy test that will remove that remaining abstraction leak.

## Equivalence evidence

The permanent M29 suite proves:

- all 13 valid official assertions plus Unicode, NUL, mixed-newline,
  repeated-match, invalid-UTF-8, boundary, all-octet, and non-cascading cases
  through 31 portable vectors;
- all 31 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 9,338 deterministic text inputs and 31,847 deterministic byte inputs agree
  with the exact retained JavaScript implementation;
- every fresh package passes its formatter/style, static analysis, compiler,
  tests, and applicable C/C++ sanitizer gates; and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazelisk test //examples/real-world/normalize-newline:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port. Completion gate counts and hosted-CI evidence are
recorded in the M29 milestone.
