# Checked Rust cross-crate constant re-exports

- Status: checked compiler inventory, C/Java lowering and alias-aware bundle
  publication implemented; end-to-end closure remains in progress.
- Parent: [public constants](rust-public-constants.md)
- Implementations: [C17](languages/c/rust-constant-reexports.md),
  [Java21](languages/java/rust-constant-reexports.md)

## Identity and compiler boundary

A public alias is a binding to a declaration, not a new declaration. Preserve the
re-exporting module/name/namespace separately from the defining Rust DefId and
stable declaration identity. Obtain both from rustc's checked resolved module
bindings; do not resolve aliases by parsing source strings or emitted code.

Retain a compiler-private stable-ID-to-DefId inventory alongside the exact cached
finite export graph. Repeated declaration aliases must resolve to the same DefId.
A collision between distinct compiler declaration definitions is an error. Local declarations retain
LocalDefId and the existing closed Function/Constant kind. Foreign module
constants have a separate private typed descriptor with DefId; never pretend
they are owned local declarations.

The extended inventory admits only public ordinary foreign module constants in
the value namespace. Foreign functions, modules, associated constants, types and
other unsupported exported kinds remain diagnosed. Classification is not scalar
type/value admission and not a target certificate. The standalone inventory constructor stays strict. Authenticated C/Java bundle
assembly uses the extended inventory and deterministically unions foreign exports
with expression-used imports, deduplicating by defining identity. Every member is
authenticated against its original producer certificate before output. The union
retains the 4096-constant limit; classification alone cannot bypass target checks.

## Finite graph and ownership

Keep all local module edges and alias names, deduplicating defining declarations
but never alias bindings. Local module cycles remain finite graph edges, not
unbounded expanded paths. Existing module, binding, name-byte, ancestry, scan and
declaration limits remain active. Unique owned and foreign declarations share
the 4096 selected-declaration limit.

Selected crate provenance must exist independently of owned declarations. A
re-export-only crate has its own module docs and export graph, but no fabricated
function, constant, source identity or copied producer storage. Producer
declaration documentation stays with the defining crate.

## Target authority and publication

Source classification alone must not enable output. Lower each selected foreign
constant through the executable import mapping and the original defining
producer certificate, including aliases with no body reads. Preserve complete
dependency closure and all intermediate selected crate boundaries. A foreign
export's target identity is its producer's ordinary symbol/path.

A target certificate reconciles every owned/foreign binding with the exact
registered owned declaration or retained imported witness. Missing, wrong-kind,
wrong-owner, replaced, or inconsistent type/value evidence is rejected.
Independently certified equal-looking packages are not interchangeable.

Version alias-bearing metadata explicitly. Descriptions retain the re-exporting
binding and defining owner/reference without reconstructing authority. Reserve
and validate every output before atomic publication. Older schemas retain their
meaning. No runtime, accessor, copied field, macro alias, or raw target text is
introduced to simulate a re-export.

## Proof

Use real checked Rust producer, alias-only intermediate and mixed root crates.
Cover direct, renamed, transitive, diamond and nested-module aliases; compiler
visibility; exact defining identity; supported and unsupported kinds; empty-owned
inventories; finite cycles and bounds. Target/native tests separately prove
certificate closure, one defining storage location, exact values, docs and
source-only output. Mutations must fail the proper boundary; changed producer
source must invalidate dependent Bazel results and fail unchanged native truth.
