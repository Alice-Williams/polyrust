# Java typed-generation specification

- Status: normative for M34A
- Target ID: `org.polyrust.java`
- Language/toolchain: hermetic Java 21

## 1. Scope and package

The plugin emits a Java 21 package rooted at
`src/main/java/org/polyrust/generated`, including generated declarations,
optional structural runtime declarations, native/conformance tests, and
negative compilation fixtures. A separately compiled consumer MUST be able to
use every portable public API. No undeclared runtime dependency is permitted.

Java is the first inferred typed-program target. `JavaPluginBuilder` MUST
register a typed executable `JavaCapabilityMapping<Capability = C>` separately
for every admitted portable capability. Only an `Implemented<Mapping>` builder
slot derives `JavaPlugin: Supports<C>`, and `Supports<C>::mapping()` returns that exact
handler. Its typed entry point accepts only `TypedProgram<R>` under the bound
`JavaPlugin: SupportsAll<R>`. Typed records, fields, constructors, functions,
calls, values, and operations lower through those registered handlers into the
same certified Java AST, linker, post-link checker, and total renderer.

The typed path has no user-caused Java syntax or capability diagnostic after
construction. The current `CheckedProgram` entry point remains the explicitly
dynamic compatibility path until existing examples are migrated.

The concrete entry point is total at its public typed boundary:

```rust
fn generate_typed<R>(&self, program: &TypedProgram<R>) -> OutputManifest
where
    R: Requirements,
    JavaPlugin: SupportsAll<R>;
```

It delegates to the same verified CoreIR-to-Java compiler as the dynamic path.
Any rejection is converted to an invariant panic identifying a PolyRust defect;
it is not returned as a user validation branch. Java has no manual or empty
`Supports<C>` implementations. No profile, wildcard, or default
implementation can make an unregistered feature admissible.

`JavaPluginBuilder::support(mapping)` infers `C`, consumes the builder, replaces
the single `C` slot from `Missing` to `Implemented<Mapping>`, and requires
`Mapping: JavaCapabilityMapping<Capability = C>`. Every operation mapping accepts a closed,
feature-specific enum containing already-lowered `JavaExpr` operands and
returns a `JavaExpr` or typed `JavaExprPlan`; declaration/type mappings return
their corresponding Java AST category. An erased feature input/output enum,
source text, tokens, imports, helper names, and unchecked AST are forbidden.
The Java lowerer calls the stored mapping. Merely storing evidence while a
separate match performs the real lowering does not satisfy this specification.

Mappings for declarations, values, and control flow MUST accept portable typed
AST or verified CoreIR inputs. Accepting an already-complete `JavaExpr`,
`JavaMethod`, or `JavaTypeDeclaration` and returning it unchanged is not a
support implementation.

The Java capability files and complete catalogue are governed by
[the portable capability catalogue](../layers/00-capability-catalogue.md).

The dynamic `JavaCapabilityRegistry` derives feature presence and strategy
from the same built plugin registration catalogue, then applies its existing
shape-specific checks. It cannot advertise a feature whose mapping slot is
missing.

## 2. Capability strategies

The exhaustive Java registry distinguishes native primitive/reference
semantics from emulated unsigned/checked operations, exact float bits,
immutable byte/list values, tagged option/result, and portable Unicode scalar
validation. Every CoreIR feature is acknowledged as native, emulated, or
unsupported.

Exceptions, `null`, reflection, identity, and implementation inheritance MUST
NOT silently stand in for portable result, option, dispatch, or composition
semantics.

## 3. Java AST

The dialect owns:

- `JavaType` with primitive, reference, array, generic, wildcard, and type-use
  context variants;
- `JavaExpr` and `JavaPrecedence`;
- `JavaStmt`, `JavaBlock`, `JavaSwitchArm`, and patterns admitted by policy;
- `JavaDeclaration`, `JavaModifier`, `JavaMember`, and `JavaHeritage`;
- `JavaCompilationUnit`, `JavaPackage`, and closed grammar/format categories;
  and
- closed enums for operators, invocation kinds, visibility, declaration kinds,
  imports, annotations, and literals.

Primitive versus boxed use is explicit and checked. No executable Java source
string enters the AST.

Raw UTF-16-unit and internal-null literals are privileged implementation nodes,
not general expression literals. Verification admits them only in an exact
registered runtime storage context whose inactive representation requires that
value; they cannot appear in generated public values, ordinary locals,
arguments, fields, or returns. Expression statements are limited to Java's
statement-expression grammar. Switch arms must have selector-compatible,
unique, non-dominated patterns and render the complete Java `case` grammar.
Lexical bindings from locals, parameters, foreach, catch, switch patterns, and
`instanceof` flow scopes must be unique throughout every overlapping Java
scope. Catch clauses are ordered and no later type may be a subtype of an
earlier catch. `instanceof` binder uniqueness is checked wherever the typed
expression may occur, not only when a surrounding control-flow node consumes
the binding.

Array ownership is part of `JavaType`. The only metadata-only change admitted
by the executable AST is the enum-valued `FreshCopyToBoundary` transition from
an internally allocated mutable array to a defensive-copy boundary. The
verifier rejects every other source/target pairing; rendering this proof node
emits its operand without a Java cast because ownership is a generator
invariant, not a Java runtime type.
Boundary verification is recursive through every generic, wildcard, array,
option, result, and list payload. An `InternalMutable` array at any depth is
therefore rejected in field, parameter, or result position; a shallow
container copy is not proof that nested arrays cannot alias caller state.
An ordinary Java cast preserves `JavaArrayOwnership` exactly at every array
layer. It cannot convert a boundary array to `InternalMutable`, recover an
owned array from `Object` or a type variable, or substitute for
`FreshCopyToBoundary`.

## 4. Type mapping

| CoreIR type | Java representation |
| --- | --- |
| Unit | generated singleton value/type |
| Bool | `boolean` or boxed only in required generic contexts |
| I32 / I64 | `int` / `long` plus exact checked helpers |
| F64 | `double` plus raw-bit helpers |
| Char | validated Unicode scalar stored as `int`, not UTF-16 `char` |
| String | validated scalar-safe `String` |
| Bytes | immutable generated value which defensively copies `byte[]` |
| List<T> | immutable `List<T>` boundary with `List.copyOf`/owned construction |
| Option<T> | generated tagged generic; never nullable/Optional approximation |
| Result<T,E> | generated tagged generic |
| Record | Java `record` or final value class chosen by typed strategy |
| Tagged enum | sealed tagged interface with final record/value variants |
| Interface | flat Java interface implemented by immutable values |

Generic boxing decisions are represented in `JavaType`, not inferred while
rendering. Registered callable signatures retain the complete constructed
`JavaType`, including every generic argument, wildcard bound, array component,
ownership marker, and type variable; a coarse "generic" category is not a
legal signature identity. Arrays and mutable collection references MUST NOT
escape portable value boundaries.
An explicit cast must be non-redundant and warning-free under
`javac -Xlint:all -Werror`. A parameterized cast whose target is not reifiable
is rejected rather than emitted as an unchecked conversion.
Wildcard bounds are reference types. Known member signatures distinguish
receiver, parameter, result, and nested type-argument positions: invocation
boxing is legal only where Java permits it, while the receiver and emitted
primary-expression result type remain exact. Wildcard capture has an explicit
upper-bound/Object result rule rather than weakening all result matching.

Public callable parameters and generated record components are normalized by
a `CoreTypeId`-directed lowering plan. Lists are rebuilt element-by-element and
sealed with `List.copyOf`; nested lists, options, and results recurse through
their exact checked payload types. Strings are scalar-validated at every
recursive position. This plan MUST NOT use erased casts, reflection, or a
renderer-side type test. Generated return values may reuse already-normalized
values, but every path by which an external value enters generated code MUST
execute the type-directed plan.

Generic runtime tagged-value constructors and untyped factories are not public
Java API: type erasure cannot soundly deep-copy an arbitrary `T`. The generated
entry class instead exposes one reserved, structurally named factory for each
concrete option/result type in the checked Core type arena. Those factories
normalize the exact payload `CoreTypeId` before invoking package-private
runtime construction. Names encode the full type shape without runtime numeric
IDs, so nested generic erasure cannot create overload collisions.
Tagged-value equality first compares the discriminant and then observes only
the active payload. Equality MUST NOT call a partial accessor for an inactive
option/result branch.

## 5. Declarations and control

- Constants are `static final` values; only Java constant expressions receive
  compile-time-constant treatment. Generated constant fields are emitted in a
  deterministic dependency-first topological order, not portable declaration
  name order. The target AST verifier independently rejects any generated
  field initializer which refers to itself or to a later field in the same
  type.
- Records/final classes validate and copy mutable inputs in typed constructors.
- Record components reject Java's reserved `Object` member names. An explicit
  canonical constructor is never less accessible than its record, and an
  explicit component accessor is public, concrete, non-static, non-generic,
  and returns the exact component type.
- Because the admitted Java AST has no static-initializer node, every `static
  final` field MUST have a typed initializer. A blank static final is rejected
  rather than relying on source Java which this AST cannot represent.
- Every blank instance-final field is assigned exactly once on every normally
  completing constructor path. An initialized final cannot be assigned again,
  and conditional, repeated, loop-dependent, or early-return omissions fail
  target AST verification before rendering. Reads of blank finals are checked
  against the same path state before a write is recorded.
- A local without an initializer is unreadable until assignment is proven on
  every normally completing path. Branch state joins use intersection and
  exclude alternatives which cannot complete normally.
- Every statement must be reachable from a normally completing predecessor.
  Return, throw, break, continue, exhaustive branch, exhaustive switch, and
  try/catch completion are determined structurally before rendering.
- Loop reachability follows the admitted Java constant-boolean grammar. A
  constant-false loop is rejected because even its empty body statement is
  unreachable, and a constant-true loop cannot complete normally unless a
  reachable `break` targets that exact loop. Breaks owned by nested loops or
  switches do not count. Constant forms outside the admitted grammar fail
  closed rather than being treated as dynamic.
- Field initializers have an explicit static or instance lexical scope. They
  cannot refer to locals, use `this` from static context, read a constructor-
  assigned blank final, or allow an unhandled checked exception.
- Tagged matches lower to exhaustive verified switches or explicit tag
  switches according to the selected Java 21 strategy.
- Explicit final temporaries preserve CoreIR receiver, operand, and argument
  order. Every nontrivial receiver/operand/argument is materialized exactly at
  its source evaluation point before a later child can execute; a composed
  Java expression cannot duplicate a source expression or let later fallible
  work overtake an earlier allocation or call.
- Portable `Evaluate` and ignored statement-body results are materialized as
  typed final local initializers. They never rely on Java's narrower
  expression-statement grammar, and their expression-plan prefixes still
  propagate portable failures before the result is discarded.
- Normal portable `Result` flow uses tagged values, not exceptions.
- Checked arithmetic, shift, float-bit, UTF-8/scalar, bytes, and collection
  behavior use known typed callables/helpers.
- Language equality and conformance equality are distinct typed operations.
  Recursive `semanticEquals` follows IEEE language behavior; recursive
  `deepEquals` compares F64 raw bits so expected-value tests distinguish signed
  zero and retain NaN payloads. Generated and runtime value types implement
  both methods explicitly.
- Empty-needle string replacement inserts only at Unicode scalar boundaries;
  it must not delegate to UTF-16 code-unit boundary behavior.
- Structural methods which collide with inherited `Object` signatures obey
  exact public instance override rules; final/reserved inherited methods fail
  closed. These checks apply to every method provenance, including registered
  interface, implementation, and callable methods. Portable `clone` is
  conservatively rejected because the generic AST cannot express a certified
  Java `Object.clone` implementation. A nested type cannot reuse the name of
  any enclosing type.

## 6. Interfaces, composition, and target heritage

Portable interfaces lower to flat sealed Java interfaces with no `extends`
clause. Their `permits` list is derived exactly from checked implementation
declarations, and immutable generated records/final classes explicitly
`implement` them. An interface with no generated implementation is an explicit
unsupported Java shape rather than an open extension point. Multiple
independent interface conformances are allowed. First-class interface values
use native Java interface dispatch, while generated APIs avoid `null`, reject
external implementations at Java compilation time, and do not expose object
identity as portable behavior.

Every implementing method is a public, concrete, non-static instance method
whose name, complete generic parameter types, and result type match the exact
registered interface declaration. The implementation-origin enum alone is not
proof of an override. Interface conformance and `@Override` validation use the
same exact predicate.

A generated interface's method set consists only of registered
`JavaMethodDeclaration::Interface` identities. Structural members are admitted
only in non-generated typed shells; conformance verification must reject, not
filter out, any unregistered method in a generated interface.

Composition uses final named fields and explicit delegation. Default methods,
interface-extension chains, abstract reusable base classes, and inherited state
are forbidden for portable implementation.

Shape preflight rejects generated static or interface method signatures that
would illegally hide, override, or conflict with inherited `java.lang.Object`
members after Java normalization and erasure. This applies even when the
portable source name is otherwise legal.

`JavaHeritage` may express the shared target-only one-edge adapter policy. Its
only certified class-extension form is a generated final leaf extending one
external framework base and delegating behavior to a composed component. The
verifier rejects generated bases, chains, inherited portable state, and reuse
hierarchies.

## 7. Symbols and imports

Closed catalogues include exact admitted types and members from:

- `java.lang` primitives/wrappers and exact numeric/string operations;
- `java.math.BigInteger` where a registered strategy needs it;
- `java.nio.ByteBuffer`;
- `java.nio.charset.CharacterCodingException`, `CodingErrorAction`, and
  `StandardCharsets`;
- `java.util.ArrayList`, `LinkedHashMap`, `List`, `Map`, and `Objects`; and
- generated declarations and runtime helpers.

Every callable records owner type, static/instance/constructor invocation,
generic substitutions, parameters, result, checked exceptions, nullability,
and purity/evaluation constraints. The resolver derives imports from typed
references, treats `java.lang` as implicit, handles same-package symbols, and
uses deterministic qualification for collisions. Lowerers never request
imports directly.

## 8. Runtime helpers

`JavaRuntimeHelper` is a closed enum whose nodes expand to typed Java
declarations and dependencies. Runtime option/result, exact arithmetic,
floating-point, Unicode, bytes, immutable collections, and interface support
are structural AST. There is no `RUNTIME` source constant and no verbatim
runtime body. Runtime value types and the accessors required by portable public
signatures may be public, but implementation helper callables are package
private. External code enters through generated, type-directed public APIs,
never directly through an unnormalized runtime helper.

Runtime helpers may contribute typed member fragments to the single typed
`Runtime` class shell. After helper linking, the Java plugin MUST recompose the
shell and every selected fragment into one `JavaTypeDeclaration` and rerun the
complete declaration verifier before rendering. Member-by-member verification
does not satisfy this rule. Cross-fragment names/erasures, constructor and
blank-final state, field ordering, lexical scope, checked exceptions, and
declaration-kind grammar are therefore checked in their final class context.
The shell is one canonical typed value: generated package, runtime source role,
the exact `Runtime.java` path, public final `Runtime` class, and its private
empty constructor. Runtime-member items cannot originate in a lowerer's source
file; they are admitted only from the linker-selected registered helper
closure, whose exact resolved item sequence is rederived during verification.
This identity is bidirectional: use of the canonical path, runtime role,
runtime placement, or a generated-package top-level type named `Runtime`
requires every other canonical identity component and the exact shell.

## 9. File and package policy

Public top-level type placement, one-public-type file rules, package-info,
generated/runtime/test roots, and deterministic dependency-safe member order
are represented by typed file roles. Split declarations, constant dependency
cycles, and circular/self field initialization are rejected. File paths are
derived from validated package/type identities, never supplied as executable
fragments. When a compilation unit contains one public top-level type, its
basename MUST equal that type's identifier plus `.java`.
`Main` public/implementation files and the exact runtime file live under
`src/main/java/org/polyrust/generated/`. Native, conformance, and negative-test
placements live under `src/test/java/org/polyrust/generated/`; each placement
has exactly its matching source role, and no additional path segment may be
supplied beneath the declared package directory.
Runtime member fragments are confined to the runtime file, which contains
exactly one typed class shell; both rules are rejected before rendering.

Method annotations are closed enum values with declaration-context checks.
Annotations are unique. `@Override` is admitted only for an instance method
with a verified generated-interface or runtime semantic-interface target.
`@SafeVarargs` is rejected while the Java AST has no typed varargs parameter
form.

## 10. Rendering

The Java post-link checker is the sole constructor of opaque
`RenderReadyPackage<JavaDialect>`. It validates complete Java 21 compilation
units after runtime-fragment composition and import/name resolution. Its closed
rules include lexical names and protected keywords; package/import/type order;
public-type filename identity; modifiers, annotations, members, heritage and
interface conformance; field and local definite assignment; reachability and
statement context; checked exceptions; generic, cast, array, wildcard, callable
and constructor legality; and precedence-bearing expression shape.

The total Java renderer directly and structurally covers compilation units,
packages, imports, annotations, heritage, declarations, members, types, blocks,
switches, statements, expressions, literals, comments, and tests. Exhaustive
Rust matches own every Java keyword, punctuation mark, delimiter, separator,
precedence parenthesis, indentation rule, and escape. It accepts only the
render-ready certificate and has no syntax-validation error path.

Java executable source uses no Handlebars template, serialized render view,
token/source escape hatch, third-party AST generator, or string-dispatched
grammar kind. The renderer MUST NOT select imports, runtime implementations,
features, symbols, or semantics.

Hermetic `javac --release 21 -Xlint:all -Werror` is the independent acceptance
oracle. A formatter, if added, is pinned and acts only as a no-diff check after
the unformatted output already compiles.

## 11. Validation

The verifier checks primitive/boxed contexts, nullability, known/generated call
signatures and checked exceptions, modifier combinations, interface
conformance, sealed exhaustiveness, immutable collection/array boundaries,
heritage depth and ownership, import namespaces/collisions, precedence,
definite return, and absence of opaque source. Resolved whole-file verification
rechecks every split or helper-injected declaration after linking and before a
render view can be built. It also verifies annotation legality and static-final
initialization rather than relying on `javac` to discover forged AST shapes.
A deterministic mutation corpus samples modifiers, annotations, nested and
bounded generic types, arrays, records, generic interfaces, nested types,
known calls, casts, branches, imports, and canonical file roots. Every
verifier-accepted sampled AST must link, render, and compile under hermetic
Java 21 with `-Xlint:all -Werror`. This is an executable sampling oracle, not a
claim that finite fuzzing proves the whole AST; category-specific negative
verifier tests and paired `javac` counterexamples remain mandatory.
The opaque-source policy scans all production Rust items even when a test-only
item appears earlier in the file; only the balanced `#[cfg(test)]` item itself
is excluded. The attribute marker is recognized lexically outside Rust string,
raw-string, character, line-comment, and nested block-comment contents, so a
marker-shaped decoy cannot hide later production source.

## 12. Success evidence

- AST, type-use, modifier, symbol-catalogue, and typed-call compile-fail tests.
- Exact import/helper matrices, including the prior runtime import set.
- Flat-interface, multiple-conformance, first-class/nested interface-value,
  dispatch, delegation, and rejected heritage-chain fixtures.
- Hermetic `javac --release 21` with all selected lint warnings treated as
  errors, plus native tests.
- A deterministic, reproducible Java AST mutation/compiler-oracle corpus which
  covers the named construct categories, sends every accepted sampled case
  through the real linker and renderer, and compiles the complete batch with
  the hermetic JDK. The evidence matrix pairs unsupported/invalid categories
  with exact verifier rejection and compiler-negative fixtures.
- Separately compiled public-consumer and deliberate negative type fixtures.
- Mutation/aliasing, `null`, Unicode, overflow, and F64 raw-bit boundaries.
- Hermetic positive and negative reachability fixtures for constant loops,
  dependency-ordered generated constants, and once-only left-to-right
  evaluation of allocations, receivers, operands, and arguments.
- Paired verifier/compiler counterexamples for unchecked and redundant casts,
  array-ownership forgery, inherited-`Object` collisions, and unregistered
  generated-interface methods, plus an executable array-aliasing witness.
- Three-generation determinism and every historical/canonical conformance
  vector.

Both generated native and conformance entry points execute the same typed AST
assertions for every portable test declaration. Each assertion identifies its
source test name, compares values with raw-bit-aware deep equality, compares
error payloads, and increments a completion counter checked against the exact
generated inventory. An empty placeholder `main` is not conformance evidence.

## 13. Migration exit

The Java plugin passes only after raw `JavaCode`, `RUNTIME`, hard-coded import
loops/strings, and manual dependency registration are deleted and every
executable compilation unit flows through verified Java AST, automatic symbol
resolution, render-ready certification, and total structural rendering.

The Java typed path additionally passes only when every used feature has an
executable compile-time mapping registered by the plugin builder, the
checked-in inferred example
exercises an arbitrary-arity function and record plus nested arithmetic, all
invalid typed examples fail Rust compilation, and the accepted output compiles
and executes under the hermetic Java 21 toolchain.

M34A-10U also exercises the complete PolyIR v0 intrinsic catalogue exposed by
M34A-08U: bitwise and shifts, float inspection, string transformations, bytes,
lists, option/result operations, numeric conversions, and UTF-8 conversions.
Every family has a separate registered Java mapping and native edge-case
evidence; interfaces remain the separate Layer 6 branded surface.

The checked-in authoring example is
`crates/backend-java/examples/generate_typed.rs`. Bazel materializes its
six-file package under the `generate_typed_package` target. The independent
consumer checks that `computed()` evaluates `(7 + 2) * (7 - 2) + 5` as `50`
and that `make_point(3, 4, 5)` returns a `Point3` with the exact three fields.
Generated operator operands are parenthesized, with final temporaries
preserving portable left-to-right evaluation order. The example therefore
proves that neither callable nor record shape is capped at arity two.
