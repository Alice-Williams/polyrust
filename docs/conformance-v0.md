# Differential conformance v0

Conformance case IDs are stable strings paired with one Core capability and a
target-neutral portable value. Expected data uses
`polyrust.canonical.v0`; cases never contain target source text. The checked
registration fixture supplies the portable contract-dispatch test, while the
versioned corpus supplies 50 named boundary and nested-value vectors.

The in-process harness runs the reference evaluator, verifies canonical
round-trips, generates every required backend twice, compares complete manifest
bytes, and verifies native-test manifest coverage. Bazel's `all_targets_test`
then executes the generated Rust, TypeScript, Python, and Go packages with their
native compilers and test frameworks.

Mismatch records always include the case, invoked function/helper, input,
oracle, target, and first structural difference. Staged arithmetic, Unicode,
and enum-tag mutations are required to produce mismatches. Local and release
execution use the pinned development image; required toolchains are never
silently skipped.
