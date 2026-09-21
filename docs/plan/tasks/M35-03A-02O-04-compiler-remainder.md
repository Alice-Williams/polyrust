# M35-03A-02O-04 — Checked Rust binary64 remainder

- Status: complete
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02O-03

## Contract

Add a private canonical RemainderInput and an executable FloatingRemainder
capability slot. Require original HIR/TypeckResults, built-in Rem identity,
exact unadjusted f64 operands/result. Materialize left then right once.
C constructs the typed catalogue call; Java constructs the structural operator.

## Definition of done and tests

Three actual Rust crates generate ordinary C/Java packages. Native results and
traces agree with Rust and the independent oracle; exact manifests/visibility,
original imports and certificate-derived C library closure hold. AST probes
reject copied input/wrong context and detect actual operator/call/dataflow
mutations; probe bytes equal production. Missing/duplicate/wrong binding and
private-witness compile failures are exact. Integer %, f32, overloads, casts,
mutable assignment and rem_euclid reject atomically. Export real examples.
Full gate, old hashes, preserved WIP and two clean reviews precede completion.

## Implementation and focused evidence

The source contract is a separate private RemainderInput/FloatingRemainder
capability, not an expansion of FloatingArithmetic. Its reader authenticates
the original rustc HIR node and typechecking owner, built-in Rem identity,
unadjusted f64 result and both ordered children. Both executable mappings
finish and materialize the left operand before beginning the right.

- Fourteen compile-negative tests require the intended error for missing,
  duplicate, wrong-capability/context/output/input and private-witness cases.
  The target-independent source contract and the twenty historical omission
  fixtures still pass. The initial focused gate passed all 18 targets.
- Three actual source crates retain five function identities, an original
  public module, a private helper, original imports and documentation.
  Native proof checks 22,832 target observations per run against Rust and the
  independent integer/rational oracle, with separate owner/client compilation
  under strict Java21 and GCC14/Zig O0/O2.
- Six compiling faults detect nearest-quotient substitution, operand reversal,
  zero-sign loss and dropped/duplicated/reordered original producer calls.
  The last three preserve every value but fail the independent trace oracle.
  C links from certified library inventory; missing -lm fails to link.
- Thirteen AST observations per language check canonical source/context,
  copied nodes and swapped stored children, actual typed operation/dataflow,
  and original per-operand calls. Actual operator/call/argument substitutions
  and right-result disconnection are detected; probe/production bytes match.
  Native composition covers negation, abs, is_nan and trunc, including -0.0.
- Forty valid unsupported-source cases reject atomically, preserving existing
  sentinels. Earlier remainder-negative fixtures now use unsupported Euclidean
  remainder; their former supported compositions have positive coverage.

The first native harness used Java's dependency-list field for C as well.
C instead records imported original owners. The repaired test compares
imports[].owner to Java's dependency set, retaining exact identity checks.
No production mapping was changed for this fixture repair.

Repaired tree da37f0906380709dea11dc72b3c41a674ea3acb3 passed the native
gate, invocation c19b69df-c04e-43eb-839c-5417e16b1627. Actual unmodified
packages, original Rust and baseline clients are copied byte-for-byte to
generated/examples/floating-remainder-da37f090/README.md (ignored, not committed).
The full release/lint gate on that exact tree passed all 900 targets:
bb906eae-c78d-409e-b54c-037144194eb6 (72 executed, 828 cached).
All 331 files across 46 older generated bundles remain byte-identical,
and all 38 recorded ownership/adjacent work-in-progress hashes match.
The disposable checkout matches the candidate Git index, including pinned
lock metadata (--lockfile_mode=off). Independent-review closure follows.

## Independent review repairs

Both reviewers found proof gaps rather than a production mapping defect.
All were accepted: native Rust operand traces were not measured, set-based
imports and keyed declaration dictionaries could hide duplicates, documentation
checks were incomplete, and privacy was not exercised from outside the package.

The repaired harness builds a test-only instrumented copy of the original Rust
leaf through pinned Bazel Rust rules. Only two eprint calls are inserted; the
original middle/root/reference sources are unchanged. Native Rust values and
actual operand traces must agree with the independent oracle and generated
C/Java. Exact import multiplicities, raw declaration counts/kinds and complete
normalized module/crate/function documentation have deliberate mutation controls.
External Java access must fail compilation; explicit-extern C access must fail
linking under both compilers and optimizations. Exposing the private helper in
disposable copies must make those same consumers succeed.

Repaired source tree ef1bf8caaf3a300ce9b3f246199fa52b46514288 passed all
900 release/lint targets, invocation 57b44f51-7a6c-4ddd-9668-6485ce67723a
(2 executed, 898 cached). The focused repaired native/buildifier run was
506c490e-0008-44c6-8f73-1b4246c21db9. Both original independent reviewers
rechecked and cleared the repairs. The native proof measures 11,416 Rust cases
and 22,832 generated target observations (middle plus root) per run.
All 331 older generated files and 38 preserved WIP hashes still match.

Final source tree 0765c30bf1b156126be1d9692e2ecd3bff69a345 clarifies that
observation count in the driver and exported README. It passes all 900 targets,
809b2399-10fb-4838-854f-1bcdc5a93239 (2 executed, 898 cached). Final actual
examples are copied and compared byte-for-byte at
generated/examples/floating-remainder-0765c30b/README.md. They are ignored
local artifacts, not checked-in generated source.
