# M35-03A-02F-02B-03B — Authenticated Java constant consumers

- Status: planned
- Parent: [Java constant APIs](M35-03A-02F-02B-03-java-constant-api.md)
- Depends on: M35-03A-02F-02B-03A

## Contract

Complete the consumer half of the [Java constant specification](../../specification/typed-generation/languages/java/rust-public-constants.md).
Register only JavaDependencyConstant witnesses in the existing dependency scope.
Add JavaImportedValue and an explicit dependency-value variant to JavaValueRef.
Derive shared DependencyValueSpec rows and qualified field paths from retained
producer authority; never invent KnownField, free-form paths or synthetic calls.

Update exact scope freezing, references, typing, capability admission, dependency
catalogues, post-link reconstruction, renderer and resource/source accounting.
Traverse all retained value/function owners, including unused registrations and
transitive dependencies, with existing namespace and conflicting-authority rules.
Foreign fields never become owned declarations and cannot be assigned.

## Definition of done and tests

- Values-only and mixed independently compiled Java21 consumers agree with exact
  independent truth. Repeated reads and mixed calls retain one coherent producer
  scope, without Runtime output or boxing.
- Wrong scopes, stale certificates, incompatible primitive types, paths, owners,
  source IDs and coupled catalogue/reference changes reject. Public compile-
  negative tests prevent forged witnesses and unchecked renderer entry.
- Constants-only dependencies compose with call-bearing packages and diamonds;
  no zero-cost shortcut drops real invocation/source bounds. Unused references
  retain owner conflict checks without generating unnecessary code.
- Finality/native-write failures and compiling semantic mutants have independent
  oracles. Generated examples are inspected, ignored and preserved outside Docker.
- Fresh independent review, full isolated Bazel/lint proof and evidence precede a
  scoped push. Complete parent03 only after producer and consumer criteria pass;
  compiler/bundle children04/05 and legacy retirement remain pending.
