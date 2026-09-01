# escape-string-regexp 5.0.0 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[sindresorhus/escape-string-regexp](https://github.com/sindresorhus/escape-string-regexp)
at immutable revision
`cbc42403142c96923b482604e1f3d627b1956aff` (version 5.0.0). The pinned
revision is MIT licensed; its license, implementation, upstream metadata, and
official test source are retained under
`third_party/escape-string-regexp/`.

The upstream TypeScript declaration exposes one function:

```text
escapeStringRegexp(string: string): string
```

PolyRust represents this entire valid typed domain. JavaScript's dynamic
non-string `TypeError` is outside that domain and becomes a static parameter
type error in every generated package.

## Portable implementation

The checked program is authored in
`examples/real-world/escape-string-regexp/src/lib.rs`. It uses only
`StringReplaceAll`: first it doubles existing backslashes, then it replaces
each other regular-expression syntax character with its escaped spelling, and
finally replaces hyphen with `\\x2d`.

This sequential formulation is equivalent to upstream's character-class
replacement:

1. every needle is one distinct character;
2. backslash is processed first;
3. later passes introduce only backslashes, never their own needle; and
4. hyphen is absent from the first upstream character class and remains the
   final pass in both algorithms.

The implementation has no target switch, embedded target source, regular
expression dependency, or oracle dependency. One checked program generates
Rust, TypeScript, Python, and Go.

## Equivalence evidence

The permanent Bazel suite proves:

- all three official upstream cases plus 15 portable boundary vectors pass the
  reference evaluator and native tests in every generated package;
- the generated TypeScript function is byte-for-byte equal to the pinned
  upstream implementation for 3,750 unique inputs: every ASCII scalar, every
  pair and triple of the 15 escaped characters, repeated characters, controls,
  combining text, and non-BMP Unicode;
- every differential output also constructs an anchored Unicode JavaScript
  regular expression that matches its original input;
- generated Rust passes `rustfmt`, `clippy -D warnings`, compilation, and
  tests;
- generated TypeScript passes Prettier, `tsc`, and Node tests;
- generated Python passes compilation, Ruff format/lint, strict Mypy, and
  Pytest;
- generated Go passes `gofmt`, `go vet`, and `go test`; and
- three independent generations are byte-identical.

Reproduce the port-specific evidence in the Linux dev container:

```sh
bazelisk test //examples/real-world/escape-string-regexp:all --test_output=errors
```

The repository release gate additionally runs this suite through
`//:release_gate`, so a backend, linter, native compiler, oracle, or
determinism check cannot be omitted silently.

## Gaps discovered and filled

M17 added target-independent global literal string replacement to the IR,
checker, evaluator, builder surface, and all four backends. The port also found
and permanently regressed two emitter issues: unused stable imports in
function-only generated Python modules and invalid Go literals for Rust-specific
control/Unicode escape spellings.

There are no known behavior omissions within the upstream typed API. This port
does not claim that escaping arbitrary text makes it safe for every possible
regular-expression syntactic context; upstream documents the same boundary.
