# M35-02 — Owned allocation and cleanup proof

- Status: planned
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
