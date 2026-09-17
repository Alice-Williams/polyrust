# C17 binary64 truncation mapping

## Typed lowering

Use existing CExpressions::known(CKnownCall::FloatTruncate) and checked call
construction with exactly one F64 value and F64 result. Materialize the original
source receiver once; retain C full-expression sequencing. FloatTruncate is the
only new standard callable admitted by this closed profile. Its argument is
walked for generated-call authority, ownership, numeric and resource checks.

The existing catalogue derives math.h and CSystemLibrary::Math. A certificate
exposes the union of actual local library requirements and requirements retained
by original imported package authorities. Identity-only packages need neither.
A transitive consumer needs the library even when its source has no trunc call.
No caller-supplied string, signature pure flag or handwritten header list is
evidence. This is absence of generated-storage effects, not a claim of no fenv
effects. Standard-library internal frames are not zero cost. The catalogue owns
a nonzero 64 KiB supported-profile stack reserve for FloatTruncate. Each generated
caller includes the maximum reserve among its known calls, then composes ordinary
callee/imported-owner paths with checked arithmetic. Repeated sequential library
calls do not multiply that reserve. This may conservatively charge mutually
exclusive library and generated call paths together; it never omits their cost.
No other known call receives a reserve or permission from this evidence.

The allowance follows actual guarded-stack/watermark measurements, not a guessed
external-call exemption. The complete three-package probe used 6,264..9,152 bytes
(including worker/TLS/startup and generated frames) under GCC14/Zig O0/O2 and GCC
ASan/UBSan. Every configuration completed on a guarded 64 KiB thread stack, also
measured on 256 KiB stacks. Both guard faults and a one-byte allowance failed as
expected. The 64 KiB reserve retains substantial measured headroom and the existing
1 MiB complete-path policy. This is the existing empirical pinned-Linux-profile
model, not a theorem for arbitrary libm/toolchain/runtime replacements; rerun the
probes before extending that supported environment.

## Package boundary

The source integration checkpoint serializes and authenticates the library
closure in the package manifest and has native consumers derive link options
from it. Target foundation exposes typed evidence but does not enable source
truncation until that publication path is ready. The structural formatter may
print a catalogue callable name in existing call syntax. It must not choose the
mapping, add dependencies or implement rounding semantics.

## Required proof

Exact non-NaN result bits, signed zeros, infinities and NaN categories under
GCC14/Zig C17 at O0/O2; original/imported owners; call-count faults; malformed
arguments and unrelated known-call rejection; nested generated identity edges;
dynamic header presence/absence and transitive system-library sets. Native
links use these typed requirements, not an unconditional test-only -lm fix.

Reference: [C committee draft N1570](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf),
sections 7.12.9.8 and F.10.6.8. The admitted binary64 environment supplies the
IEC 60559 behavior, with exception-flag observation excluded.
