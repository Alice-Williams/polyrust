# M35-03A-02F-02B-04B — Compiler public constant declarations and reads

- Status: complete
- Parent: [compiler public constants](M35-03A-02F-02B-04-source-constant-mappings.md)
- Depends on: M35-03A-02F-02B-04A

## Contract

Implement the shared public-constant source contract through executable
PublicConstants and PublicConstantReads bindings for C and Java. Resolve a closed
function/constant/module export inventory from compiler identities. Package
contexts register exact evaluated scalar constants before lowering function
bodies; body contexts read only the corresponding typed owned value references.

Preserve complete aliases, docs and effective visibility. Constants-only source
packages use the package-state path without a function Reader. Preserve existing
selected-entry semantics with an explicit documented selection policy; do not
silently substitute folded literals for authenticated public-package API reads.
Foreign producer joins and versioned multi-crate metadata remain child05 and must
fail closed until supported.

## Definition of done and tests

- Bool/i32/i64 declarations and reads work in real single-crate Rust/C/Java,
  including constants-only and mixed packages, computed values and forward reads.
- Compiler mapping probes verify actual typed declarations/references, exact
  identities, values, types, readonlyness, source ownership and emitted bytes.
- Missing, duplicate, wrong capability/context/input/output and forged compiler
  input controls exist for each new per-backend binding.
- Independent native consumers cover both bools, signed boundaries, wide values,
  aliases, private same-spelled constants and docs; native writes fail.
- Unsupported types, borrowed storage, generic owners and incomplete exports
  reject before publication. Compiling semantic mutants fail independent truth.
- Old public-constant rejection tests are replaced only alongside positive proof.
  All private/local constant regressions, full release/lint gates and independent
  review pass; ignored examples are exported and inspected before scoped push.

## Implementation sequence

1. [04B-01 — Shared typed export inventory](M35-03A-02F-02B-04B-01-export-inventory.md) — complete.
2. Register executable public constant declarations and reads for both targets,
   integrate owned single-crate metadata and constants-only publication, and add
   native/compiler/mutation proof required above.

The first checkpoint unifies source selection without enabling public constant
output. The declaration/read and publication evidence below completes this parent;
inventory classification alone was not capability support.

## Implementation and proof order

1. Add private compiler-authenticated declaration/read inputs. Register the two
   executable mappings in both consuming builders; register declarations on
   package state before constructing any function Reader.
2. Carry owned constant identities through package/Reader transitions. Public
   package reads resolve those references, never names or folded substitutes.
   Selected-entry generation remains an explicit value-only projection.
3. Assemble ordinary C const header/source objects and Java public static final
   fields. Extend owned C metadata with a lossless, explicit constant schema;
   leave multi-crate constant publication rejected until child05.
4. Add builder compile-negative controls, compiler/target mapping probes,
   constants-only/mixed native truth and native write rejection. Replace old
   public-constant rejection cases only with these positive proofs.
5. Run focused and unfiltered Linux Bazel gates, fresh independent review loops,
   export/inspect ignored examples, then commit and push the completed step.

## Verification evidence

- Both compiler lowerers register the same private declaration/read inputs through
  eighteen-slot consuming builders. Declaration mappings use package State;
  read mappings use the actual function Reader. Constants-only paths construct
  no Reader, no dummy callable and no custom runtime.
- The standalone capability probe now includes shared source-origin metadata and
  its portable_codegen data types to authenticate declaration inventory. It still
  has no C/Java backend dependency and source inputs contain no target AST nodes.
  The original no-codegen observation in M35-01E-01 describes that older checkpoint,
  not this expanded compiler-origin contract.
- Twenty-eight new compile-negative targets cover both bindings in both backends:
  missing/duplicate registration, wrong capability/context/output/input and
  private-input construction. Existing missing-capability controls now register
  the new slots so they still fail for their intended original omission.
- The actual C/Java probe adapters verify ten registered declarations, ten public
  mixed-package reads, exact compiler identity/value/type and emitted-byte
  equality. Missing, altered-value and substituted-identity read maps reject.
  C metadata probes reject seven expectation/retained-manifest corruptions.
- Independent Rust consumers and an independently written numeric oracle agree
  with separately compiled Java 21, GCC 14 and Zig consumers at O0/O2. Coverage
  includes both bools, signed limits, values outside double precision, computed
  values, textual forward declarations/reads, aliases, private same-name values,
  and docs. Native writes to all ten unique exported objects/fields reject.
  Compilable wrong-value mutants fail the original oracle; Java consumers are
  recompiled against each mutant to account for javac constant folding.
- A separate Rust/native selected-entry fixture proves that public constants
  still fold into literals without registering public objects/fields. Probe and
  production bytes agree and no public-constant mapping callback runs.
- The 80-execution atomic rejection matrix preserves absent/existing output for
  unsupported widths/storage/generic owners, invalid evaluation and foreign
  public module-constant reads. Old public bool/i32/i64 rejection cases were
  replaced only after positive native evidence passed.
- The first full run exposed four test targets with stale probe dependencies,
  exhaustive Reader snapshots or lifetime annotations. All were fixed, not
  disabled. The next full run passed 716/716 tests on tree
  b26e0975423d8a2b8210f0299aad84df65ddad47, invocation
  eb7bbbc7-030e-4cb6-bc5e-b2b33eceb8bb.
- Final code/spec tree 70a08dcb8f03169425232b9fd01ebe1e960ff318 passed
  the unfiltered Linux container command
  `bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test
  //... //:release_gate --noshow_progress --noverbose_failures
  --test_output=errors --test_summary=terse --keep_going`.
  Result: 1,040 targets; 716/716 tests; 12 executed, 704 cached; 80.233 seconds.
  Invocation: 4d34a9f1-4f18-48e7-829b-92c7322c555d. The isolated archive's
  2,609 Git blobs and executable modes were verified before testing.
- Actual generated constants-only/mixed/selected-entry C and Java artifacts and
  their Rust inputs were exported and inspected at ignored host directory
  generated/examples/public-constants-70a08dc. Directory publication occurred
  on the container's Linux filesystem before copying artifacts to the Windows
  bind mount, which does not support the publisher's atomic no-replace rename.
  No generated output is staged. Twenty preserved unrelated file hashes match.

## Review evaluation

The first independent Sol Extra High review identified missing selected-entry
and foreign-read integration proof and a falsely named forward-reference case.
All were accepted and fixed. The stale C specification status was also corrected.

A suggested duplicate type/readonly check in C metadata reconstruction was
withdrawn after tracing the certificate boundary: the C source profile already
requires const bool/i32/i64 storage with an exact literal before a
RenderReadyPackage can reach the manifest. Repeating that checker in JSON would
not repair a demonstrated defect. Existing profile/certificate negative tests
remain authoritative; new manifest tests cover its distinct identity/inventory
reconstruction responsibility.

A fresh independent Sol Extra High reviewer audited exact tree
70a08dcb8f03169425232b9fd01ebe1e960ff318 without relying on the first review and
reported no actionable correctness, regression or required-proof defects.
Completion bookkeeping is included in the final isolated gate before the scoped
commit/push. Child05, other constant value families and overall legacy runtime
retirement are not completed by this task.
