# Rust compiler frontend experiment

- Status: accepted experimental scope; production integration pending M35-03
- Plan: [M35](../plan/milestones/M35-rustc-frontend-proof.md)
- C integration contract: [HIR to existing C types](typed-generation/languages/c/rust-hir-lowering.md)

## Architecture

Ordinary Rust passes through the pinned rustc parser, type checker and borrow
checker. An adapter extracts compiler data after successful analysis. Closed
capability admission accepts only implemented shapes and lowers them into the
existing backend-c registry and AST. Shared verification, linking, post-link
checks and resource admission produce RenderReadyPackage<CDialect> before
CStructuralRenderer spells C17.

The extraction representation is rustc HIR plus its TypeckResults, not a
separately parsed AST or the flattened MIR control-flow graph. Read it only
after successful compiler analysis, including borrow checking. Resolved
declaration/local identities, field indices, types and implicit adjustments
come from the compiler. Reject unimplemented adjustments and overloaded calls.

Preserve structured branches and block scopes. The first admitted C profile
excludes labels and goto nodes. Future loop and cleanup mappings must explicitly
specify structured target lowering; MIR may supply ownership/drop facts but
does not require rendering its control-flow edges as goto statements.

There is no custom Rust/MIR-text parser or duplicate Rust borrow checker.
Invalid Rust and valid unsupported Rust have distinct diagnostics; neither
produces an artifact. Compiler-checked source establishes ownership of the
program itself, rather than ownership of handles in a generator builder.

Preserve resolved textual doc attributes on admitted declarations and members.
Ordinary comments and original formatting are out of scope. Use the existing
typed target comment handling; no source-comment recovery or custom HIR
converter is required. The former miniature C model and renderer have been
removed; no duplicate target AST or renderer fallback remains.

## First subset and boundary

Scalar functions, non-Copy scalar-field structs, local moves, shared references,
field reads and conditional returns. No heap or destructor claim is made.
Native Rust/C receive the same integer-boundary and branch-covering vectors.

Rust establishes input legality; the adapter must still preserve semantics.
Safe Rust can leak, panic or abort allocation. Arbitrary libraries, unsafe,
FFI, custom Drop, threads and async are not automatically supported.

Pin compiler internals and isolate their adapter. Compiler components are
build-time dependencies; generated C needs no Rust runtime. Production typed
AST, linker, capability and renderer obligations remain in force. Existing
C proof work is preserved pending the separate integration decision.

## Sources

- [Compiler driver](https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html).
- [Typed MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html).
- [Compiler HIR](https://rustc-dev-guide.rust-lang.org/hir.html).
- [Borrow checking](https://rustc-dev-guide.rust-lang.org/borrow-check.html).
- [Drop elaboration](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html).
