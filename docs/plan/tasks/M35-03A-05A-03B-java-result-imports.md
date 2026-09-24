# M35-03A-05A-03B — Certified Java result-family imports

- Status: complete
- Parent: [Java result transport](M35-03A-05A-03-java-results.md)
- Depends on: [local results](M35-03A-05A-03A-java-local-results.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-scalar-results.md)
- Implementation contract: [nominal imports](../../specification/typed-generation/languages/java/rust-result-imports.md)

## Contract

Derive opaque result-family/type/member/constructor witnesses from the complete
original certified declarations. Retain original owner, declaration identity,
resolved package/member path and closed subtype relationships. Consumers and
relays reference those witnesses, never copied local type registrations or
caller-written metadata. Import membership does not grant declaration ownership.

Extend the language-owned type vocabulary explicitly for authenticated foreign
nominals. Prefer the shared known-type, constructor, callable and member binding
contracts, with an enum distinguishing standard-library types from certified
dependency types. Do not add arbitrary qualified-name strings or a parallel
untyped renderer path. Audit every type-sensitive operation affected by the new
variant, including assignment, patterns, constructor checks, member access,
source inventory, visibility, import discovery and resource accounting.

Only publish families required by exact public signatures or actual public
declarations. Private or unused imported types must not become public API.
Preserve original defining ownership through reexports and relays; independently
certified lookalikes are not interchangeable. The whole dependency closure,
including public nested names and original source export ownership, must agree.

## Definition of done and tests

- Independently generated producer/relay/consumer packages compile separately
  and execute with original types, constructors, payloads and call signatures.
  Type-only uses discover imports structurally and never copy declarations.
- Reject missing/forged/wrong-owner type and member witnesses, wrong authentic
  same-shaped types, signature substitutions, mixed certificates for one owner,
  conflicts/cycles, renamed paths and foreign declarations or subtype additions.
- External Java consumers cannot implement a new sealed variant. Public types
  are accessible where their signatures require them; private helpers remain
  inaccessible. No raw/unchecked casts or null result representations appear.
- Original certificates drive closure and capacity evidence; type-only imports
  cannot bypass budgets. Failed registration/publication is atomic.
- Preserve old outputs/WIP; full Linux release/lint, strict javac consumers,
  normal/interpreted execution and fresh GPT-6-SOL review precede commit/push.

Compiler Result identity and source admission remain 05A-04 work.

## Implementation order

1. Define opaque exported family/type/constructor/accessor authority tied to the
   existing original Java dependency-package identity. Derive layouts from the
   local family proof inside that same certificate; never import package-local
   GeneratedTypeId values as consumer-owned types. Keep publication closed while
   these internal representations are being introduced.
2. Add language-owned referenced-type/constructor/member enums over standard
   symbols and certified nominal symbols. Route them through the shared known
   symbol catalogue and structural discovery. Update every exhaustive match;
   split the standard type catalogue out of ast/types.rs before expanding it.
3. Extend consuming/frozen dependency registration and original-owner closure
   checks to type-only and member-only references. Bind imported signatures to
   the consumer's exact registered types. Reject mixed certificates and owned
   namespace overlap before producing a usable package or partially updated API.
4. Extend the bounded source-owner profile only for explicitly selected and
   authenticated result families. Keep its existing private-record branch strict.
   Verify result-aware bodies and source/JVM bounds before opening publication;
   a declaration-family proof alone cannot authorize arbitrary method bodies.
5. Prove separately compiled producer/relay/consumer behavior, then run the
   complete release/lint/preservation/reviewer loop. Selected-arm mutation proof
   remains 03C, not a reason to skip nominal import tests here.

Do not commit an intermediate public publication path that lacks any of these
checks. Compiler source-type metadata continues to reject unadmitted Result
signatures until the later source-integration milestone.

## Architecture preflight

Independent GPT-6-SOL extra-high preflight found no fundamental contradiction
with the existing shared linker. Its three transition findings are accepted:
normalize producer/exported/consumer signatures in separate phases; replace
infallible function/value-only scope behavior with checked atomic nominal/member
registration; and reconcile exact selected family inventory, including family-only
owners anchored by explicit source-package metadata and public enclosing types.
The implementation contract now specifies these obligations. This preflight is
not implementation review or completion evidence.

## Implementation and completion evidence

Original family/role identities, shared known-symbol wrappers, producer/exported/
consumer signature phases and consuming nominal/member registration are implemented.
The source-owner profile admits only selected canonical families, safe upcasts,
explicit non-null boundaries and guarded payload observations. Family-only
publication retains explicit source-package provenance. Compiler Result source
admission remains closed.

Focused tests pass for separately compiled original producer, importer and relay
packages. Normal and interpreted JVM execution each cover 40 boundary/fallback
combinations, both variants, payloads, copy identity and null boundaries. Five
separately compiled negative Java consumers reject foreign sealed implementations,
private payload access/mutation, wrong-family calls and missing error accessors.
Typed mutation tests cover same-shaped family substitutions, wrong constructor
arguments, accessor metadata, absent/wrong scopes and renamed resolved symbols.
Exact and one-below registration/certification capacity checks include unused
type/member-only dependencies and duplicate registration accounting.

A broader identity audit also found an existing standard-member gap: a Known
StringLength witness accepted a caller-provided different method name with the
same signature. The checker now requires the catalogue spelling too; hashCode
and nonexistent substitutions reject while length certifies. This is a core
certification fix, not an additional supported feature.

The first full release attempt exposed exhaustive-renderer policy violations and
AST size lint failures from inline nominal metadata. Explicit enum arms and
shared immutable handle storage and boxed member signatures address those
findings without lint suppressions or output changes.

The complete Linux Bazel release/lint run passes all 1,041 targets (123 executed)
on code tree `1c3ba3017d707ba498922d705f365119525115c4`, including 476 Java unit
cases, zero failures/ignored cases, and four separately selected native suites.
Invocation `a4d716b6-feb3-402a-91ce-43a3d1db9ec2` took 1,138.895 seconds.
The focused Clippy aspect and policy/compile-fail checks also pass. Independent
GPT-6-SOL extra-high broad review and a final storage/signature follow-up found
no remaining core errors; the architecture preflight findings were implemented.
All 530 prior compiler-bundle files, four character-constant example bundles,
and 45 unrelated WIP files retain their original bytes. The final documentation
checkpoint is re-gated before its scoped commit/push. Hosted CI is tracked
separately; local completion does not claim a hosted success.

Selected-arm execution proof remains 03C; compiler Result admission remains
05A-04. Neither is implied by completion of original nominal imports.
