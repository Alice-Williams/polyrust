# M13 — Implement required Go backend

- Status: planned
- Phase: 3
- Depends on: M08, M09

## Outcome

Generate a readable, deterministic Go module for the full v0 IR and prove that
explicit pointers, value copying, slices, and nil do not leak target-specific
semantics into PolyRust programs.

## Implementation checklist

- Go name/keyword handling, escaping, imports, type mapping, and lowering.
- Constant lowering, interface/method generation, compile-time conformance
  assertions, restricted contract dispatch, and native `_test.go` generation.
- Reviewed tagged-enum, `Option`, and `Result` representations that distinguish
  every PolyRust value without relying on ambiguous nil conventions.
- Exact `int32`/`int64` checked/wrapping helpers where Go operators differ.
- Immutable list API/copy discipline for slice-backed storage.
- `go.mod`, generated tests/runner support, and pointer/value API design document.
- A pinned Go toolchain in CI or documented local provisioning, because Go is not
  currently installed on this machine.

## Required exit evidence

- Every v0 capability has declared support.
- Pointer use is an internal/public-layout choice with documented escape and
  alias analysis; no generated operation can mutate an aliased PolyRust value.
- `Option<T>` distinguishes `None` from `Some(zero value)` for every supported T.
- Every contract implementation has a compile-time assertion and every portable
  test becomes a discoverable Go test.
- Generated packages have deterministic imports/file layout and no reflection or
  `unsafe` package usage.
- Backend uses only public checked-IR/codegen interfaces.

### Verification

- Unit/golden tests for exported/unexported names, keywords, escaping, Unicode,
  numeric boundaries, variants, interfaces/implementations, portable tests,
  generics, imports, and every lowering case.
- Pointer/value tests that copy records and lists, mutate target-visible copies
  where permitted, and prove generated PolyRust results remain unaliased.
- Nil/zero-value matrix for nested `Option`, `Result`, lists, bytes, and records.
- Native generated-module checks:

```text
gofmt -d .
go vet ./...
go test ./...
```

- Static scan proving generated code does not import `unsafe` or `reflect`.

### Completion gate

All v0 fixtures are deterministic, native format/vet/test checks pass on the
supported Go matrix, pointer/slice invariants have direct tests, the backend
passes the contract suite, API snapshots are reviewed, and at least 20 evaluator
vectors agree with generated Go.

## Scope boundary

Portable raw pointers, goroutines/channels, cgo, reflection, and arbitrary Go
module dependencies.
