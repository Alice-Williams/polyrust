# C17 lowering from compiler-checked Rust HIR

- Status: normative design for M35; implementation incomplete
- Target: `org.polyrust.c`, C17 on the existing pinned Linux x86_64 ABI
- Scope: the Rust-source frontend, not a redefinition of portable CoreIR
- Plan: [M35](../../../../plan/milestones/M35-rustc-frontend-proof.md)

## Pipeline and authority

```text
Rust source + pinned compiler configuration
  -> successful rustc analysis (parsing, resolution, typing, borrow checking)
  -> HIR + TypeckResults + resolved documentation attributes
  -> C capability admission and registered typed lowering
  -> existing typed C AST and authenticated C registry
  -> verification -> linking -> post-link checking -> resource admission
  -> RenderReadyPackage<CDialect> -> structural C rendering -> manifest
```

Reuse rustc's HIR. Do not implement another Rust parser, AST-to-HIR converter,
borrow checker, or parser for printed compiler output. MIR may later supply
authenticated drop facts; it MUST NOT dictate goto-based output.

The compiler adapter owns all unstable rustc dependencies. Its C bridge may
inspect compiler HIR directly and construct backend-c AST values through their
checked constructors. The backend's AST, linker and renderer MUST NOT depend
on rustc internals. A separate handwritten Rust-HIR clone or second C AST is
not the production integration boundary.

Analysis success MUST precede C admission for the same crate and configuration.
Compiler facts cannot be forged by supplying a public Boolean, deserializing
an opaque certificate, or mixing data from different compiler sessions.
HIR alone is not a type/borrow-check certificate. Borrow-checking remains
rustc's responsibility even though the converter reads HIR rather than MIR.

## Meaning of the guarantees

Rust analysis proves source legality under the selected compiler settings.
C constructors prove their documented local type/category invariants.
Linking and target certification establish the admitted target obligations.
Native differential tests and mapping tests establish evidence of translation
correctness. None of these alone is a universal equivalence theorem.

Rust source arrives as dynamic input: unsupported source shapes may fail
capability admission. Compiling the converter in Rust does not make every
customer Rust program convertible at converter-build time. The existing
`TypedProgram<R>` / `SupportsAll<R>` contract is unchanged for builder inputs.

Do not duplicate source ownership analysis. Retain C-specific checks for the
representation we introduce: storage duration, qualifiers, initialization,
sequencing, cleanup, numeric UB, declaration visibility and target capacity.
Rust validity does not automatically prove those generated-C properties.

## Admission and source configuration

The initial bridge accepts one selected non-generic safe Rust-ABI function
with exact `fn(i32) -> i32` signature and the shapes enumerated in the
[mapping contract](rust-hir-mappings.md). The experiment selects `score`;
that test harness convention is not a general function-arity limit. Selection
must not promote a source-private function into the production public API;
any test-only adapter has TestSource ownership and is absent from public exports.
Existing `CFunctionType` retains its vector of parameters.

Require safe callable metadata, Rust ABI and non-variadic status explicitly.
Reject foreign declarations/calls, unsafe bodies, raw-pointer operations,
unimplemented generics, mutable/interior-mutability operations, heap owners,
custom Drop, async and unimplemented library calls before output publication.
An identically spelled method or type from another declaration is not support.

Pin edition, rustc/rustc-dev identity, target, cfg, overflow checks and panic
strategy as frontend inputs. Freeze and test the exact configuration before
admitting operations whose behavior depends on it. The initial proof has no
arithmetic/panic/allocation mapping and makes no such behavior claim.
Sanitize ambient `RUSTC_BOOTSTRAP` for input compilation; the adapter's
build-only exemption cannot become customer-source feature authorization.

All files read by rustc, including modules and documentation includes, must
be declared Bazel inputs. Either declare their transitive closure or reject
unsupported external-source forms; merely retaining a rustc span is not a
cache dependency. Never reread untracked files during rendering.

## Capability registration

Before normalized-type lowering, C and Java share the source-owned alias-use
admission visitor in source_admission.rs. It scans unused item surfaces as well
as reachable bodies, with at most 100,000 charged visitor operations and 128
nested guarded categories. Rustc ControlFlow breaks stop both enclosing walks
and the outer item iteration on exhaustion. Budget failures diagnose after
successful Rust analysis and before publication; unused alias declarations alone
remain legal. The shared policy is separate from executable capability contracts
and target resource certification.

Keep the decided builder pattern: implemented typed mapping slots, inferred
requirements, closed enums and executable mappings. A support flag without
the corresponding lowering function is forbidden. One file per capability
owns its admission shapes, lowering, dependencies and tests.

The Rust adapter derives uses from resolved compiler nodes, types, adjustments
and calls, then selects those mappings. Its session-bound typed input adapters
must distinguish Rust semantics from portable operations. Reuse a portable
capability only when its full semantic contract actually matches. Rust moves,
overflow behavior and panic are not silently converted to portable cloning,
Result propagation or recoverable allocation failure.

Every admitted shape has exactly one owned mapping with typed input/output
categories and an invocation contract. All rejected shapes receive a closed
reason plus compiler provenance. Explicitly acknowledge compiler enum variants
when updating the pin; an acknowledged family may reject with a typed reason,
but a new compiler variant cannot silently become supported.

The small experiment does not implement the entire Functions, Records or
Interfaces capability. It MUST NOT register complete `Supports<C>` slots
on the strength of its few fixtures.

## Identity, files and rendering

Use compiler declaration identities, field indices and local binding identities
inside the compiler session; use the existing C registry to obtain typed C
references. Text is a requested spelling, never identity. Register declarations
before body lowering and retain owner/scope on every reference.

Extend `CGeneratedOrigin` with typed compiler-source provenance at integration.
Its payload must identify the crate, declaration/instantiation and body-local
origin as applicable. Do not forge `CoreDeclaration`, use a spelling as an ID,
or label every source item as `Synthesized(Runtime)`.
Session-local DefId/HirId values are not persisted output naming keys.
The shared target graph carries the same metadata in
`GeneratedOrigin::RustSource(Arc<RustSourceOrigin>)`. Projecting a C registration
must preserve that origin, not substitute a legacy Core ID or runtime synthesis.
Neither origin enum is a source-analysis certificate; its truth is established
by the compiler-boundary mapping, separately from target syntax verification.
Deterministic naming uses canonical source identity plus collision handling,
not hash-map iteration, memory addresses or absolute workspace paths.

Files remain `CSourceFile` / `CFileRef` / `CFileRole`. Typed references to
`CScalarType::I32` and catalogued symbols cause the linker to derive headers;
lowering MUST NOT maintain an import string list or emit preambles.
Public headers are independently consumable. The preserved output boundary is
the Rust crate, not each physical source file: flattening files within one
crate is allowed and is the default C strategy. Do not merge separate crates.
Retain source-file/module provenance and declared/effective visibility,
including restricted visibility and re-exports. The
[crate and visibility contract](rust-hir-files-and-visibility.md) is mandatory.

The renderer remains total over the certified target representation and owns
only C spelling, parentheses, escaping and formatting. No HIR traversal,
ownership decision, feature dispatch or include inference belongs there.
No raw C, executable templates or opaque token nodes are introduced.

## Status and precedence

The experiment now lowers directly into backend-c's registered AST and shared
verification/linking/resource certification before CStructuralRenderer. Its
former miniature model and renderer have been removed. This closed one-file
prototype still requires integrated proof, capability-owned mapping slots and
the planned crate/API boundary work; it is not complete production Rust support.

For Rust-source lowering this contract takes precedence over portable-only
assumptions in the C overview: private no-Drop Rust records may be C value
structs, and source analysis uses rustc. The portable opaque-owner ABI and
existing builder behavior do not change. The C renderer still launches no
compiler and the generated C requires no Rust runtime.

Existing cleanup-jump target nodes are not deleted, but the Rust-source
mapping may not emit them. Any future exception requires an explicit design
change and proof, not an accidental MIR edge translation.

See [node mappings](rust-hir-mappings.md),
[documentation and proof](rust-hir-documentation-and-proof.md),
[crate boundaries and visibility](rust-hir-files-and-visibility.md), and
[existing C AST validity](ast-and-validity.md).
