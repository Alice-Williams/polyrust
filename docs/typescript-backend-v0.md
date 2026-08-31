# TypeScript backend v0

The `org.polyrust.typescript` backend emits a dependency-free, strict ESM
package for Node 24 and TypeScript 7. Generated public declarations preserve
nominal record/contract intent with readonly classes and explicit `implements`
clauses. Contract dispatch remains restricted to implementations proven by the
checker.

`i32` and `f64` use JavaScript `number`; every integer-producing operation uses
the fixed-width runtime helpers. `i64` is always `bigint`, and wide literals are
serialized as decimal strings before JavaScript parses the embedded checked IR.
`Option` and value-level `Result` are discriminated unions. Runtime failure uses
the separate `PolyResult<T>` union rather than exceptions.

Strings are validated as Unicode scalar sequences where scalar semantics are
required. Bytes and lists are exposed as readonly arrays, construction copies
inputs, and list operations return new arrays. Records freeze their public
state. The generated runtime is an implementation detail shared by emitted
functions and portable tests; it interprets only the already-checked v0 IR.

The generated package contains deterministic `package.json`, `tsconfig.json`,
runtime/API sources, one native Node test per portable test, 20 boundary
conformance vectors, and a negative type fixture. Bazel's native gate runs
Prettier, strict `tsc`, and `npm test` using the versions pinned in the dev
container.
