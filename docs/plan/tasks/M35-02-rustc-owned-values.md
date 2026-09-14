# M35-02 — Owned allocation and cleanup proof

- Status: in-progress
- Depends on: M35-01B, M35-01C and M35-01D (typed C, documentation and crate boundaries)

## Goal

Prove compiler-provided ownership and drop information for real owned values
before replacing the existing C ownership analysis.

## Definition of done

- Document the exact MIR phase and retained type/call metadata.
- Retain HIR-based structured output. If MIR supplies cleanup facts, define
  their authenticated correspondence to HIR and structured exit scopes;
  do not default to emitting MIR edges as goto statements.
- Map a selected subset of Box, String or Vec through authenticated compiler
  declaration identities; a method spelling cannot select a library mapping.
- Preserve conditional moves, partial initialization and compiler drop
  elaboration across branches and function boundaries.
- Specify panic, allocation failure and destructor behaviour. Allocation abort
  must not silently become a recoverable portable Result error.
- Retain allocator provenance and ordinary cleanup obligations without claiming
  that all safe Rust is leak-free.

## Tests and proof

Native Rust/C differential construction, move, clone, nested ownership,
conditional destruction and early-return cases; strict compilers, ASan/UBSan
and a tested failure allocator. Unsupported unwind/custom-Drop cases reject.
Mutation tests detect missing/double cleanup in target lowering.

## Commit gate

Fresh independent review, required Linux/Bazel gates, documented evidence,
commit and push. Production cutover is separate.

## Ordered checkpoints

Start with `Box<i32>`, then owned scalar-field records; String/Vec are deferred.
No backend advertises heap support merely because the compiler probe succeeds.

1. [M35-02A — Pinned compiler drop evidence](M35-02A-compiler-drop-evidence.md).
2. [M35-02B — Structured ownership correspondence](M35-02B-structured-owned-mapping.md).
3. [M35-02C — Typed C Box mapping](M35-02C-c-box-mapping.md).
4. [M35-02D — Native cleanup proof and closure](M35-02D-native-owned-proof.md).

The [owned-value contract](../../specification/typed-generation/languages/c/rust-owned-values.md)
keeps compiler observation, source/target correspondence and runtime cleanup
proof distinct. Existing no-heap C/Java gates and unrelated M34 work remain intact.
