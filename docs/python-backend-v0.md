# Python backend v0

The `org.polyrust.python` backend emits a dependency-free Python 3.13 package
from checked IR. Public functions are fully annotated and return
`PolyResult[T]`. Records are frozen, slotted dataclasses; contracts are typed
`Protocol` declarations; explicit implementations are emitted as concrete
methods on the implementing record. Enum payloads, `Option`, and value-level
`Result` use tagged frozen values rather than Python `None` conventions.

Both `i32` and `i64` use Python `int` storage, but every arithmetic operation
passes through exact fixed-width checked or wrapping helpers. Floats are rebuilt
from their canonical IEEE-754 bits. Strings reject surrogate code points when
Unicode-scalar semantics are requested. Bytes are immutable `bytes`, and lists
are tuples; append and concatenation always allocate a new tuple.

The emitted runtime interprets only checked v0 IR and is private implementation
machinery. Public generated APIs contain neither `Any` nor mutable-list types.
The native Bazel gate runs compileall, Ruff format/lint, strict mypy, pytest, an
expected-failure invalid-tag type fixture, one native test per portable test,
and 20 evaluator-aligned boundary vectors using the versions pinned in the dev
container.
