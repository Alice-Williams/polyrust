# ADR-0001: One portable semantic model, not pairwise translation

- Status: accepted
- Milestone: M00
- Date: 2026-08-31

## Context

Pairwise translation grows quadratically and inherits incompatible semantics
from every source language. The product instead needs one program to generate
several target-language implementations that can be tested together.

## Decision

- The source of truth is a versioned, language-neutral program model.
- The initial authoring frontend is a verbose Rust builder.
- Future macros or a restricted Rust parser lower into the same unchecked IR.
- Existing Rust, Go, TypeScript, and Python programs are not general inputs.
- A feature is portable only after its semantics and required target lowerings
  are specified and tested.

## Alternatives considered

- Any-to-any transpilation: rejected because semantic and testing cost grows
  with every source/target pair.
- Rust AST as the permanent IR: rejected because Rust ownership, traits, macros,
  and libraries do not have automatic equivalents in other targets.
- Text templates: rejected because they cannot provide target-independent type
  checking or semantic conformance.

## Consequences

The supported language begins deliberately small. Authors gain one testable
definition and deterministic targets, but cannot use arbitrary host-language
features without extending the portable model.

## Enforcement

M02 defines target-neutral IR, M04 rejects unsupported constructs, M05 supplies
the semantic oracle, and M10–M14 compile and compare generated native tests.
