# C17 platform, resource and native proof contract

- Status: normative migration target; planned targets are not execution evidence
- Owners: M34A-11-03/04/06/07/09

## Supported platform and floating-point environment

The initial certified native platform is Linux x86_64, little-endian, with
8-bit bytes, exact 32/64-bit integers, 64-bit size_t and pointers, and IEC 60559
binary64 double (radix 2, mantissa 53, maximum exponent 1024, minimum exponent
-1021, sizeof(double)==8). int is 32 bits. Integer helpers assume and assert
the selected fixed-width two's-complement representation. Typed platform
assertions check these numeric properties; a native bit-pattern probe checks
endianness and binary64 layout. Allocated objects require no alignment beyond
max_align_t. Unsupported platform configurations are not silently admitted.

The compiler is the Zig SDK selected by MODULE.bazel's hermetic_cc_toolchain
4.3.0 dependency and lockfile. The independent sanitizer compiler is exactly
GCC 14.2.0. System C/Math libraries are platform dependencies, not bundled
third-party generator libraries.

Callers must retain round-to-nearest/ties-to-even, masked FP traps and gradual
underflow (no flush-to-zero/denormals-are-zero mode) during generated calls.
The native harness verifies this default environment and restores it after
adversarial environment probes. Generation does not mutate process FP state.
FLT_EVAL_METHOD must be zero. No fast-math, reassociation or implicit fused
multiply-add is allowed; compile with -fno-fast-math -ffp-contract=off.
Separate operation nodes are rounded separately. Probes include signed zero,
subnormals, infinities, NaNs, remainder, truncation and a multiplication/addition
counterexample where contraction changes the result. Portable NaN-class
expectation and exact raw-bit representation tests remain distinct.

## Typed native dependencies and metadata

CSystemLibrary is a closed enum, initially Math. Known catalogue entries such
as fmod/trunc carry the Math requirement; operations satisfied without a
library reference do not request it speculatively. Linking projects the
deduplicated requirement into manifest dependencies using a closed
native-system-library mapping. The native harness translates Math to -lm
after source/object inputs. It never executes caller-supplied link strings.

Non-executable metadata includes source roles, exact public symbol/signature
map, ABI version, platform properties and system libraries. It is deterministic
structured data plus human-readable README prose. No backend-generated BUILD,
shell, compiler-command template or package-manager hook is admitted. Bazel
rules/tools own execution and consume the closed manifest projection.
Negative dependency tests delete Math for a nonconstant fmod consumer and must
fail linking, then restore it and pass; a scalar-only package must not gain Math.

## Translation-phase and namespace policy

Reserve C17 keywords, all leading-underscore names, known standard identifiers,
runtime names and generated helper/file-guard identities. Macro reservations
apply before ordinary/tag/member/label namespace allocation, because macros
can replace a member or tag token too. The header catalogue includes every
macro exported by admitted headers that a generated name can collide with,
including NULL, bool/true/false, CHAR_BIT, integer limits, SIZE_MAX, floating
limits, EOF and any admitted allocator/platform macros. Unknown foreign
command-line macros are outside the certified build contract.

Allocate known/runtime names first, then portable symbols in canonical source
identity order. A generated name has a safe poly_ prefix, an injectively
escaped readable source spelling and a deterministic discriminator when
needed. Never truncate names into collisions. Target identifier-length
capacity is checked separately. Public ABI metadata records original module,
declaration/member identity, kind, visible name and complete generated
signature; consumer tests resolve names through this same map. Two symbols
cannot merge merely because their readable spellings sanitize identically.
A separate namespace prevents confusing a tag, typedef, local or label.

Executable literals use typed bytes/lengths and integer escape nodes, never
untrusted C token text. Escape question marks so trigraph replacement cannot
alter source, and delimit numeric escapes so following digits cannot extend
them. Embedded NUL is data, not a terminator.

Documentation uses a dedicated non-executable comment node. Normalize CRLF/CR
to LF; encode non-ASCII/control bytes as printable escapes; break every */ and
?? sequence; encode backslashes so line splicing cannot expose tokens.
The renderer owns comment delimiters. A comment containing #include,
backslash-newline, a trigraph, or a closing delimiter must remain only a
comment. Portable string contents are not normalized by this comment policy.
Discard(value) is a separate typed statement rendered as a void conversion;
it supports unused values/parameters without widening the value type domain.

## Resource accounting and admission budgets

Generation capacity is not a generic AST arity/type error or runtime allocator
failure. CResourceError identifies the exact resource category, source owner,
measured use and admitted limit. Counts/sizes use checked arithmetic. Before
rendering, account for:

- complete object layout: size, alignment, padding, array products and largest
  allocation request; opaque handles do not hide private-layout overflow;
- encoded literal bytes, escaped source bytes and output package bytes;
- declarator/type/expression/control nesting;
- parameters per prototype, fields per aggregate, cases per switch, statements
  per function and declarations per translation unit;
- identifier bytes, external symbol uniqueness and specialization count.

Representation limits come from the pinned ABI. Conservative compiler budgets
are explicit policy constants owned by resources/, not inferred from whether
one native sample happened to compile. Stage 04 must choose and record numeric
limits beside reproducible positive boundary probes and one-over-limit
rejections. Until that numeric inventory and probes exist, no C render-ready
capacity adapter can be exposed. This document does not assert an unmeasured
numeric threshold as proven. A later budget increase requires rerunning probes
under both compilers and updating the evidence.

## Exact native commands

The Bazel native harness invokes the pinned compiler with the following
common arguments for generated and consumer translation units:

    -std=c17 -Wall -Wextra -Wpedantic -Werror
    -Wstrict-prototypes -Wmissing-prototypes
    -fno-fast-math -ffp-contract=off

Include directories and source paths come from typed file roles. Math adds
-lm at link time only when declared. Run normal native binaries at -O0 and -O2.
Run GCC 14.2.0 separately with -fsanitize=address (ASAN_OPTIONS includes
detect_leaks=1:halt_on_error=1) and -fsanitize=undefined
-fno-sanitize-recover=undefined (UBSAN_OPTIONS=halt_on_error=1), at both
optimization levels. Every generated test entry point and the separate ABI/
ownership consumer must execute, not just compile.

The isolated compiler-negative category is ScalarForStructInitializer:
declare struct poly_negative { int value; }, then initialize an object of that
struct from scalar 1. The paired valid initializer is {1}. Both variants
otherwise share a valid main(void), declarations and compiler flags. The
negative must fail for the incompatible initializer, not an unrelated missing
prototype/include/link symbol; the positive must compile and return success.
Only a closed test-fixture renderer may construct this invalid source. It
cannot produce a production certificate or be selected by a file-role string.

## Required target inventory

Existing targets remain regression obligations. The following additional
labels are planned and must actually be added before their owning stage closes.

| Label under //crates/backend-c: | Owner | Required evidence |
| --- | --- | --- |
| portable_backend_c_test | Existing; all stages | Focused structural, registry, verifier and plan tests |
| c_typed_compile_fail_test | Existing; all stages | Invalid Rust construction/mapping/proof boundary rejects |
| c_grammar_inventory_test | 04 | Every closed AST variant has positive, mutation and native coverage |
| c_ast_compiler_oracle_test | 04 | Certified ASTs compile; rejected contextual mutations cannot certify |
| c_resource_probe_test | 04 | Recorded numeric boundaries, both compilers, checked capacity errors |
| c_fp_environment_test | 05 | Platform properties, noncontraction and float edge/raw-bit matrix |
| c_ownership_fault_test | 06 | Fail every allocation index; no leaks/double drop; unchanged inputs |
| c_public_abi_test | 06 | Separate header consumer, lifecycle and flat interface signatures |
| c_capability_plan_test | 07 | All 42 rows/all variants, invoked mappings and rejected plan mutations |
| c_historical_replay_test | 08 | Exact historical case inventory and both native test entry points |

The ownership allocator control records attempts, successful allocations,
releases and currently live bytes/blocks. First run normally to determine N
allocation sites reached for that scenario; then fail each 1..N and run N+1
as a positive control. Each failing path checks exact transport status,
unchanged inputs, empty output and zero leaked allocations after caller cleanup.
A fixture with N==0 cannot stand in for an allocation-failure test.

## No-vacuity and historical membership

Both generated_test and conformance_test contain the same canonical portable
test inventory. Each case checks transport Success, computational outcome
branch and value/error expectation, then increments a completion counter.
The final counter equals the non-omitted inventory length. Case identities
appear in failure messages. A separate mismatched expected value must make
each entry point exit nonzero at that exact case. An intentionally empty
program is tested separately and is not capability or conformance coverage.

The required historical C ports are escape-string-regexp, trim-newlines,
slash, strip-bom, html-escaper, truncate-utf8-bytes, parse-ms,
is-fullwidth-code-point, normalize-newline, has-flag, split-on-first and
stdlib-is-negative-zero. For each, record its checked source identity, test
count, generated native/conformance execution, evaluator agreement and
sanitizer result. Canonical registration, interface/composition and
models-and-validation fixtures are additional obligations. The unrelated
untracked stdlib-abs work is excluded, not silently counted as passing.

Run the full tracked-rule graph, //:release_gate, and
//crates/conformance:polyrust-conformance -- --all-targets --determinism
through Bazel in Linux. Normal caches remain enabled. Existing per-port
differential tests and three-generation package tests supplement the canonical
runner; 50 canonical cases alone do not prove every historical port ran.
