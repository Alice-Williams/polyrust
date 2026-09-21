# M35-03A-02O-02 — C binary64 remainder foundation

- Status: complete
- Parent: [02O](M35-03A-02O-floating-remainder.md)
- Depends on: 02O-01

## Contract

Admit only the typed CKnownCall::FloatRemainder catalogue signature, not C %
on doubles or a raw helper. Derive math.h and the Math system-library closure
from the certified call. Extend recursive admission, source/resource accounting
and original dependency evaluation without widening unrelated known calls.

## Definition of done and tests

Three typed target owners plus separately compiled clients agree with the
independent oracle under GCC14/Zig O0/O2 and strict warnings. Native controls
detect wrong remainder family, operand order/count and signed-zero loss.
Measure the actual supported fmod call/import chain with guarded stacks and
watermarks; missing library metadata, wrong signatures, unregistered callables,
mixed widths and resource budgets reject. Existing integer-UB and unrelated
known-call rejection remains. Full gate, unchanged old artifacts and review pass.

## Focused implementation evidence

Tree 995a78c4b672c747b69f6d56727e571ebaf87836 passed both native value/trace
and guarded-stack tests, invocation bd0a99d2-a458-4013-ac2f-918d8050e044
(63.759 seconds overall; 20.32 seconds in the two native tests).

Three independently certified target owners use two original identity producers,
a two-operand remainder function and a forwarding importer. Each GCC14/Zig
O0/O2 run checks 22,832 exact non-NaN bit/NaN-category results from 11,416 input
pairs. Six compiling faults cover nearest-quotient remainder, reversed operands,
zero-sign loss, and dropped/duplicated/reordered producer calls. The last three
preserve values and are detected by independently expected A/B traces. Omitting
the certified Math link dependency fails to link under GCC.

The actual fmod/import chain runs on guarded 64 KiB and 256 KiB pthread stacks,
under GCC/Zig O0/O2 plus GCC ASan/UBSan. Its watermark is measured afresh rather
than inherited from truncation. Generated function frame reports are compared
with exported certificate bounds; a one-byte allowance and both guard-page
faults reject. Actual AST call chains test resource admission and missing or
underestimated library cost.

## Completion evidence

Corrected source tree 977cf651f00ea2a31a340fb2de8763ebec084e32 passed
all 882 release/lint targets in the Linux development container, invocation
323b98b0-0321-49fc-9f3c-cd883c24c2f1 (846 executed, 36 cached).
The C unit/native partition passed 810 tests; capacity partitions ran separately.
Two fresh Sol Extra High reviewers found no remaining correctness/proof defect.
The first review caught an unused fixture parameter; explicitly discarding its
typed value fixed the fixture without weakening production admission.

An initial Windows archive used CRLF checkout conversion, breaking the generated
Zig launcher. The tested archive was re-exported with core.autocrlf=false;
no generator change was needed. Evicted cache inputs triggered Bazel's automatic
retry before the passing run. Bazel regenerated only MODULE.bazel.lock metadata
in the disposable checkout; final closure restores the committed metadata and
uses --lockfile_mode=off, leaving pinned dependencies and test caching intact.

This completes the C target foundation only. Rust-source remainder admission,
Java mapping and wider runtime-free parity remain separate checkpoints.
