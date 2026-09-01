# html-escaper 3.0.3 compatibility port

## Provenance and scope

This port reproduces the complete typed public behavior of
[WebReflection/html-escaper](https://github.com/WebReflection/html-escaper) at
implementation revision `c6e2b50d7b6f486afb3ddc92bfcfec89857b75d7`.
The independent DefinitelyTyped declaration is pinned at revision
`cd61c555bfc93e985b313263a42ed78074570d08`. Together they define two public
functions: `escape(str: string): string` and
`unescape(str: string): string`. The implementation, declaration, official
tests, and MIT license are retained under `third_party/html-escaper/` with Git
blob hashes matching the pinned revisions.

The JavaScript implementation also coerces booleans and numbers and rejects
other dynamic values. Those behaviors are outside the retained string-only
type declaration. Each generated package enforces its native string type.

## Portable implementation

M23 adds the general `StringReplaceMany` intrinsic. It takes a source followed
by one or more needle/replacement pairs, all strings. At each Unicode scalar
boundary it selects the first matching needle, appends that pair's replacement,
and advances over the matched source. Replacement output is never rescanned.
Empty needles have defined scalar-by-scalar insertion behavior, so the
operation is total and deterministic rather than relying on a target regex
engine.

The checked `escape` function supplies the five upstream character mappings.
The checked `unescape` function supplies all ten named and numeric entity
spellings. Both are ordinary uses of the reusable intrinsic: there is no HTML
operation, target switch, embedded target source, or project-specific backend
path. Ordered simultaneous replacement is essential for inputs such as
`&amp;lt;`, which must decode to `&lt;` rather than recursively to `<`.

The operation is permanently validated in the IR serialization table, checker
valid/invalid signature tests, evaluator priority/Unicode/empty-needle vectors,
and Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C runtimes. The C17
implementation uses view arrays and one allocation after an overflow-checked
size pass; ownership tests cover non-recursive output, forced allocation
failure, a zeroed failure result, and safe destruction.

## Equivalence evidence

The permanent M23 suite proves:

- all four official upstream assertions and 38 additional entity, nesting,
  boundary, Unicode, NUL, and unknown-input vectors pass in the evaluator and
  every generated package;
- both generated functions match the pinned ESM implementation for 108,498
  function/input comparisons over 54,249 unique strings;
- the corpus exhausts four-token strings over entity fragments, every escaped
  character, astral Unicode, and NUL, then adds two large repeated inputs;
- fresh Rust, TypeScript, JavaScript, Python, Go, Java, C++, and C packages pass
  native formatting/style, static checks, compilation, tests, and the applicable
  C/C++ sanitizer gates;
- three independent generations are byte-identical; and
- the complete 138-test repository suite and 116-test release gate pass,
  including Buildifier, Rustfmt, and Clippy.

Reproduce the port-specific proof in the Linux dev container:

```sh
bazelisk test //examples/real-world/html-escaper:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port.
