# Conditional owned record compiler observations

- Status: complete; observation only, no conditional body admission
- Plan: [M35-02B-03M-01](../../../../plan/tasks/M35-02B-03M-01-conditional-observations.md)

## Source inventory

Use six safe nongeneric free functions with an immutable Boolean parameter
and three immutable i32 parameters. The first three immutable lets construct
standard Box<i32, Global> from the distinct scalar parameters. A local named
Pair has two fields of that exact Box type and no custom Drop or generic
arguments. Every function returns an i32 through a dereferenced live local Box.

- Conditional initialization: declare Pair without an initializer, then assign
  the complete aggregate only in the true arm. Both outcomes read the third
  independent Box. Include reversed aggregate initializer evaluation order.
- Scoped partial movement: initialize Pair before the if, then extract its
  first or second field into a short-lived true-arm binding. Both outcomes
  subsequently read the independent Box.
- Early return: initialize Pair before the if, extract/read/return one field
  in the true arm, and extract/read the opposite field in the continuation.
  Include both first-field and second-field early-return variants.

No explicit drop calls, unsafe code, mutation after initialization, else arm,
loops, generics, borrowing, custom allocators or nested records occur here.
Compiler observations are not a promise of support for other valid Rust.

## Pinned normal-MIR behavior

Rust 1.98.0 with panic=abort produces three constructor calls and one source
Boolean decision, whose scalar staging copies the actual Boolean parameter.
Traverse both source outcomes independently and evaluate subsequent compiler
cleanup switches only from their actual preceding Boolean constant writes.
Record the complete source/cleanup decision locations and full Return.

Conditional initialization stages an aggregate in a temporary, then moves the
whole Pair into the previously uninitialized binding. It has three cleanup
flag decisions per outcome: initialized record, remaining second source Box,
remaining first source Box. Their observed values are respectively true,
false, false on the true arm and false, true, true on the false arm. The true
path drops Pair then the independent Box; the false path drops independent,
second and first source Boxes. It never drops the uninitialized Pair.

Scoped partial movement has one cleanup-flag decision. On the true path it
drops the extracted Box inside the arm, reads the independent Box, then drops
the remaining record field and independent Box. On the false path it reads
the independent Box, drops both record fields in declaration order, then the
independent Box. Crucially, the intact false-path Pair still uses two field
Drops, not one whole-record Drop: a possible partial move elsewhere in the
body causes field-wise drop elaboration. Do not reuse straight-line grouped
cleanup solely because both leaves remain live on this particular path.

Early-return bodies have no remaining cleanup switches in PostCleanup MIR.
Each arm reads its own extracted Box, drops it, drops the opposite record
field, then drops the independent Box before the shared final Return. Retain
the distinct canonical early-return expression versus root tail expression;
shared MIR Return does not erase that source distinction.

## Required observation oracle

Use actual HirId, AdtDef/DefId, FieldIdx, full Ty, Place and Location, not names,
spans, numeric local choices or debug text, for all assertions. Match canonical
HIR formal parameters with the compiler's MIR argument iterator in typed
positional order, not by raw local numbers. Authenticate
standard constructor identity/type arguments, exact scalar parameter origins,
source-order aggregate staging, declaration-indexed operands, conditional
whole-record assignment and each selected field. Assert actual read producer,
normal cleanup order and canonical source exit. Both traversals together must
cover every normal block; cycles and unexpected statements/terminators fail.
Bound the fixture tracer to 128 blocks and 256 locals before traversal.

For records initialized before the if, aggregate construction must precede the
source decision. No additional promise is made that the pure Copy staging the
immutable Boolean parameter occurs after the aggregate: its provenance and
value are fixed, and staging it earlier does not change ownership or behavior.
Conditional initialization, in contrast, must perform its field-owner staging
only after the true source decision.

The six-function inventory must be nonempty and exact. Invalid source never
publishes observations. Valid source perturbations must fail the intended
oracle rather than a parser/type error. The final suite replaces exploratory
debug output with explicit assertion results. Full historical/native/lint
gates and a fresh review precede completion; M-02/M-03 own safe admission.
