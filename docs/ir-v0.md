# PolyRust IR v0 canonical schema

Status: normative for IR version `0.1.0`

This document defines the unchecked, target-independent v0 program model and its
canonical `.poly.json` encoding. Name resolution, type correctness, contract
conformance, exhaustiveness, and capability computation are deliberately deferred
to the checker.

## Document envelope

Every document is a UTF-8 JSON object with these fields in this order:

```json
{"ir_version":"0.1.0","module":{"name":"example","declarations":[]},"metadata":{}}
```

| Field | Meaning |
| --- | --- |
| `ir_version` | Exact `major.minor.patch` semantic-version text |
| `module` | One portable namespace and its declaration set |
| `metadata` | Non-semantic producer metadata; keys are sorted lexicographically |

Readers reject malformed versions, unknown fields, unknown enum variants, and an
unsupported major version. The v0 reader accepts any `0.x.y` version that uses a
schema it can decode; a nonzero/different major is incompatible. Minor-version
preservation behavior may be tightened before 1.0.

## Common encoding rules

- Struct fields serialize in their Rust schema order.
- Sum types use adjacent tagging: `{"kind":"snake_case_variant","data":...}`.
  Fieldless variants omit `data`.
- No optional unknown fields are silently discarded; every schema struct and sum
  rejects unknown content.
- Canonical output is compact UTF-8 JSON with no trailing newline or insignificant
  whitespace.
- Top-level declarations are a semantic set and serialize in ascending `NodeId`
  order. Other vectors preserve specified source order. Maps use lexicographic key
  order.
- Timestamps, absolute producer paths, random IDs, target syntax, and hash-map
  iteration order are forbidden as canonical inputs.
- IEEE-754 `F64` values encode their exact unsigned 64-bit representation. This
  preserves every NaN payload, infinities, and negative zero without relying on
  JSON number behavior.

The exhaustive canonical fixture is
`crates/ir/src/v0/testdata/every-node.poly.json`.

## Node identity and source

Every declaration, member, expression, statement, block, match arm, pattern, and
constant expression carries:

```text
NodeMeta {
  id: NodeId,
  source: SourceRef
}
```

`NodeId` is a nonzero unsigned 64-bit identity unique within one document.
References use IDs rather than target names. The structural validation pass
rejects zero and duplicates before checking.

`SourceRef` is either:

- `file`: a logical file label and half-open UTF-8 byte range; or
- `logical`: ordered builder/frontend path segments.

Canonical documents must not embed absolute machine-local paths.

## Declarations

The module declaration sum is closed for v0:

| Kind | Required content |
| --- | --- |
| `constant` | header, explicit type, restricted constant expression |
| `alias` | header and non-recursive target type |
| `record` | header and immutable typed fields |
| `enum` | header and closed unit/record-shaped variants |
| `contract` | header and immutable instance-method signatures |
| `implementation` | header, contract ID, record ID, and method bodies |
| `function` | header, explicit parameters/return type, and pure block |
| `test` | header, typed invocation, and typed value/error expectation |

Declaration headers contain `NodeMeta`, a portable identifier, `public` or
`package` visibility, and non-semantic documentation paragraphs. Fields,
variants, parameters, and methods use member headers with the same identity,
source, name, and documentation model.

## Types and values

Required type variants are `unit`, `bool`, `i32`, `i64`, `f64`, `char`, `string`,
`bytes`, `list`, `option`, `result`, `named`, and restricted `contract`.
`named` references a record, enum, or alias declaration. `contract` may be used
only in parameter positions; the checker enforces that restriction.

Portable values cover all scalar types, immutable bytes/lists, `none`/`some`,
`ok`/`err`, records, and enum variants. Aggregate fields are keyed by declaration
IDs. Portable test arguments and expected outcomes pair every value with an
explicit `TypeRef`.

## Expressions and statements

Unchecked expressions include literals, locals, constants, immutable `self`,
record/enum/list/option/result construction, field access, function calls,
nominal concrete or contract method calls, explicit intrinsics, `if`, exhaustive
`match`, and expression blocks.

Statements include immutable `let`, bounded ordered `for_each`, explicit
`return`, and expression statements. Patterns cover booleans, tagged variants,
`Option`, `Result`, and wildcard. The checker later proves type compatibility,
return coverage, exhaustiveness, and the absence of recursion.

Intrinsic names specify semantics instead of target punctuation. The closed v0
set covers:

- short-circuit Boolean operations and structural/IEEE comparisons;
- checked and wrapping signed integer arithmetic, bitwise operations, and checked
  shifts;
- IEEE binary64 arithmetic and truncating remainder;
- Unicode-scalar string operations;
- immutable bytes/list operations;
- option/result queries;
- explicit integer and UTF-8 conversions.

There are no `RustType`, `GoType`, `PythonType`, `TypeScriptType`, raw source, or
target import variants.

## Constant expressions

Constants have a deliberately smaller syntax than function bodies: literals,
references, record/enum/list/option/result construction, and an explicit
intrinsic. The checker proves reference acyclicity and permits only intrinsics
marked `const_safe`.

## Canonical API and limits

`portable_ir::v0::to_canonical_json` validates structure, sorts declarations,
and emits canonical bytes. `from_json` decodes with these defaults:

| Resource | Default limit |
| --- | ---: |
| Total input | 8 MiB |
| JSON depth | 128 |
| Structural JSON values | 1,000,000 |
| One UTF-8 string/key | 1 MiB |

`from_json_with_limits` accepts stricter caller-provided limits. Failures return
`JsonError` with a stable category: byte, depth, node, or string limit; invalid
JSON; unknown field/variant; unsupported version; or invalid structure. M03 maps
these categories to stable user-facing diagnostic codes.

The reader uses Serde JSON's bounded recursive decoder, applies configured limits
before exposing a document, and never interprets input as target code. Randomized
malformed byte tests are required to remain panic-free.

## Compatibility rule

Changing a field, tag, canonical order, value representation, or interpretation
can be a semantic compatibility break even before 1.0. Any such change requires:

1. an `IrVersion` decision;
2. updated normative documentation and golden bytes;
3. parse/serialize/parse and insertion-order determinism tests; and
4. designs and conformance impact for all required target backends.
