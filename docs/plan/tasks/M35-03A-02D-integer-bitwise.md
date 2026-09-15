# M35-03A-02D — Built-in signed integer bitwise operations

- Status: complete
- Parent: [M35-03A-02](M35-03A-02-scalar-parity.md)
- Depends on: M35-03A-02C
- Specification: [integer bitwise mapping](../../specification/typed-generation/rust-integer-bitwise.md)

## Contract

Add built-in i32/i64 complement, and, or and xor to both Rust-source backends.
Use a private compiler-checked input, a closed operation enum, executable builder
slots and ordinary typed target operators. Preserve once-only left-to-right
operand evaluation. Do not enable shifts, casts, arithmetic, Boolean eager
operators, operator overloads or new integer widths incidentally.

## Definition of done and tests

- Rust, separately compiled C17 and Java 21 libraries/consumers agree with an
  independent integer oracle on extrema, sign bits, alternating bit patterns,
  each bit position, nesting, records/shared reads and dependency calls.
- Instrument actual generated copies to prove operand order/count. Deliberate
  dropped-complement and reversed-operand mutations must fail the oracle.
- Compile-negative contracts cover missing/duplicate/wrong-capability/context/
  output/input registrations and private input construction for each backend.
- Target AST admission rejects wrong widths/types/operators; native C tests
  run GCC and Zig at O0/O2, Java uses all compiler warnings as errors.
- Read-only probes check 28 actual mapped AST nodes per target, including
  Java precedence and C integer promotion, and emit identical production bytes.
- Unsupported source rejects atomically; no custom runtime file or dependency.
- Rust/Bazel lint and complete isolated Bazel gates pass, followed by independent
  review, inspectable uncommitted examples and a dedicated commit/push.

## Progress

Specification and implementation started. No support completion or runtime
retirement is claimed until the above evidence is recorded.

The first isolated build caught a redundant i64 conversion in the Rust oracle
and the builder crossing Clippy's type-complexity threshold. The conversion was
removed. Two narrow documented expectations per builder retain the independent
type-level capability slots; no compiler error, runtime warning, test or semantic
check is disabled. The broader cleanup of registration syntax is not bundled
with this operation's behavior changes.

The initial full gate found stale negative expectations (integer complement is
now deliberately supported), old missing-slot fixtures without the new mapping
import, two integration-lint omissions and redundant consumer casts. These
were corrected without dropping the relevant source/graph/storage/API checks.
Fresh review also requested direct evidence of AST metadata: 28-node read-only
probes now check both target mappings and byte-identical production output.

The corrected implementation snapshot
`549ae201543a20ebe080e8e6fd3cb178189b9953` passes the new native,
AST, source-rejection, compile-contract and Clippy checks, plus both backend unit
suites. Its targeted gate was 26/27: the remaining failure was the C graph test's
missing zero-import expectation for its newly accepted complement fixture.
That expectation is now supplied and passes the complete release gate below.

## Completion evidence

- Exact implementation tree `8bd4b5e2006b70b24c7b498c52fdc2af8ffefc4a`
  passed all 570 tests across 735 targets in 524.467 seconds, invocation
  `15770881-3322-42e0-bcb7-cf9aeaf3c681`. The isolated Linux archive matched
  2,449 Git blobs/modes. Rust/Bazel lint and all existing release checks passed.
- Native two-crate tests compare 176,384 exact results (22,048 operand pairs)
  against Rust and an independent integer oracle. Separately compiled GCC and
  Zig O0/O2 and Java 21 warning-clean consumers agree. Dropped-complement and
  value-preserving operand-order mutations are detected by values or call traces.
- Fourteen compile-negative registration/input contracts, 40 atomic source
  rejection checks, 104 C profile cases and 180 Java dependency-body cases pass.
  Read-only probes verify 28 actual mapped AST nodes per target and byte-identical
  production output; transitive scalar-call and full Java API proofs also pass.
- The initial Sol Extra High reviewer accepted all evaluated repairs. A fresh
  blind Sol Extra High review of the exact implementation tree independently
  found no remaining core errors. Neither review required unrelated new features.
- Tested examples are exported outside the container, uncommitted and ignored,
  in `generated/m35-bitwise-549/c` and `generated/m35-bitwise-549/java`.
  That targeted snapshot has identical implementation to the final tree; only
  the C graph test expectation and documentation changed afterwards.
- Completion-document changes receive a final isolated gate before commit/push.
  Full scalar/catalogue parity and legacy runtime retirement remain incomplete.
