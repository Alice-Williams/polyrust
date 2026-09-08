# Java target-resource boundary

- Status: normative
- Implementation: M34A-10AA, in progress

## Contract

Portable typing proves capability availability and well-formed portable
operations. It does not prove that a finite JVM class file can contain an
arbitrarily large program. The generic recursive lists remain uncapped.

Java's typed generation entry point returns
`Result<OutputManifest, JavaResourceError>`. The error has private construction
and exposes structured diagnostics with the stable `TargetResourceLimit` code.
Only capacity diagnostics may inhabit this result. A syntax, type, capability,
linking, or certificate failure for a typed portable input remains an internal
implementation defect; it must never be disguised as a capacity error.

The dynamic checked-program entry point retains its existing diagnostic path.
Both entry points use the same Java resource checks, before source is emitted.
No native compiler process or third-party dependency enters production code.

## Ownership

Resource checks inspect the language-owned AST, after target name allocation.
They run in `TargetResourceValidation` after syntax certification over linked
files, including the fully composed Runtime class. Invalid target ASTs fail
first, so capacity errors cannot mask an accompanying syntax/mapping defect.
The checks must not sum independent classes as one class or miss members
injected as helper fragments. No renderer performs
resource checking, chunking, name rewriting, or ABI transformation.
Shared rendered-file/package size limits remain a later `Rendering`-stage
resource failure, using the same stable resource code, rather than being
misclassified as grammar errors. Only these two phases may create the typed
Java resource result.

## Exact structural limits

The [Java 21 JVM specification](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)
defines the relevant class-file encodings in sections 4.1, 4.3.3, 4.4.7, and
4.7.3. Required checks include:

- Method descriptors consume at most 255 parameter slots. Primitive `long`
  and `double` consume two; other admitted parameter types consume one. An
  instance method or constructor additionally consumes the receiver slot.
  Record canonical constructors and enum constructor synthetic parameters
  must be accounted for even when absent from the source member list.
  Non-static nested ordinary classes additionally capture an enclosing
  instance, including implicit default constructors; static nested classes,
  records, enums and interfaces do not.
- Encoded names and signatures respect the unsigned-16-bit modified-UTF-8
  limit. Account for package, enclosing-class separators, generated suffixes,
  erased descriptors and generic signatures, rather than only simple names.
  Generated-member binary names include the package prefix. A combined class
  Signature is checked only when javac actually emits that attribute; raw
  non-generic interface names remain separate constant-pool entries.
  The walk covers executable-local types/names and reference signatures, not
  only declaration headers. Instantiated varargs lists are not JVM descriptors.
  Implicit enum `valueOf(String)` descriptors reserve 22 bytes beyond the
  enum's binary name; record `equals` ObjectMethods call-site descriptors
  reserve 23. These dominate the other implicit owner-bearing descriptors.
  Record component-name recipes accept 65,535 encoded bytes: unlike source
  literals, pinned javac does not impose the stricter 65,534-byte threshold.
  Source-level type names must also remain unique after JVM nesting separators
  are applied: a top-level `Outer$Inner` cannot coexist with `Outer.Inner`.
  This is an AST validity check, not a resource error or a ban on dollar names.
- Array descriptors have at most 255 dimensions.
- Class member counts include language/compiler-synthesized members, not
  just explicit AST members. Fields, methods, interfaces and constant-pool
  indices have separate budgets.

The pinned compiler's stricter string-literal boundary is 65,534 encoded bytes.
Text mappings split scalar-safe chunks and use typed non-constant `concat`
calls; they do not reject an otherwise representable large portable string.
The admitted Java AST uses numeric `Add` only: every string concatenation,
including runtime construction, uses the typed `String.concat` member. This
prevents folding through literals, final locals or constant fields without a
second constant evaluator. Switch-label literals use the same scalar/encoding
payload checks as expression literals, in addition to selector compatibility.

## Compiler-dependent capacity

Method code size and constant-pool occupancy depend on compiler lowering,
including record/enum support, bridges, lambdas, field initialization, switches,
boxing and varargs arrays. A raw AST-node count is not an exact bytecode proof.
Any conservative admission budget must have an explicit coverage argument for
every admitted construct and compiler-generated contribution, use saturating
accounting, identify itself as a conservative budget in diagnostics, and retain
native positive/negative boundary tests. Do not call an unproved heuristic an
exact JVM capacity guarantee. The implemented reservations and native class-file
evidence follow the [compiler admission budget](compiler-budget.md). M34A-10AA
remains open until the full checkpoint and independent review pass.

Argument packing, class splitting and helper-method extraction are separate
optional representational extensions, not prerequisites to supporting ordinary
functions, records or interfaces. None is silently introduced by this cleanup.

## Proof

Pair each exact rejection with a near-boundary accepted AST and a Java 21
compiler control. Test primitive versus boxed slots, static versus instance,
record/enum synthesis, nested binary names, descriptor aggregation, arrays,
and post-link runtime composition. Exercise the typed API with resource errors
and preserve its Rust compile-fail capability/unchecked-input tests. Existing
real-world, native-consumer, strict-lint, snapshot and determinism gates remain
mandatory. Success evidence is scoped to the implemented checks; it is not a
formal proof about arbitrary future compiler versions. The encoding oracle
uses an in-memory Java file manager, so filesystem filename limits do not mask
class-file UTF8 boundaries. It also loads the emitted classes: javac alone can
accept an enum constructor whose synthetic parameters violate JVM slot limits.
