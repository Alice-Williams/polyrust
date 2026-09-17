# M35-03A-02M-01 — Certified C truncation standard-call foundation

- Status: complete
- Parent: [truncation](M35-03A-02M-floating-truncation.md)

## Contract

Admit exactly CKnownCall::FloatTruncate in the closed shared scalar profile.
Keep the existing checked callable signature, full-expression sequencing,
numeric/ownership/resource checks and original direct-call edges. Its argument
must still be walked; this standard call does not grant authority to a nested
unresolved generated call. Other known calls remain outside this profile.

Derive math.h from the actual typed call. Expose a read-only certificate-derived
system-library set that includes original imported-owner requirements, including
constant-only imports. Consumers do not invent imports or linking flags.
No Rust source admission or handwritten runtime is added here. The structural
call formatter prints the admitted catalogue identity using its existing call
syntax; it makes no semantic lowering or dependency decision.

## Resource proof correction

The initial implementation passed all 840 release tests, but final cross-spec
review found that treating libm frames as zero/external cost contradicts the
existing complete-path stack contract. This is a required proof gap, not an
optional feature. Do not mark this checkpoint complete on that gate. Establish
a catalogue-owned conservative stack allowance from controlled-stack/watermark
native evidence, compose it through direct/imported calls, and test missing-cost
and budget rejection before re-running the full gate. Do not weaken the prior
resource guarantee merely to admit a new standard call.

## Definition of done and tests

- Producer and separately certified importing consumers compile with GCC14 and
  Zig C17 at O0/O2 with strict warnings and floating-point flags.
- An independent integer-bit oracle covers positive/negative fractions, integral
  boundaries, subnormals, signed zero, infinity and NaN category.
- Wrong rounding direction, lost negative zero, dropped calls and duplicated
  calls are detected by separate value and trace observations.
- Primitive calling-contract negatives and unadmitted known calls reject.
- Nested generated argument calls retain exact identity and fail closed without
  a real definition. C sequencing rules are unchanged.
- math.h occurs only where referenced; transitive math link requirements survive
  a consumer that contains no local math call. Identity-only packages have none.
- Full Linux Bazel release/Rust/Bazel lint gates and independent review pass.
  Record exact tree, invocation and review decisions before marking complete.

## Resource correction evidence

The corrected implementation uses a private catalogue-owned NonZeroU64
64 KiB reserve only for FloatTruncate. Each function charges its maximum
sequential known-call reserve and composes it through direct/imported call
paths. The stricter no-direct-edge whole-unit fallback also adds the maximum
reserve. Missing costs reject; arithmetic remains checked.

- Guarded native probe tree: a00477c43acd6e6f889d6a10e69ee5d4b003a82c.
  Invocation 98d649f1-2312-4ad6-a1d9-c705cb4c0f6f passed; measurement rerun
  8e95f2a2-26ac-4519-a75f-8bfedf3a7971 passed.
- The actual rendered three-package chain runs on guarded 64 KiB and 256 KiB
  pthread stacks. GCC14 and Zig O0/O2 plus GCC ASan/UBSan O0/O2 measured
  6,264–9,152 bytes for the complete worker path. Generated frames measured
  8–48 bytes. Both guard-write controls and a one-byte allowance fail.
- Corrected implementation tree: 91e983bc6f4b7b1d0bb04dbcec1d7c4da704c936.
  Three exact resource/native tests passed in invocation
  13aae3e4-84c7-4554-8bbc-eb6e0660cf73 (59.222 seconds).
  Twelve actual generated frames are admitted at 1,020,160 bytes; thirteen
  reject at 1,106,944 bytes. Missing/one-byte-cost controls change admission
  and are detected. Imported certificates retain the charged bound.
- Two independent reviews of that exact implementation found no remaining
  core defect or required proof gap. The source-policy follow-up adds only an
  exact test-fixture exception, with adjacent-path rejection tests; the second
  reviewer also inspected that delta and found no issue.
- This is empirical support for the pinned tested environment, not a portable
  theorem about every libm. Compiler, standard-library or target changes must
  renew the controlled-stack proof before reusing this catalogue reserve.

## Completion

All 840 Linux Bazel release tests pass on tree
1e03205a99d5f035e905428ec17fcedcaad69117, invocation
a65b8b96-d72f-4843-afc1-0383e64866c3 (49.251 seconds; five executed,
835 cached). This includes Rust formatting/Clippy, Bazel formatting/lint,
source-policy and its failure-injection control, GCC/Zig native proofs and
the separate capacity targets. The C unit suite has 798 passing tests plus
five separately executed capacity cases.

The preceding implementation run (44823434-305f-43e7-a6cd-80d51ca2fbc3)
passed 839/840 tests; only the newly added handwritten probe needed its
exact-path policy classification. No tests were disabled or broad policy
exceptions added. All 138 file hashes across 18 earlier compiler-generated
bundles are unchanged. All 19 preserved ownership-work hashes are unchanged.

Both independent reviews were accepted. An additional constant-only native
link fixture and result-detachment mutation were considered optional: the
actual typed constant-import closure is already checked, transitive native
link options are consumed from the certificate, each wrapper result has an
independent bit oracle, and dropped/duplicated calls have separate traces.
The later zero-cost libm finding was a core issue and was fixed, not waived.

C target foundation only is complete. Java rounding-call admission and
checked Rust source integration remain the next separate checkpoints.
Publishing remains pending explicit resolution of the existing push-approval
block; passing local evidence is not a claim about unpublished CI.
