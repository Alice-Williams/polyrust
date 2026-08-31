# Go backend v0

The `org.polyrust.go` backend emits a deterministic Go 1.25 module from checked
IR. Public integer types are exactly `int32` and `int64`; records are value
structs; contracts are interfaces with compile-time conformance assertions;
`Option` and value-level `Result` carry explicit tags and therefore distinguish
zero values. Runtime failures use `PolyResult[T]`.

Portable lists and bytes use wrappers with unexported slices. Constructors copy
input slices, `Values` returns a copy, and append creates a new wrapper. This
keeps slice capacity and aliasing out of portable semantics. Generated records
and contract receivers are values. Pointers are limited to the private optional
error payload and the internal interpreter receiver; no public portable value
requires pointer identity or mutation. Generated code imports neither `unsafe`
nor `reflect`.

The generated runtime consumes only checked v0 IR. The native Bazel gate uses
the complete `MODULE.bazel`-pinned Go SDK to run `gofmt`, `go vet`, and
`go test`, plus a forbidden-import scan, one Go test per portable test, and 20
evaluator-aligned boundary/copy vectors.
