# M34A-08R — Add intrinsic validity certificates and total rendering

- Status: planned
- Depends on: M34A-09
- Supersedes: the executable-source portion of M34A-08
- Blocks: M34A-10V and every remaining language migration

## Goal

Implement ADR-0005's shared proof-carrying phase boundary so safe code can
render executable source only from an opaque, language-certified package.

## Definition of done

- `UnresolvedPackage<D>`, `VerifiedPackage<D>`, `LinkedPackage<D>`, and
  `RenderReadyPackage<D>` are distinct ownership-consuming states.
- Only target verification constructs `VerifiedPackage<D>` and only the
  mandatory post-link language certifier constructs `RenderReadyPackage<D>`.
- Proof wrapper fields and constructors are private, wrappers are not
  deserializable, and no safe mutable access to enclosed AST exists.
- The linker accepts only `VerifiedPackage<D>` and the executable renderer
  accepts only `RenderReadyPackage<D>`.
- A sealed `TotalTargetRenderer<D>` returns deterministic rendered artifacts
  without a grammar-validation error path.
- The compiler adapter performs lower, verify, link, certify, total render, and
  manifest assembly in that exact order with atomic failure.
- The old Handlebars adapter remains available only to explicitly non-compliant
  not-yet-migrated language work; it is not reachable from the certified
  `LanguagePlugin` path and is deleted by M34A-18.
- Shared source policy forbids executable templates, raw/token/source escape
  nodes, wildcard grammar matches, and string-dispatched node kinds in a
  migrated renderer.

## Tests

- Rust compile-fail cases for unresolved-to-link, unresolved-to-render,
  verified-to-render, linked-to-render, certificate construction, field access,
  mutation, and deserialization attempts.
- Pipeline trace/fault injection at target verification, linking,
  certification, total rendering resource checks, and manifest assembly.
- A minimal external plugin proves the same sealed adapter sequence.
- Three-run certificate evidence and rendered-byte determinism.
- `bazel test //crates/codegen:all --nocache_test_results --test_output=errors`
- `bazel test //tools/policy:all --nocache_test_results --test_output=errors`

## Commit gate

Commit and push `M34A-08R: add intrinsic validity certificates` only after the
focused tests, documentation checks, Rustfmt, Clippy, and Buildifier pass in the
Linux development container.

## Exit evidence

To be filled with exact targets, counts, commit SHA, remote SHA, and hosted CI
run after implementation.
