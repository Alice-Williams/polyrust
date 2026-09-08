# C17 capability admission and mapping certificates

- Status: normative; reuses the completed Java contract, not Java mappings

The [exhaustive strategy inventory](capability-inventory.md) defines every
capability's closed input family, output category and C-specific obligations.

The shared catalogue's 42 capabilities and all closed input variants define
the migration inventory. `CPluginBuilder.support(mapping)` consumes a Missing
slot and stores the exact checked executable handler. Missing/unsupported
slots cannot derive Supports; duplicates, wrong dialects, input categories
and output categories fail Rust compilation. A catalogue addition forces an
explicit implemented or unsupported decision.

`preflight/` records exact checked uses, structural prerequisites and mapping
slot identity. It does not infer support from a helper name, target string or
the presence of a C AST constructor. It validates the same concrete registry
used by lowering. Structural orchestration remains an explicitly structural
admission, not a fictitious Modules mapping.

Each capability-owned sealed plan is selected from its exact portable/CoreIR
input before its handler runs. The mandatory checked wrapper invokes that
handler and authenticates the mapping-owned output skeleton, typed symbols,
signatures, effects, allocation/cleanup requirements and selected strategy.
Child subtrees are opaque only where they belong to a different mapping.
A forged or ignored plan, different operation, erased output category or
manually attached helper cannot produce a certificate.

`capabilities/` has one snake-case file per capability. Its mod.rs registers
mappings; support machinery and a semantics-free exhaustive dispatcher may be
separate small modules. No all-intrinsics erased input or central second
implementation is permitted.

Modules, Functions and PortableTests include their full declarations/calls/
harness contracts. Interfaces includes its full binding/lifecycle bundle.
Enums means payload-free enums in the typed frontend; older payload enums are
a separately recorded compatibility shape. Local reads use their checked
binding origin. Fallible calls/intrinsics infer ResultPropagation, and boolean
RHS plans preserve conditional evaluation.

Before registering a complete capability, every input variant needs mapping
invocation, exact output-plan mutation and native behavior coverage. Partial
migration checkpoints may explicitly leave slots unsupported; they cannot
claim typed generation for them or silently route to the legacy generator.
