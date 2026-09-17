# M35-03A-02O-02 — C binary64 remainder foundation

- Status: planned
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
