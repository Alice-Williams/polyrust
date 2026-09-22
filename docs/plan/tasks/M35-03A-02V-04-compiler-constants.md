# M35-03A-02V-04 — Checked signed-infinity source integration

- Status: complete
- Parent: [02V](M35-03A-02V-infinite-f64-constants.md)
- Depends on: [Java foundation](M35-03A-02V-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-infinite-f64-constants.md)

## Contract

Add Infinity(Binary64Sign) only to the distinct checked constant domain.
Retain canonical compiler definition/context, exact type/width and provenance
guards. Exhaustively map read/local/public/import/alias paths. Preserve existing
literal restrictions and reject every NaN constant atomically.

## Definition of done and tests

Original multi-crate Rust and generated C/Java agree on exact infinity bits,
with source-owned declarations, original API/docs/privacy and alias identity.
Typed probes reject wrong values/signs/types/owners and compile-negative mapping
contracts hold. NaN payloads, f32 and unsupported source forms leave output
unchanged. Existing floating compositions preserve their documented semantics.
Actual Bazel producer-sign mutation invalidates affected metadata/packages and
native tests, leaves independent producers cached, and restoration recovers
cached passing results. Export real packages, update partial inventory, pass
full release/lint, preservation and fresh review before separate commit/push.

## Implementation sequence

1. Extend ScalarConstantValue with Infinity(Binary64Sign). Classify only after
   the existing canonical definition, normalized f64 type and eight-byte
   ScalarInt checks. Keep FiniteBinary64 and literal admission unchanged.
   Every NaN sign/payload remains a diagnostic, including unused local constants.
2. Replace literal-only constant mapping helpers with exhaustive typed value
   and expression mappings. C constructs the known DoubleInfinity node and
   exact negative unary form; Java constructs the known positive/negative
   Double fields. Read/local/public/import/export paths must use the same
   exact value domain without manufacturing literal witnesses.
3. Authenticate imports by the full typed target constant value, original
   compiler declaration, type and producer certificate. Serialize finite and
   infinite F64 metadata as sixteen exact hexadecimal digits. Retain C8/Java6
   and C9 only when an actual system-library dependency requires it.
4. Add independent Rust producer, second producer, alias-only middle and mixed
   root crates. Cover both named infinities, compiler-evaluated overflow and
   signed division by zero, private/local/inherent reads and original aliases.
   Exercise both signs through ordinary admitted arithmetic/comparison paths.
   Preserve original public names, docs, private boundaries and crate ownership.
5. Compile original Rust references at O0/checks-on and O2/checks-off. Compare
   exact constant/read bits with the existing integer-only infinity oracle and
   separately compiled C (GCC/Zig O0/O2, UBSan) and Java21 (normal/-Xint)
   consumers. Recompile consumers for all compiling constant-value faults.
   Observe producer fields/objects and generated reader functions independently.
6. Add typed mapper probes and explicit wrong-sign/type/declaration/producer
   controls. Update prior finite-source rejection tests only for newly admitted
   infinities; replace those cases with still-unsupported NaN forms rather than
   deleting the atomic-publication coverage. Exercise f32, borrowed storage,
   type aliases and generic/trait constants without changing their rejection.
7. Prove actual Bazel invalidation with a producer infinity-sign change:
   affected metadata, packages and native tests rerun; an independent producer
   remains cached. Restore source exactly and verify restored cached passing
   tests and original output hashes. Record action evidence, not inferred
   dependency behavior.
8. Export the actual source-owned C/Java packages and original Rust fixture
   sources to a host-readable generated/examples checkpoint directory. Do not
   commit generated output. Update the partial parity inventory without marking
   broader scalar support or runtime removal complete. Complete preservation
   checks, full Linux Bazel release/lint gate and fresh whole-scope independent
   review before the separate milestone commit/push.

Keep source adapters, inventories, native consumers, rejection matrices and
cache proof in focused files/targets; reuse existing authenticated package and
oracle infrastructure without duplicating production mapping semantics in tests.

## Verification evidence

Native differential proof passes on tree 3493c8b3855d034ca079e39b0f263be41c25361c:
39 original reads at both full-crate Rust optimization/check profiles and
66 target observations per configuration. GCC14/Zig O0/O2, GCC UBSan and
Java21 normal/-Xint agree with the independent integer oracle. Three compiling
producer faults are detected after recompiling all dependent consumers.
Original declarations, aliases, docs, visibility, exact schemas and inferred
headers/system libraries are checked.

Integration exposed a missing C scalar-call effect case for the typed infinity
leaf. The fix does not admit arbitrary known constants: regression tests retain
rejection of missing function bodies and pointer-valued standard streams.
The shared Java graph probe also now recognizes the new fixture's explicit
private call; constant reads still do not acquire extra call frames.

Tree ffc8e27a68c8800f9aae5a6a55daed58b947a792 passes all 1,012 Linux
release/lint targets (138 executed, 874 cached), invocation
e4731382-b3e9-4a57-b163-022a9e3ad5a5. The main C unit suite passes all
847 tests; the separate finite-constant native corpus passes too.
The typed source gate checks 35 imported, three folded and two local mappings
per target, four public declarations/five reads and atomic authority failures.

Fresh broad Sol Extra High review of fac37ad672ba6eb56ac3e6e923e706c2b598466b
(the same implementation plus parity inventory closure preparation) is clean,
with no material findings or optional changes. All 462 previous generated
files and 38 unrelated WIP hashes are unchanged. Actual examples are exported
and byte-checked at generated/examples/infinite-constants-ffc8e27a, including
39 generated package files, six original Rust sources and a README.

The actual archive-isolated cache proof passed on reviewed tree fac37ad.
Its warm build had no target actions and its warm native test was cached.
Changing BITS_POSITIVE from positive to negative infinity reran exactly seven
affected metadata/package actions while the second producer stayed cached.
The old native truth failed; updated truth passed with separately recompiled
consumers. Restoring source recovered every baseline metadata/package hash
and a cached passing native result. Receipts are in the ignored directory
generated/m35-checkpoints/infinite-source-cache-fac37.

Documentation closure is checked again with the complete cached release/lint
gate before the separate commit/push. NaN constants and the wider scalar/runtime
migration remain incomplete; no legacy gate or runtime was removed.
