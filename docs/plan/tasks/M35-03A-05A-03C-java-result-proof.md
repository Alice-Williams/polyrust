# M35-03A-05A-03C — Java result execution and boundary proof

- Status: complete
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

## Implementation sequence

1. Reuse the 03B original-family/import/signature fixtures. Keep family owner,
   executable producer and branching relay as distinct compilation units. The
   producer exports a bool/int-to-Result selector and scalar success/error arm
   functions; the relay imports their opaque callable/type/member handles.
2. Generate the relay with one final selected Result local, a typed success
   pattern and the corresponding payload accessor. Each branch calls exactly
   its selected scalar helper. Do not add production instrumentation, new AST
   syntax, runtime files or a broader source-publication profile.
3. Resolve observer anchors from each original certified function path and
   signature. Require exactly one matching definition before inserting a
   test-only event hook. Compile and run the unmodified output separately as
   the control; derive neither observed events nor expected results from the
   lowering implementation.
4. Test both variants for all integers -256 through 255 plus signed extrema,
   -1, 0, 1 and 17: 1,036 cases per native configuration. Report complete case
   counts and independently count value and trace failures. Successful zero
   remains distinct from error. Normal and interpreted JVM modes must agree.
5. Build compiling typed-AST controls for an eager inactive arm, duplicated
   selector and arm-before-selector ordering. Their returned values must still
   pass while their measured traces fail for every exercised variant. Separate
   tag/payload controls fail value observations, not merely compilation or
   unrelated exceptions. Assert each mutation's actual detection count.
6. Retain 03B external-consumer negatives, source reservations and measured
   classfile bounds. Add the expensive trace matrix to native_tests.bzl so the
   existing partition contract proves exactly-once inclusion in the public
   suite. Export the actual uninstrumented generated examples under ignored
   generated output, then perform the complete release/review/preservation gate.

## Completion evidence and review

Code tree `93fc7b22d309c784d407d011baafb71db3fa36b9` passes all 1,042 Linux
Bazel release/lint targets (15 executed) in 904.824 seconds; invocation
`3c8c50f2-1049-4320-812a-139572a6bd86`. The Java ordinary unit suite passes 483
cases with zero failures/ignored cases and five native cases selected separately.
The new native target passes in 37.2 seconds and remains in the public suite and
the exactly-once partition contract. Rust/Clippy, Bazel formatting and source
policies pass without exceptions or disabled checks.

The measured native matrix executes 24 runs of 1,036 observations each, or 24,864
observations. Four unmutated configurations pass both value and trace oracles.
Each of three compiling, value-preserving faults passes all values but fails all
1,036 traces in each instrumented JVM mode. The tag fault fails all 1,036 value
observations; the payload fault fails precisely the 518 success observations.
Each fault also has separate pristine normal/interpreted value controls. Counts
are asserted before reporting, and the native test completes every input case.

Actual compiled classfiles and source bytes fit certified reservations. Tests
exercise Result construction at 100,000 body visits and one over, nested body
depth 128/129, declaration counts 100,000/100,001, JVM parameter slots 255/256,
imported call height 128/129 and exact transitive owner counts. Existing exact
family-selection/search and registration limits remain tested. The 03B strict
external-consumer negatives, null policy and copy/return proof remain in the gate.

Design review gaps for full certified observer anchors and Result-specific bounds
were accepted and fixed. Independent GPT-6-SOL extra-high review then identified
the missing production declaration-cap test; the 100,000/100,001 test closes it.
Final broad/follow-up review has no remaining core findings. An initial gate
flagged static imports in the handwritten native driver; fully qualified test
references fixed it without weakening the import policy or production renderer.

All 530 prior compiler-bundle files, four character-constant example bundles and
45 unrelated WIP files are unchanged. The three real uninstrumented Generated.java
examples and README are exported under ignored generated/m35-java-result-examples
and byte-match the native test's java-result-examples artifact. No observer or
runtime enters those generated packages. Final documentation is re-gated before
the scoped commit/push; hosted CI status is tracked separately.

This completes Java target transport and closes parent 03, not compiler Result
source admission (05A-04), general enum parity or the broader migration.
