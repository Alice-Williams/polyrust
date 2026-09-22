# M35-03A-02T-02 — C signed-widening foundation

- Status: complete
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [oracle](M35-03A-02T-01-widening-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-signed-widening.md)

## Contract

Admit only the exact internal I32-to-I64 numeric-conversion shape needed for
lossless source widening. Preserve recursive operand certification, original
dependency authority, range/loss tracking and pinned platform proof.

## Definition of done and tests

Certify separately compiled producer, forwarder and external client. GCC14/Zig
O0/O2 and GCC UBSan agree with the independent oracle; headers compile alone.
Safe compiling zero-extension, premature-narrowing and disconnected-result
controls disagree. Reject incorrect source/result widths and unsigned/floating
substitutions; preserve existing guarded conversions and ordinary negative tests.
Exercise nested unsupported operands, original-call authority and resource bounds.
No new source admission, renderer special case or runtime. Full gate, independent
review and previous-package/WIP preservation precede a separate commit/push.

## Implementation and focused evidence

The only production admission change adds Numeric(I64) over an exact I32
operand to the existing integer profile. It schedules the original operand for
recursive checking; C numeric safety, dependency authority, renderer and compiler
source admission are unchanged. Test-only packages provide three independently
certified owners: an original I32 identity dependency, a widening producer and an
I64 forwarder. The producer materializes the original dependency result once.

Six focused cases pass on tree 0e02af66dd5c7d0ac5213554e747a9f46582162d.
The first short-name filter selected zero cases because this test binary uses
exact names; that result was not accepted as evidence. The corrected run selected
all six and caught fixture registration omissions. Forwarders no longer register
an undeclared unused local, and both independent numeric-check bodies contain the
registered array declaration. All six then passed, including native execution.

Evidence covers a 49-pair conversion matrix, recursive unsupported/depth rejection,
foreign registration rejection, derived imports, source-byte and increasing
cross-owner stack bounds. Numeric-flow tests preserve the entire signed I32 range
and retain the original narrowing-loss origin through widening. Even a resulting
0..255 range cannot certify nonwrapping indices or allocation after a lossy cast;
the nonlossy constant control succeeds.

Native tests separately compile all three actual generated owners and an external
consumer, plus standalone headers. GCC14 and Zig at O0/O2 and GCC UBSan agree with
73,890 independent inputs, observing both widening and forwarding results per
run. Safe compiling zero-extension, premature-i16 narrowing and disconnected-zero
mutations match their independent fault models and disagree with truth. Original
generated sources remain unchanged; all mutations are disposable test fixtures.
Only the exact harness path is exempted from source-template policy, with adjacent
copy and production-path rejection controls. No wildcard exception is introduced.

The first full release gate caught a collapsible-if Clippy error in the new
range-test filter. The correction uses an if-let chain without changing assertions;
completion still requires a full corrected-tree gate and independent review.

Independent whole-scope Sol Extra High review of tree
1dc28ffc8707a1a3bbc62bad2d0ed53aa2510679 is clean: no substantiated core defect,
required proof gap or optional feature request. The review traced exact profile
admission and recursive reconstruction, numeric ranges/loss origins, dependency
authority, call-path bounds and source-byte accounting. It checked defined C
fault mutations and separate native compilation, exact policy exemptions and
unchanged renderer/source admission. It accepted the fixture registration and
style-only corrections; a passing full corrected-tree gate remains required.

## Completion evidence

The initial full gate, f8e007f0-91a2-416b-b36b-6bf6bbab2110, passed 968 targets
with only the recorded Clippy build failure. Corrected tree
6633ca9fa2870bea4ac814aa996eb5bedd769ed8 passes all 969 Linux release/lint targets,
b59824c4-a4af-4657-8b43-28aeae344260 (4 executed, 965 cached). The full C unit
suite passes 835 cases; five capacity cases pass separately. No tests were
disabled and no timeout or cache policy was relaxed.

All 406 existing generated file hashes and all 38 unrelated WIP hashes match.
The isolated candidate matches its exact scoped index. No Java implementation,
compiler-source admission, legacy/runtime deletion or dependency change belongs
to this checkpoint. Documentation-only closure receives a final full cached gate
before the separate commit/push; Java widening follows as 02T-03.
