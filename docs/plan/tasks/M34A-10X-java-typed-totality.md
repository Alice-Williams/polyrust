# M34A-10X — Close Java typed-shape totality gaps

- Status: in-progress
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10W, M34A-10R, and M34A-11

## Goal

Make every currently admitted Java-supported typed shape lower successfully,
without user-triggered invariant panics or new generic frontend restrictions.
An interface with no implementations is explicitly valid.

## Implementation order

1. Add focused typed regressions for restricted Object names, record accessor
   versus interface method collisions, distinct interface signatures which
   erase identically, and interfaces with zero implementations.
2. Make target symbol allocation resolve these conflicts deterministically.
   Preserve readable names where safe; allocate disambiguated names by typed
   declaration identity and use those symbols for declarations, bindings,
   projections, and both dispatch forms. Never merge distinct methods merely
   because their source names or erased signatures coincide.
3. Implement the mapping-owned uninhabited Java interface representation in
   [the language specification](../../specification/typed-generation/languages/java/unimplemented-interfaces.md).
4. Replace preflight rejection tests for these representable shapes with
   successful generation/compiler tests. Retain rejection of forged target AST
   and genuine invalid dynamic inputs.
5. Review every remaining Java-specific rejection against typed constructors;
   list unreachable cases with evidence and resolve any admitted counterexample.

## Definition of done

- All counterexamples generate without panic and compile as Java 21.
- Naming transformations preserve distinct declaration and method identities
  through separately compiled public consumers and concrete/interface calls.
- Unimplemented interfaces do not expose foreign mutable implementations or
  introduce fake generic conformance evidence.
- No new generic reserved-name restriction or implementation-count requirement.
- AST, linker, and renderer remain typed; no raw text escape hatch.
- Changes and tests are split into focused modules, aiming below 500 lines;
  substantial test fixtures do not join production Bazel source sets.

## Tests

- Positive typed construction/generation regressions for each listed shape.
- Native Java consumer tests for renamed fields/functions, independently
  implemented colliding methods, and generic-erasure collisions with differing
  parameter/result types; verify distinct behavior, not only compilation.
- Positive zero-implementation interface fixtures plus foreign-implementation
  and synthetic-instantiation compile-negative tests.
- Mutation tests for permits, synthetic visibility, enum constants, and method
  identity/signature tampering.
- All Java native/conformance tests, Rustfmt, strict Clippy, Buildifier, full
  tracked Bazel graph, release gate, and deterministic all-target conformance.
- Fresh uncapped Sol Extra High review; evaluate every finding explicitly.

## Commit gate

Commit and push a separately identified checkpoint after local proof. Do not
mark Java complete before hosted CI and the review have passed.

## Implementation and audit evidence

- The first typed `hashCode` function/field regression reproduced an invariant
  panic at Java preflight before the repair. It now generates, compiles, and
  runs through a separately compiled public Java consumer.
- Typed identity-keyed allocation covers fields, nominal types, variants,
  constants, functions, interface methods, and locals. Regression consumers
  distinguish colliding accessor/method identities, two interfaces with
  identical erased signatures but different generic parameters/results,
  `Generated` as a user type, repeated enum variants, and contextual-keyword
  parameter normalization. Concrete and interface dispatch are exercised.
- Empty and nonempty interfaces without implementations compile, including
  interface parameter/result and list/option types. Native negative compilation
  rejects a foreign implementation and synthetic instantiation both outside
  and inside the enclosing class. Runtime inspection confirms zero enum values.
- Mapping-output mutations start from a verifier-accepted baseline and reject
  constants, exposed visibility, constructors/factories, missing/wrong permits,
  missing/foreign/duplicate methods, generic-signature changes, and body changes.
- Remaining preflight rejections: malformed feature shapes are rejected only
  when they disagree with the shared checked feature collector; exact strategy
  agreement is separately tracked by M34A-10Y. Fallible constant intrinsics stay
  unsupported on the legacy dynamic path. They cannot be produced by the typed
  constant API: its private `TypedConstantNode` enum has only `Literal` and
  `Reference`, and its factory exposes no intrinsic constructor. Existing
  dynamic rejection regression remains active.
  Zero-variant enums are also unreachable through the typed enum constructor,
  which requires at least one variant. The legacy checked/dynamic path still
  needs an explicit preflight rejection rather than a later AST error; that
  known collector/Java classification mismatch is a required M34A-10Y repair.
- Local focused Java tests pass. Full integration, linters, conformance,
  fresh uncapped review, and hosted CI remain required before completion.

### Final local proof

- Full tracked graph: 432 Bazel rules, 308/308 test targets passed.
- Explicit release gate: 245/245 passed.
- Deterministic conformance: 50 cases and one portable test agree between the
  evaluator and all eight targets; repeated manifests are byte-identical.
- Gates include Java 21 compilation/native consumers, historical ports, curated
  snapshot regeneration, Rustfmt, strict Clippy, Buildifier, and source policy.
- No added third-party dependency. All builds/tests used the Linux dev container.
- Fresh final review disposition and hosted CI are still required for closure.

## Fresh review response

The uncapped Sol Extra High reviews identified the following core defects.
All were accepted; each repair has a regression:

1. Unused concrete implementation method IDs were not independently rooted
   in checked conformance. Each now carries a private-field
   `JavaImplementationWitness`; verification checks the method, interface
   symbol, record owner, and registered Core origins without requiring a call.
   A verifier-accepted baseline is mutated by method ID, interface ID, and
   witness independently.
2. Public option/result factory suffixes still normalized raw nominal names.
   They now consume the same allocated record/enum/interface names as all other
   references. Three typed/native fixtures exercise normalization collisions
   with both option and result types.
3. Synthetic empty-interface type names could collide before the linker ran.
   These requests now join the same nominal allocation namespace before AST
   verification. A typed/native fixture combines `Foo` with a user record
   `UninhabitedFoo` and checks the exact permitted synthetic class.
4. Multiple AST nodes could declare one generated type ID under different AST
   spellings, which the renderer would print under one resolved name. The
   common Java verifier rejects duplicate type identities across the package.
   The privileged empty enum also checks its exact registered name. Mutations
   cover both ordinary records and synthetic enums.
5. Removing a method from both an interface and its synthetic implementation
   could leave a registered method without any declaration. Each file item now
   proves exact, single-occurrence correspondence between its declared-symbol
   inventory and the actual AST types, callables, interface methods, fields,
   and enum values. A joint-removal mutation is rejected from a valid baseline.
6. Registered type kind/name/visibility could drift, including sealed-to-open
   interfaces. Common declaration verification now authenticates all three.
7. Registered callable visibility and generic parameters could drift. Exact
   registered visibility and non-generic portable signatures are enforced.
8. Constants lacked authoritative visibility and declaration shape. Generated
   values now carry visibility; Java checks exact name/type, initialized static
   final fields, and public enum constants. The shared linker respects this
   visibility when allocating bindings and checking cross-file references.
9. Unused checked conformances could disappear, including zero-method edges.
   An opaque checked conformance inventory verifies all record/interface edges
   and implementation method identities independently of call sites. Non-record
   portable interface implementations are rejected, except the certified empty enum.
10. Nested legacy payload-enum matches reused pattern variables. Pattern lowering
    now uses the monotonic temporary allocator; nested native execution returns
    the expected value. Payload variants remain legacy dynamic input, not a new
    typed enum feature.
11. File inventory checks alone did not prove catalogue completeness if both AST
    and claimed inventory were altered. The shared linker now requires every
    type, callable, interface method, and static value to be placed in a file.
12. An interface method could conflict with inherited/final or compiler-generated
    enum methods in its empty implementation. Interface allocation reserves enum
    member names; typed/native tests cover all seven reserved spellings.
13. One synthetic name's numeric suffix could steal another synthetic preferred
    name. All synthetic requests are reserved before allocating any suffix.
14. Registered declarations could move into nested scopes while unchanged
    references retained their original simple names. Java now authenticates its
    flat declaration placement: entry at top level, nominal types directly under
    it, and registered functions/constants directly in their source-unit owner.
    Interface method and enum-value owners retain their exact separate checks.
    The generated entry shell cannot acquire type parameters. Mutations leave
    call/value/constructor references unchanged and are rejected.
15. An enum-typed constant could be recategorized as an enum variant, bypassing
    field placement checks while leaving an unresolved outer-scope reference.
    Core-origin enum constants now require their exact Core enum owner; Core
    static fields require constant origins. A valid enum/constant/getter fixture
    is mutated without changing the getter, and verification rejects the move.
16. Portable nominal names could shadow implicit annotations. The typed
    JavaAnnotation catalogue now provides both renderer spellings and allocation
    reservations; Override and SafeVarargs have a typed/native regression.
17. Type/package expression qualifiers could be shadowed by portable nominal,
    field, or local names. A new native regression first reproduced javac's
    unresolved org.polyrust and cyclic-inheritance errors for record org. One
    catalogue-derived reservation set now covers known type names, their package
    roots, generated shells, annotations, and allocated portable nominal names
    across value namespaces. Typed consumers exercise an org parameter, an
    Objects interface-valued field, and a Math floating-point parameter.
18. Same-file bare constant/function references could bind to a parameter,
    record field, or synthesized accessor instead of the checked global identity.
    The linker now qualifies Core constant/function references with their owner;
    declaration rendering extracts only the member spelling. A separately
    compiled consumer distinguishes global C=7 and foo()=11 from parameter C=99
    and record components C=100/foo=200, through concrete and interface calls.
19. Abstract method parameter scopes were never checked because validation ran
    only for methods with bodies. Scope admission now runs unconditionally; an
    interface mutation duplicates names while preserving registered types.
20. Qualifier-safe lowering alone did not reject forged target-AST bindings.
    The shared qualifier catalogue now also guards parameter/local/pattern scope
    admission, record components, fields, enum values, and type parameters.
    Generated nominal qualifiers are added from the target package context.
    Mutations introduce an org parameter/local while retaining qualified calls.
21. Positive patterns and negated-guard promotion bypassed ordinary name
    admission. Both now use checked scope insertion; pattern expressions also
    reject qualifiers before flow promotion. Positive/negative flow fixtures
    retain valid baselines and reject package, known-type, and local type names.
22. Structural nominal names could shadow packages, annotations, known types,
    or generated shells. Verification now consumes the full shared qualifier
    catalogue. Exemptions require the registered entry origin, the real top-level
    test-file role, or the exact runtime helper under the authenticated Runtime
    shell. Top-level and nested mutations cover all reserved shell categories.
23. Distinct structural declarations could have duplicate top-level names in
    one Java package. Package-wide uniqueness now includes structural and
    registered declarations, with same-file and separate-file regressions.
24. An enum variant reference could be presented as an ordinary generated
    static value, losing its enum owner while preserving its ID/type. Ordinary
    value references now require a real static-field declaration; the enum
    reference form independently requires its exact enum owner. A constant
    initializer mutation is rejected without changing its type or variant ID.
25. Synthetic symbols were resolved as if they belonged to Generated. Their
    typed resolved paths now derive from actual declaration owners. The native
    oracle includes cross-item enum use and two distinct outer classes with
    identically named private generated types, constructors, functions, and
    constants. Declaration names and references use the same canonical path
    spelling even when generic binding allocation supplies a suffix.
26. File-level visibility did not establish Java top-level nest access.
    Package-context verification now checks private ancestors, static members,
    generated record fields, and constructors. Generated construction requires
    a static nested class or an implicitly static nominal. Tests cover all
    constructor/static/private combinations, same-nest positive baselines, and
    cross-nest member/field mutations within a single compilation unit.

The naming-policy review also clarified two non-failure cases. Field suffixes
now reserve requested interface method names before allocation. Enum member
names remain reserved for every interface intentionally: adding/removing all
implementations must not rename the interface API. Preferred names are preserved
only outside this declared compatibility reservation set.

A further naming-policy review found that an ordinary/payload-variant numeric
suffix can occupy a later private synthetic preferred name. This is intentional
ordinary-first priority, not a target validity defect. The specification now
states the two allocation phases precisely: reserve all ordinary requests and
allocate them first, then derive/reserve all private synthetic requests and
allocate those. Reserving private helpers before public names would let changes
to interface implementation count perturb public nominal spellings. The previous
unqualified phrase 'all requested names' overclaimed cross-phase reservation;
it is corrected, without changing the implementation or weakening uniqueness.

### Explicit review decisions

The proposed cross-owner structural-field read was independently reproduced
as an already-rejected input: lexical field verification requires the receiver
type to equal the current owner and the exact field to exist in that owner's
field table. It was therefore not an accepted-AST hole. Generated record-field
references did lack the corresponding nest restriction and were repaired.
The new structural-private access check is defense in depth; this work does not
broaden structural field reads beyond their existing owner-only grammar.
Two old positive tests impersonated PolyError with invented runtime shapes;
they now use ordinary registered test types rather than relaxing runtime identity.

Coherent record-layout rewrites and swapping complete same-signature method
witness/declaration/body associations were evaluated, but are not current target
typing defects. Inconsistent identities and unchanged references are checked;
a coordinated rewrite that remains valid Java is analogous to replacing a
returned literal. This target certificate does not prove arbitrary Core-to-AST
functional equivalence. A future semantic-fidelity witness may strengthen that
separate claim. The second reviewer agreed no unchanged-reference compiler
counterexample was demonstrated for the coordinated method-witness rewrite.

Additional typed/native collision fixtures for every local binding origin are
evidence hardening, not a demonstrated failure: every production local consumer
uses the same identity-keyed allocator. Strategy certificates remain a separate
required implementation milestone, M34A-10Y, not an optional feature.

The curated interface snapshot was regenerated from Bazel output. The import
policy now recognizes genuine Rust `#[test]` functions; failure injection
still rejects commented-out test attributes and adjacent production imports.
All-target deterministic conformance passes: 50 cases and one portable test
agree between the evaluator and all eight generated targets.
