# M35-03A-02F-02B-05C-04 — Certified C foreign constant exports

- Status: complete
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

## Review and evidence

The first independent Sol Extra High review identified an evidence gap in the
new zero-owned-root cycle protection. The original negative rejected during
selection and did not reach the changed dependency-closure check. Accepted and
fixed: a facade with no owned declarations exports a constant whose producer
retains an unused dependency on that same facade crate identity. Projection and
linking pass; certification rejects the transitive self dependency. A different
facade identity is the positive control. Direct and transitive complete-symbol
collision tests and a second independent native constant producer were also
added. No finding was dismissed as an optional extension.

Focused Linux Bazel evidence on tree
`252689aefa125fffbc74be31b6224434551521ea`:
`//crates/backend-c:portable_backend_c_unit_test` and
`//crates/backend-c:c_typed_compile_fail_test` passed **2/2**, 59.350 seconds,
invocation `fba88b7c-8dcd-4d8c-b8ea-f0d0f4df3873`.
The C suite passed **765 cases**, with five expensive existing cases scheduled
separately by the full gate.

The first full gate caught a test-only Clippy warning for cloning a reference
into a one-element slice. It was corrected to std::slice::from_ref without
changing semantics or lint policy. That superseded run was intentionally
interrupted and is not completion evidence. The corrected implementation tree is
`390611cded11fc3dd4dd90937d877f1300082055`.

A fresh independent Sol Extra High review of corrected tree
`390611cded11fc3dd4dd90937d877f1300082055` reported **no core findings** after
reviewing all 20 staged files and independently inspecting generated artifacts.
It covered original producer authority, finite alias graphs, exact reconstruction,
zero-owned admission, complete closure/collision/resource checks and documentation.
The reviewer did not run competing builds while the full gate owned the container.

The native proof uses constants-only and mixed producers, a second independent
constant producer, an alias-only facade and a transitive mixed facade. GCC and
Zig compile every one of the four headers independently, including repeat
inclusion, then compile each source separately and link a consumer that includes
only the final facade header. All cases run at O0 and O2 with existing strict
flags. Independent expected values cover false/true, signed minima/maxima, an
i64 above JavaScript's exact-integer range, and computed/owned scalar values.
Owned-object inventories and linking demonstrate no duplicate storage; foreign
API access returns the original retained certificate witness after owner-handle
drop. Existing imported-object registry, independent-certificate diamond and
full dependency resource controls remain enabled.

Real generated sources from that tree's passing native test are exported locally
under `generated/examples/c-constant-exports-390611/{ConstantsOnly,Mixed}/`,
including facade headers/sources and consumer.c. They are ignored, not committed.
The alias-only implementation contains only its generated own-header include;
its header contains derived producer includes, module docs and existing target
platform assertions. It has no fabricated object, function, macro alias or
runtime file. The assertions remain the existing C ABI profile checks.

The parent compiler/bundle work remains open: this backend checkpoint does not
enable production rustc foreign-export admission or remove legacy runtimes.

## Complete release gate

On exact implementation tree `390611cded11fc3dd4dd90937d877f1300082055`,
Linux dev-container command
`bazelisk --output_user_root=/tmp/polyrust-m34a10w-bazel --batch test //... //:release_gate --noshow_progress --noverbose_failures --test_output=errors --test_summary=terse --keep_going`
passed **738/738 tests**, 1,093 targets, 88 executed and 650 cached, in
515.575 seconds. Invocation: `953d8297-46c1-418c-8a8d-1b132d44598a`.
This includes Rust/Bazel lint, generated native-language checks and all five
separately scheduled C capacity tests. None were disabled or relaxed.
All 20 preserved ownership-work hashes remained unchanged.
