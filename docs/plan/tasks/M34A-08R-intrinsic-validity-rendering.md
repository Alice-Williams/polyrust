# M34A-08R — Add intrinsic validity certificates and total rendering

- Status: complete
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

- Focused proof/policy gate invocation
  `b1285f2a-3a17-46bd-a612-af857eda27c2`: 6/6 targets passed.
- Expanded shared/Java/policy/lint invocation
  `ebd6223f-35e8-4030-b6ae-47829dd5ffbd`: 36/36 targets passed,
  including Rustfmt, Clippy, and Buildifier.
- Complete tracked-repository invocation
  `d2d829fb-f417-4cd7-aa7e-67d2e2ab9f2e`: 295/295 targets passed after
  the external-plugin proof and evidence documentation were added.
- Independent release-gate invocation
  `143b2446-1470-4a29-8ca4-d9831ff207a1`: 233/233 targets passed after
  that final tracked replay.
- External typed-plugin invocation
  `ded80c9b-2673-4269-bc6b-01c23cf90e9b`: 5/5 external adapter,
  typed-source policy, Rustfmt, Clippy, and Buildifier targets passed.
- A fresh isolated `cargo test --workspace --all-features --locked` passed,
  including all 31 codegen compile-fail doctests.
- Implementation checkpoint `48fecbfaecf67b089085f2ea28203d562ca5fe68`
  was pushed and verified byte-for-byte on `origin/main`. Hosted CI run
  `33790836392` passed all eight jobs for that exact SHA, including the
  cache-cold and cache-warm complete release gates.
