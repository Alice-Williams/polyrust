# M35-03A-02F-02B-02B-02 — Authenticated C constant consumers

- Status: planned
- Parent: [C constant imports](M35-03A-02F-02B-02B-c-imported-constants.md)
- Depends on: M35-03A-02F-02B-02B-01

## Contract

Complete the consumer half of the [C constant specification](../../specification/typed-generation/languages/c/rust-public-constants.md).
Register only certified producer constants as consumer-branded object references;
foreign headers never become consumer output files and imported objects cannot
acquire owned declarations/definitions. Preserve const-qualified storage while
typed reads use the exact unqualified scalar type.

Implement CImportedValue and the shared DependencyValueSpec mapping, catalogue,
unit dependency discovery, resolved spelling and renderer path. Derive every
binding/header from the retained certificate. Integrate complete direct/transitive
export collisions, same-crate authority, file-path conflicts, storage/numeric
facts and resources. Constants-only dependencies may have zero function frames,
but real callable costs may never be dropped.

## Integration order

1. Registry: keep imported objects in a separate witness map, outside owned
   contextual inventories. Split general reference authentication from owned
   declaration authentication. Check value/function imports together for crate
   authority, declaration identity, header-path and symbol conflicts, regardless
   of registration order. Imported references use the consumer registry brand.
2. Checked AST: allow readonly global reads from that map, but never declarations,
   definitions, writes or address borrows. Seed storage facts from authenticated
   immutable values without inserting synthetic definitions. Preserve existing
   numeric and closed-scalar checks, including calls from mixed producers.
3. Shared projection: introduce CImportedValue and a separate imported-value
   binding map. Build dependency catalogue rows from retained witnesses; select
   only actual references for unit imports, while collision checks retain every
   producer export. Keep generated-symbol IDs exclusive to owned declarations.
4. Resolution/rendering: verify DependencyValueSpec against the unqualified read
   type, resolve fixed names from the producer, and deduplicate headers using the
   existing package import kind. Render the ordinary object reference through
   resolved spelling, with no special runtime or declaration-string escape hatch.
5. Composition: traverse dependencies retained by either values or functions.
   Accept a measured zero-frame constants-only certificate without relaxing
   equality to original measured costs or the full resource policy. Exercise
   unused imports, mixed imports, transitive duplicates and diamonds.
6. Prove the complete path using independent native consumers, typed negative
   controls and shared catalogue/reference mutations before exposing it to the
   compiler frontend. Export real sources and run the full gate/review loop.

Keep registry imports, shared imported-value bindings and their tests in focused
modules. Existing function-import behavior must remain covered throughout.

## Definition of done and tests

- Values-only and mixed dependencies compile/link/execute independently under
  GCC/Zig O0/O2 with exact independent bool/i32/i64 truth and readonly failures.
- Repeated value/function imports share one native header while retaining each
  typed binding. Unused exports still participate in collisions.
- Cross-registry references, owned/imported impersonation, conflicting producer
  certificates, stale metadata, wrong types/values/symbols and coupled catalogue/
  reference/import changes reject before publication.
- Compile-negative tests prohibit forged imported witnesses; no safe renderer
  entry point accepts an unchecked package.
- Actual initialization/numeric/readonly facts and direct/transitive resource
  costs are checked; no runtime, address-borrowing or fabricated local definition.
- Focused tests, inspectable ignored examples, clean fresh review and full
  isolated Bazel/lint gate precede a scoped commit/push. Complete parent02B and
  parent02 only after their combined producer/consumer criteria are proved.

## Boundary

Source compiler admission and multi-crate manifest/publication integration remain
children04/05. No custom runtime or old API is retired by this target-only step.
