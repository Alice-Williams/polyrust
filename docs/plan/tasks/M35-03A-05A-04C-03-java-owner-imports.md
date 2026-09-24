# M35-03A-05A-04C-03 — Original Java canonical-owner imports

- Status: planned
- Parent: [Java canonical owner](M35-03A-05A-04C-java-type-owner.md)
- Depends on: [owner certificate](M35-03A-05A-04C-02-java-owner-certificate.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md#java21-specification)

## Contract and implementation

Replace blanket source-root dependency identities with full typed owner keys.
Retain exact immutable certificate authority; clones share it, independently
certified lookalikes do not. Source-only APIs use an optional checked source root.
Keep existing source-crate overlap checks as well as full-owner checks.

Import original family, success constructor/accessor and each error enum constant
through consumer-scoped typed bindings. Enum constants are values, never zero-arg
Error constructors or ordinal lookups. Preserve signature phases, original
membership and all direct unused imports through transitive/diamond closure.

Compiler manifests and graph publication remain source-only until 04D and must
explicitly reject canonical dependencies rather than emit a fake core root.

## Layer-by-layer implementation order

1. Owner authority: replace Authority.root and crate-ID maps with
   TargetPackageOwner<JavaCanonicalTypeProfile>. Expose owner() and optional
   source_root(); no blanket root accessor fabricating core for type owners.
   Keep a separate source-crate collision check: distinct source roots inside
   one crate cannot evade overlap checks merely by having distinct owner keys.
   Canonical instances sharing core are not source-crate collisions.
2. Publication: keep ordinary source inventory validation intact. A separate
   canonical branch derives its one family from the descriptor retained by
   the exact certificate, not caller-selected names or source exports. Reject
   arbitrary family selections for this fixed branch. Source functions and
   constants still require real source ownership and original provenance.
3. Family layout: use a closed error-representation enum distinguishing the
   legacy empty record from the six-kind enum. Preserve old record behavior;
   the enum variant cannot yield an Error constructor. Each error-value witness
   retains its original family, semantic kind, registered constant and resolved
   path. Derive these only from the exact certified selection, never an ordinal.
4. Consumer bindings: retain separate typed scalar-constant and error-kind value
   variants. Bind an error-kind reference together with its exact imported Error
   type in one consuming scope. Freeze both memberships, check them on every
   use and charge them even when no expression reads the imported value.
   No manufactured scalar value, copied enum declaration or raw path promotion.
5. Linker and body validation: dependency value specifications, subtype/coercion
   checks and exported/consumer signatures retain the original owner. Source
   relays may transport the imported family; constructors/accessors never select
   a same-shaped local family. Derive qualified imports from typed references.
6. Closure/resources: key incremental and independently reconstructed closures
   by full owners. Preserve every direct registration in frozen dependencies,
   deduplicate only identical authorities, reject competing certificates, and
   traverse diamonds once. Charge all value/type/member names and the canonical
   enum's source/classfile costs using the existing policies. Extend structural
   source-byte reservation to the exact closed enum owner, not a zero-cost path.
7. Compiler boundary: audit Java bundle preparation/projection and manifests for
   every old source-root assumption. Until 04D, reject canonical owners throughout
   the closure, including unused imports; never emit a fictitious source root.

## Proof matrix

- Dedicated canonical-import tests share the certified 04C-02 fixture, not a
  handwritten Generated.java. Different registries and same-core distinct
  instances must remain distinguishable; cloning the original authority is safe.
- Both producer-to-export and export-to-consumer signature conversion retain
  exact family roles. All six original enum values bind with the Error type;
  wrong-scope, foreign-family, missing-membership and unregistered values reject.
- Error constructor absence and private authority construction have positive
  and compile-fail controls. Existing empty-record constructor tests remain.
- Direct unused, relay and diamond cases assert full owner sets and conflict
  rejection. Exercise the production 1024/1025 closure boundary and exact/one-over
  binding/name counters without replacing production limits with toy limits.
- Standalone javac consumers prove original nominal types and enum values are
  usable; 04C-04 supplies the full cross-producer state/optimization matrix.
- Manifest negative tests include an otherwise valid source package carrying an
  unused canonical import and prove rejection before any publication.

## Definition of done

- Original family/value/member identities survive source relays; same-core
  distinct instances coexist, competing authorities and namespace collisions do
  not. Wrong enum role, foreign subtype and copied-family references reject.
- Unused owners still pay registration, traversal, source and classfile budgets.
  Exact/one-over closure tests exercise the new keys.
- Existing source-only dependency/function/constant behavior and output bytes
  remain unchanged. Manifest negative controls cover unused canonical imports.
- Full Linux Bazel/lint, independent GPT-6-SOL review, protected-WIP audit and
  scoped commit/push. Rust Result source admission remains closed.
