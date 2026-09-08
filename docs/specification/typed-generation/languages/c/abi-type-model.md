# C17 concrete ABI type model

- Status: normative Linux x86_64 model; owner M34A-11-01S/02B/03/04
- Evidence: independent test/abi_model_probe.c under both pinned compilers
- Compiler-capacity budgets remain separate measured Stage 04 work

## Exact scalar and pointer model

The supported model is LP64, little-endian, with signed plain char and no
short-enum ABI. All generated native builds use -fsigned-char and
-fno-short-enums in addition to the platform proof flags. Plain char remains
a different C type from signed char despite equal layout and range.

| AST scalar | Actual C type / typedef compatibility | Size / alignment (bytes) | Integer rank / promotion |
| --- | --- | --- | --- |
| Bool | _Bool | 1 / 1 | below char; promotes to int |
| PlainChar | char, signed range -128..127 | 1 / 1 | char; promotes to int |
| I8 / U8 | int8_t = signed char / uint8_t = unsigned char | 1 / 1 | char; promotes to int |
| I16 / U16 | int16_t = short / uint16_t = unsigned short | 2 / 2 | short; promotes to int |
| Int / I32 | int / int32_t = int | 4 / 4 | int; unchanged |
| U32 | uint32_t = unsigned int | 4 / 4 | int; unchanged |
| I64 | int64_t = long | 8 / 8 | long; unchanged |
| U64 / Size | uint64_t = unsigned long / size_t = unsigned long | 8 / 8 | long; unchanged |
| F64 | double, IEC 60559 binary64 | 8 / 8 | floating conversion rules |
| Object/void pointer | exact qualified target retained | 8 / 8 | no numeric promotion |
| Function pointer | exact prototype and separate contract/provenance | 8 / 8 | no object-pointer conversion |
| Known max_align_t | standard allocator alignment carrier | 32 / 16 | not arithmetic |

Integer rank is Bool < char < short < int < long. Corresponding signed and
unsigned types share rank. Int/I32 and U64/Size are compatible C types;
structural declaration/origin identity is nevertheless retained separately.
Typedef expansion cannot erase reference authentication. Bool values have
domain 0..1. Fixed-width integer ranges are exact two's-complement/power-of-two
ranges for the listed widths.

Integer promotions precede arithmetic. Equal signedness selects the greater
rank. For mixed signed/unsigned: an unsigned type with rank at least the signed
rank wins; otherwise the signed type wins if it represents the entire unsigned
range; otherwise use the unsigned counterpart of the signed type. Thus
U32+I32 is U32, U32+I64 is I64, U64+I64 is U64. Shifts promote each operand
separately and return the promoted left type. Comparisons/logical operators
return actual int, followed by explicit Bool conversion where required.

A double operand converts the other arithmetic operand to double; the verifier
still enforces conversion/range and portable-operation obligations. Pointer
ordering/arithmetic is not inferred from equal layout. Pointee qualifiers,
nominal identity and function contracts remain independent constraints.

## Native enum and aggregate layout

Native enum definitions have only int-representable constants. On the pinned
model, an enum with any negative enumerator is compatible with int; otherwise
it is compatible with unsigned int. Both occupy 4 bytes with alignment 4,
and retain that compatible promoted type. Named enumerator expressions are
always int. Probe both categories, including negative and INT_MAX boundaries.
Portable payload-free enum ABI remains an independent validated uint32_t
typedef; native enum storage never substitutes for that portable representation.

Struct members retain source order. Each member offset is the checked round-up
of the prior end to its required alignment; aggregate alignment is the maximum
member alignment; total size is rounded to that alignment. Union size is the
maximum member size rounded to maximum member alignment. Array alignment is
its element alignment and size is checked element-size times nonzero bound.
No packing, custom alignment, bitfield or flexible-array directive is admitted.
A concrete probe verifies member offsets 0/8/16 and size/alignment 24/8 for
U8,U64,U16, and size/alignment 8/8 for a U8/U64 union.

Incomplete tags have no object layout yet. By-value completeness/cycles are
checked before size accounting. Known opaque FILE is only borrowed through
catalogued pointer signatures, never assumed to have one of these layouts.

## Independent evidence and stage obligations

Bazel targets c_abi_model_probe_test and c_abi_model_unoptimized_probe_test
use the hermetic Zig compiler at O2/O0. c_gcc_abi_model_probe_test checks exact
GCC 14.2.0, compiles/runs at O0/O2, and deliberately overrides plain char to
unsigned: compilation must fail at the poly_char_signed assertion. These
repository-only probes may use C _Generic/macros as an independent type oracle;
that does not add those constructs to the production target AST.

Invocation f398dd9f-042c-436b-927e-482f82105523 passes all three probes and the
focused Rust/Clippy/Rustfmt/Buildifier gates. Runtime probes check endianness,
double 1.0 bit layout and the INT64_MIN bit representation. This establishes
the selected ABI facts, not a production C certificate or a complete resource
budget. Stage 01S encodes the scalar compatibility/promotion model and expands
the two-compiler oracle to all 169 arithmetic pairs; Stage 02B consumes those
rules in typed operator/expression construction. Stage 04
mechanically compares its layout calculations with the native probes and
emits typed platform assertions. Every model change needs both compiler probes
and rejected configuration controls before certification.
