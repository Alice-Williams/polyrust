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
- `JavaCompilationUnit`, `JavaPackage`, and `JavaTemplateId`; and
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

Array ownership is part of `JavaType`. The only metadata-only change admitted
by the executable AST is the enum-valued `FreshCopyToBoundary` transition from
an internally allocated mutable array to a defensive-copy boundary. The
verifier rejects every other source/target pairing; rendering this proof node
emits its operand without a Java cast because ownership is a generator
invariant, not a Java runtime type.

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
rendering. Arrays and mutable collection references MUST NOT escape portable
value boundaries.

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
  compile-time-constant treatment.
- Records/final classes validate and copy mutable inputs in typed constructors.
- Tagged matches lower to exhaustive verified switches or explicit tag
  switches according to the selected Java 21 strategy.
- Explicit temporaries preserve CoreIR receiver and argument order.
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
runtime body.

## 9. File and package policy

Public top-level type placement, one-public-type file rules, package-info,
generated/runtime/test roots, and deterministic member order are represented by
typed file roles. Split declarations and circular initialization are rejected.
File paths are derived from validated package/type identities, never supplied
as executable fragments.

## 10. Rendering

`JavaTemplateId` covers compilation units, packages, imports, annotations,
heritage, declarations, members, types, blocks, switches, statements,
expressions, literals, comments, and tests. Strict embedded Handlebars receives
resolved Java render views only. Templates MUST NOT contain imports, runtime
implementations, feature selection, or semantic branching.

The selected formatter, if used, is pinned and acts as a no-diff
verification/post-process rather than a semantic repair.

## 11. Validation

The verifier checks primitive/boxed contexts, nullability, known/generated call
signatures and checked exceptions, modifier combinations, interface
conformance, sealed exhaustiveness, immutable collection/array boundaries,
heritage depth and ownership, import namespaces/collisions, precedence,
definite return, and absence of opaque source.

## 12. Success evidence

- AST, type-use, modifier, symbol-catalogue, and typed-call compile-fail tests.
- Exact import/helper matrices, including the prior runtime import set.
- Flat-interface, multiple-conformance, first-class/nested interface-value,
  dispatch, delegation, and rejected heritage-chain fixtures.
- Hermetic `javac --release 21` with all selected lint warnings treated as
  errors, plus native tests.
- Separately compiled public-consumer and deliberate negative type fixtures.
- Mutation/aliasing, `null`, Unicode, overflow, and F64 raw-bit boundaries.
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
resolution, and strict templates.
