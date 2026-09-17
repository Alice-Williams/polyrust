# M35-03A-02G-01 — Certified C/Java void result foundation

- Status: complete
- Parent: [unit function results](M35-03A-02G-unit-results.md)
- Specifications: [C](../../specification/typed-generation/languages/c/rust-unit-results.md),
  [Java](../../specification/typed-generation/languages/java/rust-unit-results.md)

## Contract

Make the existing target AST/certificate/dependency path support ordinary void
function results without making void an object/value type. Compiler source
admission and public bundle schemas remain unchanged in this checkpoint.

## Implementation

1. Introduce closed CPrimitiveType::Scalar(CScalarType) / Void for the shared
   graph, leaving CObjectType/CScalarType unchanged. Migrate existing scalar
   projections mechanically and map void only from CReturnType::Void.
2. Extend bounded C profile and CDependencyApi signature admission for scalar
   parameters and scalar/void returns. Admit bare return and typed direct void
   call statements, visiting all arguments/targets and preserving call costs.
3. Extend Java source/dependency body and result admission analogously, reusing
   JavaType::primitive(JavaPrimitive::Void), Return(None) and direct invocation statements. Do not widen
   ordinary value/storage/constant types or trust the signature's pure flag.
4. Reconstruct callable signatures and dependencies exactly from original
   certificates. Check missing/recertified/wrong-result/consumer-scoped authorities.
5. Add focused typed producer/consumer fixtures and native/compile-negative proof.
   Keep new fixtures and test modules separate from production modules.

## Definition of done and tests

- Void producers and mixed scalar/void packages certify; scalar-only output
  and existing proofs remain unchanged.
- Original-authority direct/transitive void consumers compile and execute with
  GCC/Zig O0/O2 and Java 21 strict lint; public C headers compile independently.
  Reflection/typed evidence proves genuine void and no extra unit storage.
- Scalar-to-void and void-to-scalar body/signature mutations fail; void cannot
  become a parameter, local, field, constant or value-call result via this path.
  Wrong arguments/owners, stale signatures and forged handles reject.
- Real calls still contribute dependency/resource/call-height evidence.
  Artifact guards prove no runtime/wrapper/helper class or unit record.
- Fresh independent Sol Extra High review has no core finding. Full exact-tree
  Linux Bazel release, Rust/Bazel lint and native gates pass before commit/push.
- Rust-source adapters continue rejecting unit signatures until child 02G-02
  supplies typed compiler mappings and versioned publication proof.

## Completion evidence

- Exact isolated tree 34797b7c052be9e4043a0aed77d528f1d45c2ba4 passed Linux
  dev-container Bazel //... //:release_gate: 1,111 targets, **742/742 tests**,
  16 executed / 726 cached, 449.047 seconds. Invocation:
  49c73c9c-4234-4332-8dc7-f0fdcbe757d8.
- C unit suite: **772 passed**, including separately compiled GCC/Zig O0/O2
  void owner chains, standalone public headers, typed void function pointers and
  native void-as-value rejection. Capacity tests also passed in the full gate.
- Java unit suite: **350 passed**, including Java 21 strict compilation,
  separate owner/client compilation and reflection proving primitive void.
- Typed negatives cover invalid value/void returns and storage, original versus
  recertified owners, argument/result mismatches and unregistered callables.
  Retained effect calls increase stack/call-height evidence and occur exactly
  once in generated source; no runtime or unit storage appears.
- Review findings about C closed-body proof/prototype traversal and Java void
  expression context/byte reservation were accepted and fixed. Fresh independent
  Sol Extra High review of the exact tree above found **no findings** across the
  production delta and proof suites.
- Compiler adapters and publication schemas are intentionally unchanged. This
  checkpoint does not yet admit Rust unit signatures or complete runtime parity.
- The final documentation-only completion tree is release-gated before its own
  scoped commit/push. All 19 separately preserved ownership files retain their
  original hashes; the previously clean allocation_calls.rs change is intentional
  and included in this target-void checkpoint.
