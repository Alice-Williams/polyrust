# Compiler-owned Box construction inputs

- Status: staged M35-02B-01 contract; no target heap admission
- Parent: [owned values](rust-owned-values.md)
- Plan: [constructor identity checkpoint](../../../../plan/tasks/M35-02B-01-box-constructor-identities.md)

## Source capability

`OwnedBoxConstruction` describes one authenticated constructor operation, not
all Box functionality. Its input borrows the compiler's canonical HIR call and
argument and retains body LocalDefId, callee DefId, GenericArgsRef and result Ty.
Fields and construction are private to the compiler admission boundary; there
is no raw-string or deserialized input factory. Existing Capability, Mapping
and Supports contracts remain the single executable registration mechanism.

After successful compiler analysis, admission must check:

1. The expression is the canonical compiler HIR node for its HirId and belongs
   to the requested body owner. Its direct-call operand is a compiler-resolved
   associated-function path, not a function-item variable or arbitrary callee
   expression whose evaluation could require additional mappings.
2. That FnDef equals the compiler's `box_new` diagnostic item. Renamed imports
   and qualified paths do not change authority; a same-spelled method or wrapper
   function does not acquire it.
3. The instantiated argument is exactly i32; the callable signature is a safe
   ordinary Rust one-argument signature returning the expression's exact type.
4. The result is the compiler's `owned_box` ADT with i32 payload and the type
   instantiated by that authentic constructor, not an independently guessed
   struct/pointer or a spelling-based type lookup.

Rust's [Box source](https://doc.rust-lang.org/src/alloc/boxed.rs.html) identifies
the constructor and owned type with compiler items. The implementation must
verify availability and signatures against our pinned rustc-dev toolchain;
moving online documentation is not proof of the pinned API.

## Binding and evidence

A consuming builder has an empty slot and a registered-mapping state. Only the
registered state can build bindings implementing Supports<OwnedBoxConstruction>.
That implementation returns the stored executable mapping; invocation uses its
declared context/input/output types. Missing, duplicate and incorrectly typed
registrations are compile-negative tests, not runtime string dispatch.

The compiler-only probe compares per-body constructor/type inventories in HIR
and drop-elaborated MIR for its admitted-call fixtures. Additional negative
fixtures show that an authentic FnDef reached via a local function-item variable
or callee block still rejects; its presence in MIR is not operation admission.
Inventory equality does not identify which HIR binding
corresponds to which MIR local, nor the lexical exit for a drop. That relation is
the next part of M35-02B and must reject ambiguity rather than guess from names,
debug metadata, source-span coincidence or traversal position. The C renderer
and no-heap C/Java admission remain unchanged during this checkpoint.

## Reproducible proof targets

```sh
bazelisk test //experiments/rustc-frontend:owned_construction_test \
  //experiments/rustc-frontend:owned_construction_format_test \
  //experiments/rustc-frontend:owned_construction_contract_test
```

The executable is built by pinned Clippy with warnings denied. The dedicated
Rustfmt action includes all nested modules and the fixture. Compiler-negative
targets separately test missing/duplicate registration, wrong capability,
wrong consumer context/output, wrong input and private-input fabrication.
The consumer's typed binding factory fixes context/output; the reusable slot
builder remains target-independent. Compiler failures must have exactly the
expected error code/count and no additional error diagnostics.

Runtime compiler assertions reject copied noncanonical nodes and wrong body
owners. Independent valid-Rust mutations replace a genuine call with a wrapper,
change the unsupported payload into an admitted one, and rename the counterfeit
method. Invalid borrow/move cases fail before any success marker, and an empty
fixture inventory cannot pass. None of these tests construct a target package.
