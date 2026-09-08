# Java 21 compiler admission budget

- Status: implementation contract, M34A-10AA
- Scope: the certified Java subset, not arbitrary Java source

This is a conservative admission policy for the pinned Java 21 compiler, not
an exact class-file size prediction or a theorem about all Java compilers.
The initial audit uses Zulu 21.0.9+10-LTS, source revision `98fc10faac11`,
including `jdk.compiler/com/sun/tools/javac/jvm/Gen.java` and its switch
selection rule. Compiler upgrades require replaying and reviewing the native
class-file accounting tests. No compiler is invoked by production generation.

## Contribution rules

All counters saturate. Children are added, not deduplicated; this deliberately
overcounts constants, mutually exclusive branches, reused locals and stack
lifetimes. A class's explicit and compiler-synthesized methods are checked
individually. Separate classes do not share a method or constant-pool budget.

| Construct | Code bytes reserved | Constant-pool slots reserved | Other accounting |
| --- | ---: | ---: | --- |
| Method entry/exit | 64 | 32 | Receiver/parameter slots included in locals |
| Literal or value load | 8 | 16 plus type tree | `ldc_w`/`getstatic` use three bytes; wide local loads four, with conversion margin |
| Other expression node | 32 | 16 plus type tree | Eight stack slots; child contributions added |
| Type tree node | 0 | 8 | Includes erased/generic/annotation/debug references |
| Invocation/construction argument | 16 | 8 | Boxing, casts, varargs array fill, two stack slots |
| Simple statement | 32 | 16 | Stores/conversions, invocation disposal, return/throw or assertion construction |
| Break/continue | 8 | 16 | Wide branch is five bytes |
| Control-flow statement | 64 | 16 | Eight temporary/local and stack slots |
| Switch arm | 64 | 16 plus pattern type | String equality/hash, labels, casts, branches |
| Catch | 64 | 16 plus exception type | One exception-table entry, two local slots |
| Field/component/member metadata | 0 | 32 plus type tree | Names, descriptors, signatures, attributes |
| Class baseline | 0 | 512 | Standard attributes, known platform support |
| Package-declared type metadata | 0 | 16 per type per class | Conservative nested/inner/nest reference closure |

Expression cases are exhaustive: literals and loads, unary/binary operators,
conditionals, calls, constructors, arrays/indexing, fields, casts, interface
coercions, ownership markers and `instanceof`. Literal contents do not add code
per character: their separate encoded-byte check already bounds the constant.
Long/double constants take two pool slots and fit the literal reservation.
String `+` is not admitted. Calls reserve argument overhead even when no boxing
or varargs conversion is needed. Ownership markers render their child only.
Lambdas are rejected by certification; they must not silently acquire a budget.

Statements cover locals, assignments, expression/return/throw, assertions,
branches, both loop forms, switches, catches, break and continue. Wide loads,
wide branches, comparison materialization and alignment fit their reservations.
For integer switches, javac's table/lookup cost rule bounds table range by
`5 * label_count - 10`; 64 bytes per arm covers either encoding, including
sparse integer keys. String switches add a hash/equality dispatch and fit the
same per-arm reservation. Pattern switches additionally reserve one bootstrap
entry and one argument per arm. Stack-map entries cannot exceed code bytes.

## Compiler synthesis

Instance field initializers are added to every explicit constructor and the
reserved default/canonical constructor. Static initializers share one checked
`<clinit>`. Non-static inner classes reserve an enclosing-instance field and
constructor assignment. Record reservations include fields, accessors,
canonical construction, three Object methods and their bootstrap metadata.
Enum reservations include constant fields, `$VALUES`, constructor, `values`,
`valueOf`, `$values` and `<clinit>`, with per-constant initialization/array fill.
Conservative duplicate reservations for explicitly supplied members are allowed.

Enum switches also contribute to a separate synthetic helper-class budget for
the complete enclosing nest. Conservatively, every switch reserves a possible
map, including default-only enum switches: labels alone do not establish the
selector's kind. Each switch reserves a map field; each arm reserves
initialization code, constants and a catch entry. The helper's combined
initializer is checked, rather than treating each switch as an isolated method.
Generated interface conformance has exact signatures and no generic heritage;
covariant/generic bridges are not admitted. This invariant and lambda rejection
must stay covered when the AST grows.

## Limits and evidence

Check code bytes, pool slots (including the reserved zero index), fields,
methods, locals, stack, exception entries, bootstrap entries/arguments and nest
metadata separately against their class-file encodings. Diagnostics explicitly
say `conservative ... budget`; rejection does not claim javac would reject the
same source. Exact slot/name/descriptor checks remain separate.

Native tests must reproduce oversized byte-array method and aggregate member
counterexamples, pair them with accepted smaller programs, and inspect actual
class files for admitted fixtures. Actual code/pool/locals/stack/member and
bootstrap counts must fit the calculated reservations. Existing eight-target
conformance and historical Java ports remain required. Until those checks and
the independent review pass, this policy remains implementation work, not
completed proof. Packing, extraction and class splitting are optional future
ways to accept programs rejected by this conservative policy.
