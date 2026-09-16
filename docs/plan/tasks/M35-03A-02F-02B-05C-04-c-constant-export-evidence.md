# M35-03A-02F-02B-05C-04 — Certified C foreign constant exports

- Status: planned
- Parent: [constant alias closure](M35-03A-02F-02B-05C-constant-alias-closure.md)
- Depends on: M35-03A-02F-02B-05C-03
- Specification: [C constant re-exports](../../specification/typed-generation/languages/c/rust-constant-reexports.md)

## Contract

A selected Rust crate may publicly re-export a scalar constant from a separately
certified C producer, including when it owns no definitions. Preserve every finite
module/name/namespace binding and the original defining declaration and producer
certificate. Do not fabricate a local constant, wrapper function, macro alias or
runtime. C names remain the defining producer's ordinary object names.

Keep the unresolved source graph, registered imported-object witnesses, certified
foreign binding views and owned definition views separate. A name/DefId alone is
not authority. Missing, stale or conflicting witnesses reject rather than
silently falling back to local inventory.

## Implementation

1. Add a focused export-selection module which reconciles explicit source-package
   metadata with registered CDependencyConstant witnesses. Enumerate the finite
   graph once, without recursively expanding alias paths. Retain all foreign
   bindings; deduplicate defining witnesses separately. Value namespace and exact
   defining identity are mandatory. Unsupported foreign modules/functions/types
   and missing imports remain errors.
2. Derive export-only imported-object references in the public header from this
   checked selection. Shared dependency linking must emit the original producer
   header even if no function reads the constant. Reconstruct the selection and
   per-file references during independent projection verification. Keep this in
   the typed projection of package metadata, not raw include text or a fake read
   expression in a function body. A public header's selected imported-value
   bindings can supply the existing shared DependencyValue references.
3. Admit a zero-owned-definition public pair only when an explicit source package
   has nonempty, fully witnessed foreign exports. Keep ordinary empty packages
   rejected. Refactor structural layout checking separately from this semantic
   admission so all consumers use the same checked decision. Audit projection,
   platform assertion installation/verification, documentation routing and
   resource traversal: structural file ordering alone must not authorize an
   empty public API. Require a public owned declaration or authenticated foreign
   binding, and an owned definition or authenticated foreign binding.
4. Extend CDependencyApi with a distinct read-only foreign-constant binding view.
   Keep constants()/constant() owned-only. Resolving a foreign binding returns the
   original CDependencyConstant, never authority newly branded as the facade.
   Validate the complete union of local function/constant IDs and foreign binding
   IDs against the selected export graph.
5. Include the selected crate in dependency self/cycle checks even if it owns zero
   declarations. Preserve all existing complete-producer symbol collision,
   certificate identity, resource-budget and transitive closure checks. Retain
   original producer documentation in the producer and facade module docs locally.
6. Keep production rustc foreign-export admission and bundle schemas unchanged
   until the Java and publication follow-ups provide their required evidence.
   Export real generated C proof fixtures locally for inspection.

## Definition of done and tests

- Positive: direct, renamed, repeated and transitive aliases; multiple bindings
  to one producer constant; multiple producers; mixed owned and foreign values;
  alias-only packages; reversed file order; finite local module-alias cycles.
- For each supported scalar (i32/i64/bool), independently compile every header and
  producer/facade/consumer source with pinned GCC and Zig at O0/O2. Assert exact
  values, one defining object, original symbol/header identity, no fake functions
  or runtime files, and module documentation placement.
- Negative: missing/replaced/conflicting producer witness, wrong namespace or
  declaration kind, unsupported foreign module, stale graph/alias, missing or
  extra projected dependency, empty package, self dependency and complete
  producer/owned symbol collisions. Coordinated projection mutations must fail
  the independent verifier, not only the frontend constructor.
- Certification-derived API views must retain original authority after the
  original API handle is dropped; owned and foreign inventories stay disjoint.
  Compile-fail probes demonstrate that foreign certificate views cannot be
  constructed from strings/unchecked ASTs.
- Existing local/private/owned constant and function paths remain enabled.
  Full Linux Bazel release, Rust/Bazel lint and native gates pass on the exact
  scoped Git tree. Fresh Sol Extra High reviews report no unaddressed core errors.
- Record evidence, commit with this task ID and push. Do not mark the parent
  re-export/compiler publication milestone complete at this checkpoint.
