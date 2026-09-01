# trim-newlines 5.0.0 compatibility port

## Provenance and scope

This port reproduces all runtime behavior of
[sindresorhus/trim-newlines](https://github.com/sindresorhus/trim-newlines) at
revision `6980540ee683a660fd82cb1bda37bf1ebd989179`. The revision is MIT
licensed and exposes three functions:

```text
trimNewlines(string) -> string
trimNewlinesStart(string) -> string
trimNewlinesEnd(string) -> string
```

The pinned license, runtime source, TypeScript declaration, and official tests
are retained under `third_party/trim-newlines/`. Upstream's TypeScript
conditional types compute more precise literal-string return types. PolyRust
keeps the complete runtime string domain and result but intentionally exposes
the cross-target `String -> String` type instead of a TypeScript-only
compile-time evaluator.

## Portable implementation

M18 adds two generic intrinsics: trim-start and trim-end by a set of Unicode
scalar values. The checked program passes `"\\r\\n"` as that set. The
both-boundaries function composes start then end; the other public functions use
one operation each.

All implementations inspect only the requested boundary and stop at the first
non-member scalar. Rust uses `trim_*_matches`, TypeScript iterates
`Array.from` scalars with a `Set`, Python uses `lstrip`/`rstrip`, and
Go uses `strings.TrimLeft`/`TrimRight`. The evaluator supplies the normative
behavior.

## Equivalence evidence

The permanent M18 suite proves:

- all 24 official functional vectors and seven additional Unicode/whitespace
  boundaries pass in the evaluator and all seven generated packages;
- all three generated TypeScript functions match the pinned upstream for
  107,851 unique inputs and 323,553 function comparisons;
- the differential corpus exhausts short strings over CR, LF, ordinary text,
  spaces, tabs, a non-BMP scalar, and Unicode line separator;
- three 90,000-newline inputs cover both-only, start-only, and end-only large
  boundaries and agree with upstream;
- each generated package passes its native formatter, static checker/linter,
  compiler, and tests; and
- three independent generations are byte-identical.

Reproduce the port evidence in the Linux dev container:

```sh
bazelisk test //examples/real-world/trim-newlines:all --test_output=errors
```

M18 is also part of `//:release_gate`, including Rustfmt, Clippy with warnings
denied, Buildifier, documentation, dependency policy, and the previous
real-world port.
