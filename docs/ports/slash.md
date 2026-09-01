# slash 5.1.0 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[sindresorhus/slash](https://github.com/sindresorhus/slash) at revision
`98b618f5a3bfcb5dd374b204868818845b87bb2f`. The package is MIT
licensed and exposes one `slash(path: string): string` function. Its pinned
license, source, declaration, and official tests are retained under
`third_party/slash/`.

Generated packages expose a named `slash` function instead of JavaScript's
default-export syntax. The input domain and returned string behavior are
otherwise complete.

## Portable implementation

The checked program first applies `StringStartsWith` to the exact
extended-length prefix `\\\\?\\`. An `if` expression returns these
paths unchanged; its other branch replaces every backslash literally with a
forward slash. The program uses only existing portable semantics and contains
no target switch, raw target source, regular expression, or path-library
dependency.

## Equivalence evidence

The permanent M19 suite proves:

- all four official vectors and eleven boundary vectors pass in the evaluator
  and every generated package;
- generated TypeScript matches the pinned upstream for 55,994 unique paths;
- the differential corpus exhausts strings through length six over backslash,
  slash, question mark, drive punctuation, ordinary text, and Unicode;
- a 90,000-backslash ordinary path is fully converted and an equally large
  extended-length path remains byte-identical;
- fresh generated Rust, TypeScript, JavaScript, Python, Go, and Java packages pass their native
  formatter, static checker/linter, compiler, and tests; and
- three independent generations are byte-identical.

Reproduce the port-specific evidence in the Linux dev container:

```sh
bazelisk test //examples/real-world/slash:all --test_output=errors
```

The same suite is mandatory through `//:release_gate`, together with the
previous ports and repository Rust/Bazel linters.
