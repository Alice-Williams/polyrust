# Layer 8: resolved render views and Handlebars

- Status: normative
- Input: verified `ResolvedPackage<D>`
- Output: `RenderedPackage`

## Ownership

Every independently lowered language plugin owns its renderer, render-view
types, `TemplateId` enum, templates, escaping rules, precedence table, and
formatting policy.

The shared layer owns only:

- strict Handlebars registry construction;
- template registration validation;
- deterministic diagnostic wrapping;
- common rendered-file size limits; and
- optional low-level document/line utilities.

JavaScript executable code is derived from TypeScript and has no independent
semantic renderer.

## Render views

A renderer first converts resolved AST nodes into private typed view structs.
Only renderer code can construct them.

```rust
struct MethodView {
    documentation: Vec<DocumentationView>,
    modifiers: Vec<RenderedModifier>,
    return_type: RenderedType,
    name: RenderedIdentifier,
    parameters: Vec<ParameterView>,
    body: BlockView,
}
```

`RenderedIdentifier`, `RenderedType`, `RenderedExpression`, and similar values
have private constructors which perform language escaping and precedence
handling. Arbitrary input strings cannot masquerade as rendered code.

## Template IDs

Each plugin defines a closed enum:

```rust
enum JavaTemplateId {
    SourceFile,
    PackageDeclaration,
    Import,
    Class,
    Interface,
    Method,
    Parameter,
    Block,
    ReturnStatement,
    StaticCall,
    InstanceCall,
    BinaryExpression,
}
```

An exhaustive Rust match maps every renderable node variant to one template ID.
Wildcard arms are forbidden. Internal Handlebars string names are private to
the template registry adapter.

## Template scope

Templates MAY express generic target grammar presentation:

- file and package layout;
- declaration shells;
- modifier lists;
- parameter lists;
- blocks and statements;
- expression punctuation;
- import/include directive spelling;
- documentation syntax; and
- metadata file layout.

Templates MUST NOT contain:

- a portable operation name or dispatch;
- a feature-specific implementation;
- a runtime-helper-specific body;
- a known external symbol decision;
- an unresolved symbol/helper ID;
- import selection logic;
- file-placement logic;
- ownership/cleanup decisions;
- target capability checks; or
- arbitrary executable snippets supplied by CoreIR/lowering.

Runtime helper AST uses the same generic templates as generated program AST.

## Handlebars registry

Certified generation uses a pinned `handlebars-rust` dependency and an embedded
template set.

The registry:

- enables strict mode;
- disables development auto-reload;
- disables script/Rhai helpers;
- registers no semantic helper;
- rejects missing and duplicate IDs;
- parses every template before generation;
- validates every referenced partial;
- uses deterministic registration order;
- disables HTML escaping and relies on typed language escape constructors; and
- exposes no user template override in certified mode.

Presentation helpers, if unavoidable, are closed enum-keyed Rust functions
limited to indentation, line joining, and whitespace. They cannot inspect
semantic or unresolved data.

## Expressions and precedence

The typed renderer owns a complete precedence/associativity enum for its emitted
expression subset. It computes parentheses before Handlebars invocation.

A template receives already classified child expression views and cannot infer
precedence from operator text.

Every operator spelling is selected from a dialect enum.

## Whitespace and formatting

Templates produce deterministic UTF-8 with LF newlines and one final newline
unless a metadata specification requires otherwise.

External target formatters remain verification tools and MAY be explicit
post-processes only where the language specification says so. A formatter:

- is pinned;
- must be deterministic;
- cannot repair imports or semantics; and
- is followed by a no-diff check.

Generated checked-in fixtures are not committed unless an explicit repository
policy says otherwise.

## Template security

Template contexts contain no filesystem handles, environment, process access,
network data, or arbitrary JSON from users. Only private serialized view
structs enter Handlebars.

Template output is subject to file size and total package size limits before
manifest construction.

## Diagnostics

Render errors include:

- target and template enum ID;
- source file role/path;
- closest AST provenance;
- missing field/partial name when applicable; and
- closed render error category.

Template source text is not included in normal diagnostics.

## Required proof

- Every `TemplateId` is registered exactly once and used.
- Every AST node variant has an exhaustive template selection arm.
- Strict-mode missing-field and missing-partial tests.
- Script/custom semantic helper rejection.
- Source-policy scan forbids portable operation/helper names in templates.
- Identifier, literal, comment, and documentation escaping matrices.
- Full precedence/associativity parenthesis matrices.
- Template output determinism.
- Formatter produces no semantic or import changes and second run is a no-op.
- Runtime and user declarations render through the same node templates.
- Fuzz/property cases cannot inject target syntax through identifiers/literals.
