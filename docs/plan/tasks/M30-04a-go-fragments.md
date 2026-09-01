# M30-04A — Migrate Go to dependency-complete fragments

- Status: planned
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
