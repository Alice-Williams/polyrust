# Layer 8: intrinsic validity certificates and total rendering

- Status: normative
- Input: `LinkedPackage<D>`
- Output: `RenderReadyPackage<D>` and then `RenderedPackage`

## Purpose

This layer establishes a type-level rendering capability: safe code can call an
executable source renderer only after the language checker has certified the
complete linked package. The guarantee is syntactic validity for the language
version and supported subset named by the plugin specification. Functional
equivalence is a separate proof.

## Proof-carrying boundary

Every independent target has a language-owned checker implementing the shared
certification boundary:

```rust
trait TargetCertifier<D: TargetDialect> {
    fn certify(
        &self,
        package: LinkedPackage<D>,
    ) -> Result<RenderReadyPackage<D>, Vec<Diagnostic>>;
}

trait TotalTargetRenderer<D: TargetDialect> {
    fn render(&self, package: &RenderReadyPackage<D>) -> RenderedPackage;
}
```

The concrete API MAY consume the render-ready package instead of borrowing it,
but it MUST preserve these properties:

- `RenderReadyPackage<D>` has private fields and no public constructor;
- only shared certification code can construct it after the language checker
  succeeds;
- it cannot be deserialized or safely mutated;
- it contains or privately owns the exact linked value which was checked;
- no renderer overload accepts an unresolved, verified, or merely linked
  package; and
- rendering has no grammar-validation error path.

A target checker MUST validate every invariant required by the supported
grammar, including whole-file/contextual restrictions that are not expressible
by a local node enum. Dynamic input-defined names and declaration graphs make a
runtime checking pass necessary; the opaque result turns that pass into a
compile-time capability for all subsequent code.

## Total structural renderer

Each plugin owns a direct Rust formatter over its closed language AST. The
formatter:

- exhaustively matches every grammar enum without wildcard fallback;
- emits all fixed keywords, punctuation, separators, and delimiters itself;
- renders identifiers and literals only through validated typed values;
- applies precedence and associativity from closed operator enums;
- renders imports/includes only from linked typed references;
- emits deterministic UTF-8 with LF newlines and the specified final newline;
- contains no semantic lowering, feature selection, helper selection, name
  resolution, validation, parsing, filesystem access, or process execution; and
- cannot return a syntax error for a certified input.

Small internal document/indent writers MAY be used. They are byte sinks, not
token streams or alternate ASTs: their API cannot inject an executable node,
skip the language checker, or accept user-provided source.

## Executable-template prohibition

Executable source MUST NOT be produced by Handlebars or another template
engine. A template remains a separate program containing unchecked grammar;
strict missing-field behavior does not certify its punctuation or context.

Templates MAY render non-executable metadata or prose when:

- the artifact role cannot be compiled or interpreted as target source;
- the template cannot concatenate, wrap, or alter executable bytes; and
- strict registration, escaping, determinism, and size rules still apply.

No third-party target AST or code-generation library is inside PolyRust's
certified source path. Native compilers, parsers, formatters, and linters are
test oracles only.

## Syntax guarantee and limits

For each dialect `D`, PolyRust claims:

```text
value has type RenderReadyPackage<D>
  => total renderer produces one deterministic package
  => every source file is syntactically valid for D's declared language level
```

The claim relies on the soundness of the language checker and structural
renderer implementation. It does not claim that target type checking succeeds
unless the language specification includes those rules in certification, that
the output is functionally equivalent, or that an external compiler has no
bugs.

Native compiler/parser testing independently checks this implication. A
formatter MUST NOT be used to turn rejected source into accepted source. If a
formatter is part of canonical output, the unformatted bytes must already parse
and a second formatter run must be a no-op.

## Diagnostics

Certification failures include the target, closed violation category, source
file role/path, closest AST provenance, and stable relevant symbol IDs. They do
not produce a render-ready value.

Rendering a certified package is infallible with respect to grammar. Resource
limits and manifest/path validation remain typed failures in their owning later
phase rather than syntax diagnostics.

## Required proof

- Compile-fail cases for every earlier-package-to-renderer boundary.
- External-consumer compile-fail cases for constructor, field, mutation, and
  deserialization attempts against `RenderReadyPackage<D>`.
- One positive and one negative checker case for every target AST variant and
  contextual restriction in the supported subset.
- Source policy forbidding raw/token/source escape nodes, executable templates,
  wildcard renderer matches, and string-dispatched grammar kinds.
- Exhaustive identifier, literal, comment, documentation, precedence, and
  associativity matrices.
- Deterministic structured mutations spanning every grammar category. Every
  accepted mutation must render and pass the pinned native compiler/parser;
  representative native-negative fixtures must fail certification.
- Runtime and user declarations pass through the identical checker and
  structural renderer.
- Three repeated certifications/renders produce identical evidence and bytes.
- Renderer unit tests require no filesystem, process, network, environment, or
  third-party code-generation dependency.
