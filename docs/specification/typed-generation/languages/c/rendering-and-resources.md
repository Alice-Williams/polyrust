# C17 structural rendering and target resources

- Status: normative for M34A-11

The [platform/resource proof inventory](platform-and-proof.md) specifies
translation-phase escaping, supported floating-point environment, accounting
categories and mandatory measured-boundary evidence before certification.

## Certified formatting

The renderer accepts only certified linked C files through the shared sealed
adapter. Small private modules cover declarators, declarations, statements,
expressions, initializers, directives and literals. Every keyword/operator/
punctuation decision matches a closed enum. No Java grammar is copied.

Full type structure determines parentheses: a pointer to a function, a function
returning a pointer, an array of pointers and a pointer to an array remain
distinct AST types. Binary expressions are parenthesized sufficiently to
preserve the exact tree; formatting never changes evaluation order or drops
temporaries. Literal escaping protects embedded zero/control bytes, hex-escape
continuations and preprocessing translation hazards. Semantic UTF-8 data uses
explicit lengths rather than strlen. Fixed-width minimum values and exact
F64 bits are selected structurally before rendering.

Output is deterministic UTF-8/LF with a final newline. Header guards and
preprocessor assertions are typed directive nodes, never executable macros
which hide operations. Formatting is a no-diff oracle, not a syntax-repair
step. C's format contract is structural canonical formatting plus the whitespace
gate, not equivalence to an external full C formatter. Run
`bazel test //crates/backend-c:generated_v0_style_test`; historical package
`c_style_test` targets use the same tools/c/test_style.sh. Stage 04 additionally
adds `//crates/backend-c:c_structural_format_test`: exact AST formatting
fixtures, three-render no-diff checks and translation-phase hostile text cases.
No third-party C formatter dependency is required or claimed.
Strict native diagnostics and parser/compiler mutation evidence remain required.

## Capacity and platform contract

`CResourceError` is a privately constructed target-capacity diagnostic result,
separate from invalid AST/unsupported capability defects and runtime allocation
failure. Resource checks run after syntax certification. Generic parameter and
field lists remain uncapped; target limits cannot be disguised as typing errors.

Pin and test the admitted platform properties: CHAR_BIT, exact integer widths,
binary64 radix/mantissa/exponent, required two's-complement bit interpretation,
object/pointer/size_t layout and allocator alignment. The current Linux native
proof is the supported platform, not a claim covering every conforming C17 ABI.
Typed static assertions fail unsupported platform compilation clearly.
Resource accounting covers aggregate layout/size multiplication, literal/array
payload size, declaration/declarator nesting, external symbol uniqueness and
compiler translation capacities. Distinguish exact representation bounds from
conservative pinned-compiler admission budgets; the latter are not claims that
the rejected source would necessarily fail a different compiler.

No compiler process or third-party generator enters production. Increasing
a budget or target platform requires positive boundary probes and oversized
negative fixtures, not only updating a constant.
