# M35-03A-05A-03B — Certified Java result-family imports

- Status: planned
- Parent: [Java result transport](M35-03A-05A-03-java-results.md)
- Depends on: [local results](M35-03A-05A-03A-java-local-results.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-scalar-results.md)

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
