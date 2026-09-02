# Typed target-AST architecture proposal

- Status: historical design proposal; resolved by ADR-0004
- Normative replacement: `docs/specification/typed-generation/README.md`
- Implementation status: no production implementation started
- Current port status: M34-03 is frozen until this proposal is accepted and
  implemented
- Accepted design decision: the portable frontend exposes composition,
  interfaces, traits, and polymorphism, but no inheritance; a target dialect
  may model inheritance as an explicit target-only construct

This file preserves the design discussion. Its open-decision wording is not
normative; ADR-0004 and the layer/language specifications contain the accepted
decisions.

## 1. Problem statement

The M30 architecture solved one problem but not the whole problem. It moved
dependency directives into structured import values and made fragments carry
their imports and runtime-helper roots. It did not require executable target
syntax to be represented structurally.

The current contract explicitly permits either a target AST or the generic
`Document` algebra. Consequently, a backend can put a complete Java method,
class, test, or runtime section in `RawText`, attach a separately maintained
list of `JavaImport` values, and still meet the written M30 contract. The
dependencies are dynamic at the file level, but the translation is still
partly a string template.

This is visible in every backend:

| Output | Current executable `RawText` boundary |
| --- | --- |
| Rust | generated source and runtime fragments become raw documents |
| TypeScript / JavaScript | generated and runtime fragments contain target text |
| Python | generated declarations, runtime, conformance, and negative tests contain target text |
| Go | generated declarations, runtime, conformance, and preambles contain target text |
| Java | generated declarations, runtime, conformance, negative tests, and file scaffolding contain target text |
| C++ | generated and parsed runtime fragments contain target text |
| C | generated and parsed runtime fragments contain target text |

The current compliance ledger is therefore evidence for **dependency-fragment
compliance**, not for a typed language-translation layer. It must not be used
as evidence for the stronger architecture proposed here.

## 2. Design objective

PolyRust must be a compiler pipeline whose final language translation is
inspectable structured data:

```text
authoring frontend
    -> unchecked PolyIR
    -> checked semantic PolyIR
    -> canonical CoreIR
    -> exhaustive enum-based language plugin lowering
    -> unresolved typed target AST package
    -> target linker and name/dependency resolver
    -> resolved typed target AST package
    -> private typed render view
    -> strict embedded Handlebars renderer
    -> OutputManifest
```

No stage before the renderer may produce executable target source text. No
stage after CoreIR may reinterpret portable semantics.

The architecture must support:

- the current Rust builder frontend;
- a future restricted-Rust parser or macro frontend;
- additional source frontends which all converge on the same unchecked
  PolyIR;
- additional output languages implemented as plugins;
- nominal interfaces, explicit interface conformance, and interface-typed
  values without inheritance;
- language-specific types, declarations, expressions, statements, files, and
  package conventions;
- compile-time typed mappings for known target types and methods;
- automatic imports/includes derived from typed symbol references;
- structural runtime helpers selected by typed helper references; and
- deterministic generation and native equivalence testing.

## 3. Non-goals

- PolyRust is not an arbitrary source-to-source translator.
- One universal AST is not required to represent every construct of every
  target language.
- The target AST is not intended to parse arbitrary handwritten programs.
- CoreIR is not machine IR, SSA, LLVM IR, or a mandate to emit low-level code.
- Identical source formatting across target languages is not a goal.
- Runtime reflection over rendered source is not permitted.
- Class inheritance, interface inheritance, inherited state, inherited
  implementation, `extends` semantics, `super` calls, and overriding are not
  exposed by portable PolyIR or CoreIR.
- Target dialects MAY model their own inheritance syntax when a plugin
  deliberately needs it for implementation or target-specific interoperation.

## 4. Normative layer boundaries

### 4.1 Frontend and unchecked PolyIR

A frontend constructs unchecked PolyIR with source locations and unresolved
names. The existing Rust builder remains one frontend. A future Rust parser
MUST produce the same unchecked PolyIR and MUST NOT directly construct
CoreIR, a target AST, or an `OutputManifest`.

### 4.2 Checked semantic PolyIR

The checker owns portable name resolution, typing, exhaustiveness, evaluation
order rules, and capability legality. `CheckedProgram` remains immutable and
unconstructable outside the checker.

Checked PolyIR describes the portable language. It MUST NOT contain Java,
Rust, TypeScript, JavaScript, Python, Go, C++, or C names, imports, syntax
nodes, runtime-helper IDs, or package-layout decisions.

### 4.3 Canonical CoreIR

CoreIR is the single target-neutral input to every language plugin. It removes
portable authoring sugar and makes the behavior that all plugins must preserve
explicit.

CoreIR MUST:

- contain only resolved, typed nodes;
- make left-to-right evaluation order explicit;
- make temporary values explicit where duplicating an expression could change
  cost, allocation, or failure behavior;
- normalize equivalent authoring forms to one representation;
- retain structured source-level control flow suitable for readable generated
  source;
- carry stable semantic operation IDs rather than target spellings;
- retain source provenance for diagnostics; and
- pass an independent verifier before any plugin receives it.

CoreIR MUST NOT:

- select a target-native type;
- select imports, files, packages, or runtime helpers;
- encode target operator precedence;
- contain rendered tokens; or
- weaken an unsupported semantic operation into an approximation.

CoreIR is intentionally smaller than checked PolyIR but higher-level than
machine IR. It is the semantic contract tested by the reference evaluator.

### 4.4 Unresolved typed target AST

Each plugin owns a real AST for its target language. For Java this includes
types such as `JavaType`, `JavaExpr`, `JavaStmt`, `JavaMember`,
`JavaTopLevelItem`, and `JavaFile`. Other plugins expose corresponding
language-specific categories.

There is no required universal expression or declaration enum. The shared
layer supplies only generic package, file, symbol, helper, provenance, and
determinism infrastructure.

Every AST node MUST:

- represent one grammar-level construct;
- distinguish grammar categories in its Rust type;
- contain child nodes rather than target source fragments;
- use validated identifiers and literals;
- use `LocalSymbolId`, `ExternalSymbolId`, or `HelperId` for references; and
- be validatable without rendering.

An AST MUST NOT contain an executable `Raw`, `Verbatim`, `Snippet`,
`Template`, token-string, or equivalent catch-all variant.

The lowering pass is the only layer allowed to decide how a CoreIR operation is
expressed in the target language. It may select a native operation, a target
library symbol, a structural helper reference, or an explicit unsupported
diagnostic.

### 4.5 Typed symbols, types, and callable catalogues

Closed sets of target behavior MUST be Rust enums. Typed IDs are reserved for
references to declarations whose number and names are defined by the input
program.

Examples of closed enums include:

- primitive and known standard-library types;
- known standard-library methods, constructors, fields, and operators;
- runtime helpers;
- invocation kinds;
- declaration and visibility kinds;
- symbol origins;
- portable feature families and feature variants;
- target lowering strategies; and
- template IDs.

Examples of valid dynamic identities include `GeneratedTypeId`,
`GeneratedMethodId`, `InterfaceId`, `InterfaceMethodId`, `LocalVariableId`,
and `FileId`.

Production code MUST NOT compare the text of an ID to select behavior. Text is
metadata for diagnostics and final spelling, not a semantic discriminator.

Every known external callable MUST have one authoritative specification
containing:

- its owner type and symbol origin;
- target-language name;
- constructor, static, instance, field, or operator kind;
- generic parameter shape;
- receiver type, when applicable;
- parameter types;
- return type;
- failure behavior;
- package/module ownership; and
- visibility/import policy.

A declarative Rust definition SHOULD generate the symbol and callable enums,
their metadata, typed constructors, exhaustive match arms, and catalogue
tests. Mapping code MUST use those generated typed constructors rather than
combining an owner string, method-name string, and argument list.

Known mapping operations SHOULD use phantom-typed expression handles such as
`JavaExpr<JavaDouble>` and `JavaExpr<JavaLong>`. A known method constructor
accepts and returns the precise expression types in its signature. Dynamic
program declarations use typed IDs and are checked by the target-AST verifier,
because arbitrary input declarations cannot be represented as Rust compile-time
types.

### 4.6 Interfaces and composition

PolyRust interfaces are nominal behavioral contracts. An interface contains
named method signatures and no state or implementation. A type explicitly
conforms to zero or more interfaces.

The initial interface model supports:

- named interface declarations;
- pure methods with immutable receivers;
- portable parameter and return types;
- explicit implementations;
- conformance to multiple independent interfaces;
- statically known calls; and
- interface-typed values with dynamic dispatch.

Portable PolyIR and CoreIR exclude class inheritance and interface inheritance.
Portable interfaces do not extend other interfaces, and portable
implementations do not declare base classes. There is no portable `super`,
protected inherited state, override chain, or inherited default
implementation.

Composition is explicit:

- a record may contain another value as a field;
- behavior delegates through an explicit field access and method call;
- no target-native embedding or member promotion changes the portable method
  set; and
- a composite capability is expressed as a value containing independently
  typed components, not as an inherited interface hierarchy.

Interface conformance is not implementation inheritance. A target MAY use
`implements`, Rust trait implementations, Go interface satisfaction, Python
protocol conformance, or another target-native representation to express the
checked method set. Any inherited implementation remains target-local and MUST
preserve the same portable observable behavior.

Target strategies are explicit enum values:

| Output | Proposed interface strategy |
| --- | --- |
| Rust | trait plus explicit `impl`; trait object for interface values |
| TypeScript | flat `interface` plus explicit generated conformance |
| JavaScript | object-method dispatch derived from the checked TypeScript mapping |
| Java | flat `interface` plus `implements`; no `extends` |
| Go | flat `interface` plus generated conformance assertion; no embedding |
| Python | flat protocol conformance for typing; no inherited portable implementation |
| C++ | composed type-erased handle and function table by default |
| C | composed context pointer, function table, and explicit clone/drop policy |

Dynamic interface values MUST preserve PolyRust value and ownership semantics.
C and C++ function-table representations therefore require explicit,
verified clone/drop behavior rather than borrowed untyped pointers.

Interface declaration, conformance, static dispatch, and dynamic dispatch are
distinct feature enum variants. A plugin reports each as `Native`, `Emulated`,
or `Unsupported`. A missing feature rejects only a program-target pair that
uses it; it does not prevent unrelated targets or programs from compiling.

Target-language ASTs MAY include typed heritage nodes such as
`JavaClassHeritage::Extends` or `CppBaseSpecifier` even though CoreIR has no
inheritance node. Such a node:

- MUST be constructed only by target lowering, a structural target runtime
  helper, or an explicitly target-specific plugin extension;
- MUST use typed class/interface references rather than source strings;
- MUST be validated by the target AST verifier;
- MUST NOT be constructible from the generic frontend;
- MUST NOT cause inherited behavior to appear in the portable method set;
- MUST add at most one generated inheritance edge;
- MUST NOT extend another generated subclass or be used as the base of another
  generated subclass;
- MUST NOT use multiple inheritance, mixins, or interface-extension chains;
- MUST NOT be used to share implementation or mutable state;
- SHOULD be restricted to a leaf adapter for an external framework or
  target-required base type;
- SHOULD delegate each implemented or overridden entry point immediately to an
  explicitly composed component;
- MUST be documented as target-specific; and
- MUST have focused adapter, delegation, native, and equivalence tests for the
  portable behavior it implements.

This preserves room for a Java, Python, or C++ plugin to integrate with a
framework which requires inheritance without adding inheritance to the
portable language. The target verifier computes the generated heritage graph
and rejects a chain, cycle, generated subclass-as-base, or multiple-inheritance
shape before rendering. An external framework may internally have its own
hierarchy, but PolyRust contributes only one direct adapter edge and does not
depend on deeper inherited implementation.

### 4.7 Target linker and resolver

Imports must be **derived**, not manually attached to source fragments.

The unresolved AST refers to external entities through semantic symbol IDs.
The target resolver traverses those structured references and determines:

- whether a reference is local, implicitly available, imported, statically
  imported, namespace-qualified, or fully qualified;
- how name collisions are resolved;
- which package/module dependency owns an external symbol;
- which helper roots are selected;
- the deterministic transitive helper closure;
- which file and file group owns every declaration;
- C/C++ declaration/definition and header/source placement;
- required exports and visibility; and
- the final spelling of every resolved reference.

This traversal is not a source-text repair scan. It is the only name-resolution
pass over typed target AST nodes. No mapping manually calls
`require_java("java.math.BigInteger")` or maintains a parallel import list.

The resolver produces a different type, `ResolvedPackage<D>`. Its constructors
are private to the resolver. Rendering an unresolved package must be impossible
at compile time.

### 4.8 Structural runtime helpers

A runtime helper is a stable `HelperId` whose implementation is a list of typed
target-AST declarations. Calls to helpers use `HelperId` rather than local name
strings.

Helper bodies may refer to other helpers and external symbols through typed
references. The linker computes their transitive closure, places each selected
helper once, and derives all resulting imports/includes.

Checked-in Java, Go, Python, TypeScript, Rust, C++, or C runtime source
templates MUST NOT be copied, split by markers, or parsed into opaque document
fragments during production generation.

Initially, runtime helpers SHOULD be authored with typed AST builders. A future
quasiquotation or parser convenience layer MAY be added only if it produces
the same typed AST, resolves every reference structurally, rejects parse
errors, and introduces no opaque executable node.

### 4.9 Files and file groups

`TargetPackage<D>` contains deterministic `TargetFile<D>` values organized by
`FileGroupId`. A file contains structured language items and target-owned file
metadata, never one body document.

Examples of structured file data include:

- Java package declarations and top-level types;
- Rust module attributes, imports, and items;
- ECMAScript modules and exports;
- Python module docstrings and statements;
- Go package declarations and top-level declarations;
- C/C++ header guards, linkage blocks, declarations, and definitions.

Import/include lists in a resolved file are produced only by the resolver.
File grouping MUST NOT be used as a dependency side channel.

### 4.10 Strict Handlebars renderer

A renderer accepts only `ResolvedPackage<D>` or `ResolvedFile<D>`. It MUST NOT
receive checked PolyIR, CoreIR, semantic capabilities, an unresolved symbol, or
a helper registry.

The renderer owns:

- keywords and punctuation;
- precedence-aware parentheses;
- whitespace, line breaking, and indentation;
- comments and documentation spelling;
- import/include directive spelling from resolved import records; and
- final source encoding and newline policy.

The renderer MUST NOT:

- select a target type or operation;
- discover or add an import;
- select or expand a helper;
- decide file placement;
- repair an invalid AST;
- branch on a portable operation; or
- inspect rendered text to infer structure.

The renderer converts a resolved AST into private, typed render-view structs
and applies embedded Handlebars templates. Template selection uses a
language-specific `TemplateId` enum and an exhaustive Rust match with no
wildcard arm.

Handlebars is a presentation engine, not the target IR. Templates MAY contain
generic grammar skeletons for files, declarations, statements, expressions,
types, patterns, and imports. They MUST NOT contain feature-specific method or
runtime implementations. Runtime helpers pass through the same generic AST
templates as user declarations.

The Handlebars registry MUST:

- enable strict mode;
- embed and pin every certified template;
- disable script helpers;
- reject missing or duplicate template registrations;
- use no semantic custom helper;
- receive no CoreIR node, capability set, unresolved symbol, or helper ID;
- receive only values constructed by private resolved-view constructors; and
- preserve deterministic ordering supplied by the resolved AST.

Presentation-only `if` and `each` blocks are allowed. Operator precedence,
parentheses, identifier escaping, literal escaping, symbol resolution, helper
selection, and import selection happen before a template is invoked.

External template customization is deferred. If added later, it is
non-certified unless the customized template set passes the complete
equivalence and native verification gates.

`Document` may remain an internal implementation utility beneath Handlebars.
`RawText` MUST be removed from public lowering/package APIs. Production uses
outside renderer modules must fail source policy.

### 4.11 Manifest assembly

Only the generic compiler adapter may convert rendered files to an
`OutputManifest`. Plugins implement typed lowering, resolution policy, and
rendering hooks; they do not implement an unrestricted
`generate(CheckedProgram) -> OutputManifest` escape hatch.

The object-safe backend registry stores compiler adapters around plugins. This
keeps third-party language plugins possible while ensuring they traverse the
same verified pipeline.

## 5. Proposed API shape

The exact Rust names are provisional. The ownership boundaries are not.

```rust
pub trait LanguagePlugin: Send + Sync + 'static {
    type Dialect: TargetDialect;
    type Lowerer: TargetLowerer<Self::Dialect>;
    type Resolver: TargetResolver<Self::Dialect>;
    type Renderer: TargetRenderer<Self::Dialect>;

    fn lowerer(&self) -> Self::Lowerer;
    fn resolver(&self) -> Self::Resolver;
    fn renderer(&self) -> Self::Renderer;
}

pub trait TargetLowerer<D: TargetDialect> {
    fn lower(
        &self,
        program: &CoreProgram,
        options: &BackendOptions,
    ) -> Result<UnresolvedPackage<D>, DiagnosticSet>;
}

pub trait TargetResolver<D: TargetDialect> {
    fn resolve(
        &self,
        package: UnresolvedPackage<D>,
    ) -> Result<ResolvedPackage<D>, DiagnosticSet>;
}

pub trait TargetRenderer<D: TargetDialect> {
    fn render(
        &self,
        package: &ResolvedPackage<D>,
    ) -> Result<Vec<RenderedFile>, DiagnosticSet>;
}
```

The generic adapter performs:

```rust
verify_core(program)?;
let unresolved = plugin.lowerer().lower(program, options)?;
verify_unresolved(&unresolved)?;
let resolved = plugin.resolver().resolve(unresolved)?;
verify_resolved(&resolved)?;
let files = plugin.renderer().render(&resolved)?;
OutputManifest::from_rendered(files)
```

`ResolvedPackage` does not expose constructors to plugins or consumers. A
renderer cannot be called with an unresolved package.

Built-in portable features form a closed enum hierarchy. Every built-in plugin
MUST acknowledge every feature through an exhaustive match. A plugin may
explicitly return `Unsupported`, so adding a feature does not require every
language to emulate it before the compiler can build.

```rust
enum FeatureSupport<S> {
    Native(S),
    Emulated(S),
    Unsupported(UnsupportedReason),
}
```

A generation request fails only when its CoreIR uses a feature which is
unsupported by one of the requested targets. The all-eight compatibility track
continues to require native or emulated support in all eight outputs.

Registration macros MAY generate exhaustive mapping tables, but built-in
feature discovery MUST NOT depend on string keys, linker inventory, runtime
reflection, or a wildcard fallback.

## 6. Example: Java binary64 absolute value

The target lowerer should produce a tree equivalent to:

```text
JavaMethod
  name: floatAbs
  parameters:
    JavaParameter(value, JavaPrimitiveType.Double)
  return_type: JavaPrimitiveType.Double
  body:
    JavaReturn
      JavaKnownStaticCall
        method: JavaKnownMethod.DoubleLongBitsToDouble
        arguments:
          JavaBitAnd
            JavaKnownStaticCall
              method: JavaKnownMethod.DoubleToRawLongBits
              arguments: [LocalReference(value)]
            JavaKnownField(JavaKnownField.LongMaxValue)
```

Each known-method/field enum variant resolves through one typed catalogue entry
which owns its declaring type, name, invocation kind, signature, and origin.
The AST cannot pair a method with the wrong owner. The resolver knows that
`java.lang.Double` and `java.lang.Long` are implicitly available, so this tree
produces no imports. If the tree referred to a typed
`JavaKnownType::BigInteger` symbol, the resolver would derive its
`java.math.BigInteger` import automatically or use a qualified spelling to
avoid a collision.

Neither the lowerer nor the runtime helper contains Java source or a list of
import strings.

## 7. Raw-data policy

Allowed string data before rendering:

- validated identifiers;
- decoded string, character, byte, and numeric literal values;
- comments and documentation content;
- package/module path components;
- external symbol metadata;
- filenames and declared package metadata; and
- non-source documentation, metadata, and asset contents.

Forbidden string data before rendering:

- complete declarations, expressions, statements, blocks, types, or patterns;
- import/include/use directives;
- package/namespace/module declarations expressed as source;
- preambles, pragmas, attributes, or header guards expressed as source;
- executable runtime templates;
- target source fragments with interpolation holes; and
- strings later parsed or scanned to recover dependencies or structure.

Handwritten upstream snapshots and native consumer fixtures are inputs and test
evidence, not generated source. They remain allowed in path-scoped fixture
areas.

## 8. Alternatives

### A. Keep dependency-complete text fragments

This is the current design. It is useful for dependency minimality but cannot
prove that syntax and symbol references agree. A raw method body may refer to a
new type without creating a typed dependency. This alternative does not solve
the reported problem.

### B. One universal target-language AST

A single enum shared by all outputs would reduce apparent duplication, but it
would either become a lowest-common-denominator language or accumulate target
flags and escape variants. It would allow invalid combinations such as applying
Java visibility rules to a C declaration. This alternative is rejected.

### C. Token streams or quasiquotation as the primary IR

Typed tokens are safer than arbitrary strings and are pleasant to author, but
they do not inherently distinguish expressions from statements, resolve
symbols, derive imports, or validate language grammar. Quasiquotation may be a
future AST-construction convenience, not the architecture boundary.

### D. Parse complete handwritten target runtimes

Parsing can prove syntax shape, but a concrete syntax tree alone does not
provide PolyRust's helper identity, external-symbol ownership, import policy,
or package-placement model. Supporting complete parsers for all eight outputs
would also make the generator depend on much larger language surfaces than it
emits. This is deferred.

### E. Per-language typed AST plus shared package/link infrastructure

This proposal chooses this option. It keeps language grammar honest while
sharing the cross-language invariants that genuinely are common: stable IDs,
symbol references, helper closure, file grouping, diagnostics, determinism,
and manifest assembly.

### F. Handlebars as the semantic generator

Feature-sized or runtime-helper-specific templates would recreate opaque source
generation outside Rust. This is rejected. Handlebars is adopted only as the
final presentation layer over a fully resolved typed AST and private render
view.

## 9. Relevant compiler precedent

This proposal follows established separation principles without adopting a
machine-code compiler IR:

- Rust lowers AST to HIR, removing syntax that is irrelevant to later
  analysis and converting sugar to a smaller representation:
  https://rustc-dev-guide.rust-lang.org/hir/lowering.html
- MLIR dialects define their own operations, types, and attributes and use
  explicit conversion passes between abstraction levels:
  https://mlir.llvm.org/docs/LangRef/
- MLIR full conversion succeeds only when all operations are legal in the
  destination representation, which matches PolyRust's proposed
  no-unlowered-node verifier:
  https://mlir.llvm.org/docs/DialectConversion/
- Tree-sitter demonstrates the value of grammar-level typed node categories,
  but its concrete syntax trees are parser infrastructure rather than a
  substitute for PolyRust symbol and helper resolution:
  https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html
- OpenAPI Generator separates transformation into a normalized code-generation
  model from the subsequent application of templates. PolyRust adopts this
  separation while using a substantially stricter typed target AST and
  resolver:
  https://openapi-generator.tech/docs/templating/
- `handlebars-rust` strict mode turns missing template fields into render
  errors and is mandatory for the proposed renderer:
  https://docs.rs/handlebars/latest/handlebars/struct.Handlebars.html#method.set_strict_mode

## 10. Enforceable invariants

The replacement architecture is complete only when tests prove:

1. no generated source role accepts `Document`, `RawText`, `String` source, or
   an opaque executable AST node;
2. every AST category rejects grammatically invalid child categories at compile
   time where Rust's type system can express the distinction;
3. known target methods have compile-time typed owners, receivers, parameters,
   return types, invocation kinds, and origins;
4. closed semantic and target choices use exhaustive enums, while dynamic IDs
   are never compared by text to select behavior;
5. invalid identifiers, literals, symbols, helper IDs, file paths, and package
   metadata produce stable diagnostics;
6. every external reference is resolved or generation fails;
7. duplicate and colliding names are deterministically imported, aliased,
   qualified, renamed, or rejected according to documented target policy;
8. imports/includes are exactly the closure derived from resolved symbol
   references, with positive and negative tests;
9. helper closure is derived from typed helper references, detects missing
   nodes and cycles, and emits each helper once;
10. every runtime helper consists only of typed AST nodes;
11. interface declarations, multiple conformance, static dispatch, and dynamic
    dispatch pass in every supported output while portable PolyIR and CoreIR
    expose no inheritance;
12. explicit component fields and delegation preserve portable value semantics;
13. any target-only inheritance is represented by typed dialect nodes, cannot
    be constructed by the generic frontend, adds at most one generated edge,
    is a leaf adapter rather than a reuse hierarchy, delegates to a composed
    component, and has focused adapter and equivalence evidence;
14. the renderer cannot receive CoreIR or unresolved AST by type;
15. renderers contain no match on portable operations;
16. Handlebars strict mode, complete enum-keyed template registration, missing
    fields, missing templates, and forbidden semantic helpers are tested;
17. no feature-specific runtime implementation exists in a template;
18. lowerers cannot directly construct `OutputManifest` through the plugin API;
19. a no-unlowered-node verifier runs before resolution;
20. a no-unresolved-reference verifier runs before rendering;
21. three full generations are byte-identical;
22. every generated package formats, lints, compiles, and passes native tests;
23. public consumers and C/C++ ABI and sanitizer tests pass; and
24. source policy fault injection proves that each forbidden escape path is
    rejected.

## 11. Proposed migration sequence

No new compatibility repository should be selected during this migration.
The pinned M34 source and completed target-independent `FloatAbs` semantic work
remain valid. M34 package generation resumes only after all supported outputs
use the accepted architecture.

### Phase 0 — accept the contract

- Resolve the open decisions in section 13.
- Accept a superseding ADR.
- Change the existing compliance ledger to distinguish M30 dependency
  compliance from typed-target-AST compliance.
- Freeze new semantic features except changes needed to prove the architecture.

### Phase 1 — shared CoreIR and compiler adapter

- Introduce verified canonical CoreIR.
- Introduce typed unresolved/resolved package states.
- Introduce stable local, external, package, and helper symbol IDs.
- Introduce enum-based feature families and explicit
  `Native`/`Emulated`/`Unsupported` support.
- Introduce nominal portable interfaces and explicit-composition semantics
  without exposing inheritance in PolyIR or CoreIR.
- Seal manifest assembly behind the generic compiler adapter.
- Add compile-fail and fault-injection tests for bypass attempts.

### Phase 2 — Java vertical slice

Java is first because the reported import/runtime example makes the failure
mode concrete. Migrate one complete feature slice through Java types,
expressions, statements, methods, classes, flat interfaces, explicit
composition, runtime helpers, typed known-method catalogues, symbol resolution,
files, and strict Handlebars rendering. Prove automatic imports and collision
handling.

The vertical slice is not permission to leave two permanent Java pipelines.
Its exit condition includes a measured migration plan for every remaining Java
node.

### Phase 3 — structural contrast

- Migrate C next to prove headers, source definitions, includes, interface
  function tables, explicit component ownership, ABI helper placement,
  cleanup, and sanitizers.
- Migrate Rust next to prove paths, modules, attributes, enums, traits, and
  portable composition semantics plus formatter/linter integration.

These three languages exercise materially different source organization before
the shared API is frozen.

### Phase 4 — remaining outputs

- TypeScript and compiler-derived JavaScript;
- Python;
- Go; and
- C++.

Each migration deletes the corresponding legacy fragment/string path. A
backend cannot be marked complete while both paths exist.

### Phase 5 — historical compliance replay

For every previously completed compatibility port:

- regenerate all eight outputs through typed target ASTs;
- run all portable vectors and retained upstream differential oracles;
- run native format/lint/compile/test suites;
- run public consumer and sanitizer suites where applicable;
- compare declared dependencies and helper closures for regressions; and
- record exact evidence in the new compliance ledger.

### Phase 6 — resume M34

- Rebuild the M34 model package on the new pipeline.
- Complete the exact-bit upstream differential oracle.
- Run the complete local and hosted gates.
- Commit and push M34 only after the architecture and equivalence evidence are
  both green.

## 12. Provisional strict audit

This is a planning result, not a downgrade under the still-active M30 contract.
It shows the expected baseline if this proposal is accepted.

| Surface | Typed executable AST | Derived symbol imports | Structural helpers | Renderer isolation | Proposed result |
| --- | --- | --- | --- | --- | --- |
| Shared codegen | Missing | Partial: imports are manually attached | Partial: closure exists, bodies are documents | Partial | **Fail** |
| Rust | Missing | Manual fragment metadata | Runtime documents | Renderer receives documents | **Fail** |
| TypeScript | Missing | Manual fragment metadata | Runtime text fragments | Renderer receives documents | **Fail** |
| JavaScript | Missing | Manual fragment metadata | Derived runtime text | Renderer receives documents | **Fail** |
| Python | Missing | Manual fragment metadata | Runtime text fragments | Renderer receives documents | **Fail** |
| Go | Missing | Manual fragment metadata | Runtime text fragments | Renderer receives documents | **Fail** |
| Java | Missing | Manual `require_java` metadata | Parsed/template runtime fragments | Renderer receives documents | **Fail** |
| C++ | Missing | Manual fragment metadata | Marker-split runtime fragments | Renderer receives documents | **Fail** |
| C | Missing | Manual fragment metadata | Marker-split runtime fragments | Renderer receives documents | **Fail** |

The existing semantic, native, differential, determinism, import-minimality,
and helper-closure tests remain valuable regression tests. They are necessary
but no longer sufficient evidence.

## 13. Decisions for joint review

### Decision 1 — executable raw-source policy

**Proposed default:** no executable raw-source escape hatch in production,
including runtime helpers.

The alternative is a narrowly allowlisted `VerbatimItem` with explicit symbol
dependencies. That is cheaper to migrate but preserves the possibility that
the dependency list and source diverge.

### Decision 2 — CoreIR timing

**Proposed default:** introduce CoreIR now, before target-AST migration.

The alternative is to let target lowerers consume `CheckedProgram` initially
and add CoreIR later. That reduces the first migration but risks copying
authoring sugar and evaluation-order normalization into eight plugins.

### Decision 3 — target-AST sharing

**Proposed default:** share infrastructure and traits, not one expression/type/
statement enum. Each language owns its grammar AST.

The alternative is a parameterized universal surface AST. It reduces boilerplate
for common syntax but creates cross-language invalid states and extension
pressure.

### Decision 4 — migration order

**Proposed default:** Java, C, Rust, then TypeScript/JavaScript, Python, Go, and
C++.

The alternative is Java followed immediately by all higher-level languages,
leaving header/source and ABI requirements until later.

### Decision 5 — commit policy during migration

**Proposed default:** commit and push each phase only when its definition of
done is green; do not commit the frozen partial M34-03 package.

The alternative is to commit the partial port on a holding branch. It provides
little value because it was created against an architecture now under review.

### Accepted decision — portable composition over inheritance

Portable PolyIR and CoreIR support flat nominal interfaces, traits,
polymorphism, and explicit composition. They do not expose class inheritance,
interface inheritance, inherited state, inherited implementation, `extends`,
`super`, or overriding.

A target dialect may represent inheritance with typed target-specific nodes
when its plugin deliberately uses it for implementation or interoperation.
That construct never enters portable CoreIR and must carry its own validation,
documentation, and equivalence evidence.

### Decision 6 — Handlebars boundary

**Proposed default:** use strict, embedded, enum-keyed Handlebars templates only
after target AST resolution. Templates express generic grammar presentation;
they never contain portable-feature or runtime-helper implementations.

The alternative is OpenAPI-style feature-sized templates. That is easier to
author initially but would move the current opaque-code problem from Rust
strings into template files.
