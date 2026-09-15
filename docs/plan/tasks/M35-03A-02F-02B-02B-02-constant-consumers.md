# M35-03A-02F-02B-02B-02 — Authenticated C constant consumers

- Status: complete
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

## Implementation and proof inventory

- CRegistry::import_constant accepts only CDependencyConstant and returns a
  consumer-branded CObjectRef. The separate constant-import map retains original
  authority; imported objects never enter owned inventories. Declaration and
  definition builders require owned objects. Values/functions share duplicate,
  source identity, certificate and header-path checks in both registration orders.
- CImportedValue retains its registered object and producer witness. Shared
  catalogue rows derive exact unqualified read types, names, owners and fixed
  package imports. Unit selection, dependency references and resolved names use
  a separate imported-value map; the existing renderer emits ordinary global
  reads. Constant storage remains const-qualified throughout.
- Context validation and complete direct/transitive dependency traversal include
  value edges, including unused registrations. Actual zero-frame producer evidence
  is permitted without weakening measured-cost equality or callable frame costs.
  Same-authority diamonds pass; different certificates for one crate reject.
- Memory entry seeds initialized immutable producer storage without fake local
  definitions. Numeric reads derive exact values through the existing literal
  evaluator. Extreme subtraction is a numeric/storage diagnostic proof only:
  the unsupported arithmetic source profile still rejects it.
- Sixteen focused tests cover readonly/cross-registry/owned-import boundaries,
  metadata and coupled binding mutations, unused imports, complete symbol
  collisions, source-crate impersonation, resource composition and native output.
  Existing legacy and typed target tests remain enabled.
- Native GCC/Zig O0/O2 tests separately compile producers, readers and independent
  truth drivers. Constants-only/mixed and read/unused configurations cover both
  booleans, signed32/64 boundaries, a value beyond binary64 precision and 62.
  All 128 readonly assignments fail compilation. Eight scalar-reference mutants
  still compile/link but fail the truth driver. Four additional native diamond
  runs prove the actual intermediate function call and shared constant producer.
- Generated sources were exported and inspected in ignored host directories
  generated/examples/c-constant-imports-d44a765 and
  generated/examples/c-constant-diamond-d44a765. No generated artifact is committed.

## Verification evidence

Tree d44a765f5a3128da458319dcf287635ba5974b50 passed all six focused targets:
743 ordinary C tests, Rust Clippy, rustfmt, buildifier, documentation and source
policy. Invocation 8d5828cf-d7a5-4c83-98c0-31a3bb13ef7d; 63.359 seconds.
All 2,555 archived Git blobs and executable modes were verified.

The same implementation tree passed the isolated full release/integration gate:
624 tests across 816 targets, 485.948 seconds, invocation
19b0b646-5e5d-42e0-9488-3b37211469eb. This includes 743 ordinary C cases, the
five heavy C capacity shards, 67 C documentation/compile-negative cases,
release_gate, all rustc-frontend experiments, C/Java and shared codegen tests,
documentation and Rust/Bazel/source-policy lint gates. Caching stayed enabled.
A fresh independent Sol Extra High review of that exact implementation tree
found no actionable current-scope errors. It audited the full diff and surrounding
producer and generic value-certificate invariants, including authentication,
readonly storage, exact numeric facts, resolved names/imports, complete dependency
closure, zero-frame costs and the native/negative proof suites.

The final documentation tree is gated again before its scoped commit/push.
This completes the C target constant API, not Java or source/bundle integration.
