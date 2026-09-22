# M35-03A-02S-02 — C wrapping-multiplication foundation

- Status: complete
- Parent: [02S](M35-03A-02S-wrapping-multiplication.md)
- Depends on: [oracle](M35-03A-02S-01-multiplication-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-wrapping-multiplication.md)

## Contract

Admit exact same-width internal U32/U64 Multiply through the existing shared
profile. Preserve signed public signatures and certified guarded reconstruction.
Unsigned modular overflow must retain MayWrap/loss provenance, never allocation
extent or nonwrapping-size evidence. Existing numeric checks remain authoritative.

## Definition of done and tests

Positive both-width AST/native examples pass; mismatched widths, unsupported
operators, signed-overflow forms and malformed reconstruction reject. Both child
subtrees are visited. Strict separate GCC14/Zig O0/O2 compilation, standalone
headers and UBSan agree with the oracle. Safe compiling addition, saturation,
narrowing and disconnected-operand faults are detected at each width. Direct
overflow-transfer tests protect MayWrap provenance. Preserve addition/subtraction
bytes, full gate and independent review; no compiler-source admission yet.

## Implementation and focused evidence

The production change admits Multiply alongside Add/Subtract only for identical
internal U32/U64 operands/results, visiting both original children. Numeric-flow,
conversion, pinned ABI and resource/ownership proofs are unchanged. Test-only
shared integer fixtures gain a Multiply enum case. Earlier unadmitted nested
multiplication controls now use still-unsupported unsigned division; no gate is
disabled and earlier generated operation shapes remain unchanged.

Tree 04dd4c39651b29e71d10c0af7385bbd0eeefb830 passes all six new focused
tests, f14b457e-ab1a-4093-ae80-1bdbd3b75821. Both widths prove guarded signed
reconstruction and reject malformed guards, conversions and signed arithmetic.
Exact profile traversal checks both children and rejects mixed types. Recursive
unsupported forms, depth, original-owner identity, derived imports, source-byte
and stack bounds remain enforced. Direct transfer tests retain MayWrap even
when UMAX*UMAX wraps to the small exact value one; nonwrapping products retain
their distinct evidence.

Native proof uses 34,546 independent pairs and both original target owners
(69,092 observations per run), separately compiling strict GCC14/Zig O0/O2
producers/clients, standalone headers and GCC UBSan. Two correct packages and
five safe compiling fault packages (wrong operand, wrong reconstruction result,
addition, narrowed result and saturation) match their independently modelled
results; each fault disagrees with truth at both widths. Saturation and narrowing
are disposable test-only mutations, never production runtime/helper code.
Full-gate and independent-review receipts are required before completion.

The first full gate caught missing policy registration for the new disposable
Python mutation fixture: its Python imports were treated as generated-template
directives. The repair adds only that exact test path to the existing fixture
allowlist and includes it in the deliberate adjacent-copy/production-path
rejection tests. No production path or wildcard exception is introduced.

Independent whole-scope Sol Extra High review, including that exact-file policy
repair, found no core defects or required proof gaps. It traced unsigned U64
products through u128 range evaluation, exact modulo/MayWrap transfer, local and
conversion loss provenance, and nonwrapping allocation/byte-count/index consumers.
It also checked the saturation mutation at MIN*-1 and all native compile flags.
Optional shape-only swapped-factor and literal target-fixture snapshot controls
are deferred: factor values commute in this pure target fixture, original source
evaluation is a later checkpoint, and existing exact-shape/native gates remain
active. The production renderer and old ordinary add/sub fixture branches are
unchanged; prior source-package byte hashes are checked separately.

## Completion evidence

The first full gate passed 947 targets with only the recorded fixture-policy
failure (34a10622-62b6-4ee8-add7-0fd091f1c05b). Repaired reviewed tree
e97c78a81de3c5af57f59932b86c21057ce7716c passes all 948 release/lint targets,
1a7882b2-1ff1-4091-bab1-863c1ee0bafb (5 executed, 943 cached). All 829 C
unit tests pass; the five capacity cases pass separately. Policy injection checks
remain enabled. The isolated checkout matches the exact index; all 389 previous
generated files and all 38 unrelated WIP hashes match.

No Java, compiler-source admission, legacy/runtime path or dependency changes.
Documentation-only closure is gated once more before this checkpoint's separate
commit/push. Java target proof and checked source integration follow separately.
