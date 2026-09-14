# Rust owned-value evidence and C mapping

- Status: staged implementation contract; heap translation is not yet supported
- Plan: [M35-02](../../../../plan/tasks/M35-02-rustc-owned-values.md)
- Existing boundary: [C HIR lowering](rust-hir-lowering.md)

## Compiler evidence first

The first extension targets `Box<i32>`, followed by Boxes containing admitted
scalar-field records and record fields that own Boxes. String and Vec remain
outside this milestone. This is a staged contract, not a claim that these forms
already generate C or Java.

Continue requiring successful full rustc analysis. HIR and TypeckResults retain
structured source operations; compiler MIR supplies additional cleanup evidence.
An initial MIR Drop is not itself a guaranteed destructor call: drop elaboration
accounts for initialization and moves, including conditional and partial drops.
Use the compiler's result rather than copying that source-language dataflow
analysis into our frontend. See [rustc drop elaboration](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html).

Probe `mir_drops_elaborated_and_const_checked` on the pinned compiler before
consumers such as `optimized_mir` can steal intermediate bodies. Verify the exact
returned phase and retain compiler-owned types; never parse printed MIR. Query
availability/lifetimes must be proven on our pinned toolchain, not inferred from
moving nightly documentation. See [MIR queries and ownership](https://rustc-dev-guide.rust-lang.org/mir/passes.html).

## Typed identity and correspondence

Keep body owners, locals, places/projections, function DefIds and instantiated
types as compiler types inside the isolated adapter. Recognize Box through its
language-item identity. Constructor and clone mappings require authenticated
compiler declarations and concrete type arguments; text such as `Box` or `new`
cannot select a capability.

MIR local indices and debug names are not a generic AST, and debug names are not
a proof of correspondence to HIR bindings. In particular, optimizations can map
several source bindings to one place. See [MIR locals and debug information](https://rustc-dev-guide.rust-lang.org/mir/index.html).
The next checkpoint must establish an unambiguous compiler-owned relation for
every admitted structured operation/exit or diagnose the unsupported shape.
The observation probe produces no RenderReadyPackage or ownership certificate.
See [pinned observations and fixture assertions](rust-drop-observations.md) for
the exact Rust 1.98.0 query phase and the remaining correspondence obligations.

## Structured C and runtime contract

Continue using existing typed C construction, verification, linking and resource
admission. The renderer consumes only a certified package. Owned operations get
executable typed capability bindings; imports arise from those registered types
and operations. HIR remains the structured source for blocks/branches/returns;
MIR graph edges do not authorize generating goto-based output.

Before admitting an owned shape, specify initialization, move/clone behavior,
allocator provenance, cleanup order and every exit. Native result equality alone
does not prove cleanup. Require allocation/drop event oracles, sanitizer runs,
failure injection and mutations that remove/duplicate required cleanup.

Keep abort-mode source compilation. Allocation failure must not silently become
a recoverable Result. Custom allocators, custom Drop, unwinding, unsafe, raw Box
conversions, leak/forget operations, threads and async require separate contracts
and otherwise diagnose. Do not claim safe Rust is leak-free or that memory
allocation events cannot be optimized. The exact selected runtime failure policy
must be fixed before the corresponding C mapping is enabled.

The existing no-heap C/Java path remains supported and fully gated. Java does not
inherit heap support from a C implementation; it needs its own mapping. Existing
C ownership checks and pending M34 work are not retired by a successful probe.
