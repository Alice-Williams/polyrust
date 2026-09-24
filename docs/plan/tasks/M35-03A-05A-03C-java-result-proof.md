# M35-03A-05A-03C — Java result execution and boundary proof

- Status: planned
- Parent: [Java result transport](M35-03A-05A-03-java-results.md)
- Depends on: [certified imports](M35-03A-05A-03B-java-result-imports.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-scalar-results.md)

## Contract

Close the transport parent's proof obligations with generated, separately
compiled producer/relay/consumer packages. Values alone are insufficient:
measure actual execution of scrutinee and selected variant helper. Keep an
uninstrumented control and test-only observation hooks derived from exact
certified method definitions; no production observer/runtime scaffolding.

Test both variants over boundary and representative primitive payloads under
normal and interpreted JVM execution. Compiling value-preserving mutants must
expose eager inactive-arm execution, repeated scrutinees and reordered calls.
Tag/payload mutations must fail the value oracle. Mutation detection must not
depend on compilation errors, assertion configuration or unrelated exceptions.

## Definition of done and tests

- Report executed input/variant counts and fault rounds, with all cases covered
  rather than only the first failing branch. Compare exact original payloads,
  successful zero versus error, copies/returns and ordered call traces.
- External strict-javac consumers exercise original public nominal signatures.
  Negative consumers prove sealed subtype closure and private implementation
  boundaries. Null boundary behavior matches the documented policy, not Err.
- Node/declaration/JVM limits and dependency depth are tested at their actual
  boundaries; source size remains within certified bounds. No custom runtime
  file, runtime import or unchecked payload cast is emitted.
- Actual generated examples are exported as ignored build artifacts, not
  committed output. Preserve prior output bytes and unrelated WIP.
- A separately cached native target remains part of the release gate. Full Linux
  release/lint and fresh broad GPT-6-SOL review pass before commit/push and before
  marking the Java transport parent complete. Compiler integration stays closed.
