# M30-04A — Migrate Go to dependency-complete fragments

- Status: complete
- Depends on: M30-03

## Current audit finding

`generated.go` declarations are accumulated into one `String`.
`runtime.go` receives a fixed seven-import inventory, including an otherwise
unused `encoding/binary` placeholder. Portable tests separately rescan nested
values to decide whether `math` is needed.

## Definition of done

- Go type, value, declaration, portable-test, and runtime mappings return
  dependency-complete fragments.
- The nested-value F64 repair scan and `encoding/binary` placeholder are deleted.
- Runtime feature nodes own their Go imports and checked-program fragments select
  roots; no optional package is imported by an empty program.
- `GoImport` validates import paths and only `GoRenderer` spells `import`.

## Required tests

- Empty plus one-feature matrices for `bytes`, `math`, `strconv`, `strings`,
  `unicode/utf8`, and test-only `testing` requirements.
- Nested F64 test values prove dependency propagation without a second walk.
- `gofmt`, `go vet`, generated native tests, conformance, and public consumer.
- Three-generation byte determinism.

## Completion evidence

- `GoCode` owns target text and structured imports; nested type, value,
  parameter, result-conversion, declaration, and portable-test composition
  preserves requirements. `generated.go` and `generated_test.go` are fragment
  folds rather than file-sized dependency repair units.
- `GoImport::parse` rejects empty, rendered, rooted, repeated-separator,
  traversal, and invalid-character paths. Only `GoRenderer` spells directives.
- The runtime template is an ordered helper registry with independent
  checked-integer, F64, bytes-replace, scalar-length, replace-many,
  truncate-UTF-8, and decode-UTF-8 roots. Common JSON/string infrastructure is
  explicit. The unused `encoding/binary` placeholder is deleted.
- Exact empty and one-feature matrices prove the presence and absence of
  `bytes`, `encoding/json`, `math`, `strconv`, `strings`, and `unicode/utf8`.
  `StringToUtf8` specifically proves that bytes output does not imply the UTF-8
  validation package.
- F64 literals attach `math` in their value fragment, including through nested
  records. The former nested-value F64 repair scan is deleted.
- Go backend/native tests run `gofmt`, `go vet`, and `go test`. The entire
  130-test real-world corpus passes, including generated packages for every
  optional Go runtime root and every determinism/differential oracle.
