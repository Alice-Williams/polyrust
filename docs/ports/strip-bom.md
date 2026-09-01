# strip-bom 5.0.0 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[sindresorhus/strip-bom](https://github.com/sindresorhus/strip-bom) at revision
`b80d7bc94e79b4744d92a2dc6328c91d9afe9775`. The package is MIT
licensed and exposes one `stripBom(string: string): string` function. Its
pinned license, source, declaration, and official test source are retained
under `third_party/strip-bom/`.

Generated packages use each backend's established identifier convention,
including `strip_bom` in Rust, TypeScript, and Python and `StripBom` in
Go. The upstream runtime's non-string JavaScript exception is outside its
declared TypeScript domain; each generated API enforces a native string type.

## Portable implementation

M20 adds the target-independent `StringStripPrefix` intrinsic with signature
`(String, String) -> String`. It removes exactly one leading substring and
leaves the source unchanged when the prefix is absent or empty. The checked
program applies that operation with U+FEFF as its prefix. It contains no target
switch, raw target source, byte indexing, or project-specific intrinsic.

The real-world proof also closed two general Go backend gaps. Go string
serialization now escapes embedded U+FEFF because the Go scanner rejects a
literal byte-order mark inside source, and generated locals avoid every Go
keyword and predeclared identifier so a parameter named `string` cannot
shadow its return type.

## Equivalence evidence

The permanent M20 suite proves:

- both official fixture behaviors and 16 additional boundary vectors pass in
  the evaluator and every generated package;
- generated TypeScript matches the pinned upstream for 55,991 unique strings;
- the differential corpus exhausts strings through length six over U+FEFF,
  ASCII, astral Unicode, a combining scalar, NUL, and newline;
- a 90,000-BOM input loses exactly one leading scalar while an equally large
  input with an ordinary leading character remains byte-identical;
- fresh generated Rust, TypeScript, Python, and Go packages pass their native
  formatter, static checker/linter, compiler, and tests; and
- three independent generations are byte-identical.

Reproduce the port-specific evidence in the Linux dev container:

```sh
bazelisk test //examples/real-world/strip-bom:all --test_output=errors
```

The same suite is mandatory through `//:release_gate`, together with all
previous ports and repository Rust/Bazel linters.
