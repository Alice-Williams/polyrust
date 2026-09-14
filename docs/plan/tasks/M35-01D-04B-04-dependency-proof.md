# M35-01D-04B-04 — Composed C dependency proof

- Status: complete
- Depends on: [M35-01D-04B-03](M35-01D-04B-03-dependency-linking.md)
- Parent: [M35-01D-04B](M35-01D-04B-certified-c-dependencies.md)

## Definition of done

Compose the existing scalar-call and live-stack analyses with exact dependency
certificate evidence, then enable complete certification of consumer packages.
Preserve all contextual/numeric/ownership checks and per-file/package source
budgets. Render separate owning packages without copied dependency bodies.

## Ordered implementation

1. Add a bounded iterative certificate-closure verifier. Deduplicate exact
   authorities; reject mixed certificates for one source crate, transitive
   consumer/owner identity overlap, and full public symbol/header collisions.
   Validate unused registrations too, without inventing includes or live calls.
2. Reconstruct each retained certificate's cached stack bound from its original
   typed package. Check every child independently, including leaves, without
   recursive re-certification or trusting caller-provided summaries.
3. Extend the existing weighted call graph with authenticated external costs
   for actual calls only. Keep returned per-function frames owned-only; preserve
   the older whole-unit no-call policy. Reuse checked longest-path arithmetic.
4. Reuse registered scalar-effect derivation for chained dependency API exports
   and registered profile traversal for resource measurement. Remove the
   all-import certification barrier only together with closure and cost checks.
5. Prove separate output/compilation, exact public/imported symbols, native frame
   bounds and controlled-stack execution, then run full gates and fresh review.

## Specific regression matrix

- Independent dependencies, a three-crate chain and a shared-certificate diamond.
- Unused imports add neither live call cost nor includes but still authenticate.
- Mixed versions across branches, transitive owner reuse by a consumer, complete
  export/include conflicts, and missing or altered cached cost evidence reject.
- Actual consumer frame plus dependency bound is charged; returned frame maps
  never contain foreign definitions. Check exact limit/one-over and overflow.
- Iterative traversal admits at most 1024 distinct dependency certificates and
  100000 examined import edges, with explicit checked-budget tests.
- Extend the full-export collision fixture with an unprefixed consumer requested
  name to prove checking occurs after actual C identifier allocation.

## Tests and proof

- Actual call-graph boundary/one-over, missing/understated dependency costs,
  overflow, cycles and altered effect evidence reject.
- Compile/link exact owning packages and consumers with pinned GCC/Zig O0/O2,
  GCC ASan/UBSan and native frame/controlled-stack evidence.
- Exact defined/imported native symbols and public-header-only consumers.
- Full release/C/shared/Java and historical capacity/compile-fail/lint gates pass.
- Fresh broad review closes 04B; compiler metadata agreement remains 04C.

## Verification

- Initial composition gate `3750aec7-36d4-4051-b3c9-2d0cf6b20e61`:
  C ordinary regressions, Clippy and formatting pass.
- Expanded gate `1d16a4b5-8d72-45f8-8b9a-6ddef2108cc6`: all six targets
  pass, including 707 ordinary C tests, Rust lint/format, compile-negative,
  documentation and typed-source policy.
- Native proof compiles exact certificate output into three separate owning
  objects and a public-header-only consumer. It checks GCC/Zig O0/O2 and GCC
  ASan/UBSan O0/O2, three include orders, exact exported/imported native symbols,
  every generated frame and controlled 1 MiB stack execution.
- Each execution checks 8207 i32 inputs (seven boundaries and 8200 seeded values)
  plus both bool values. This target-level identity fixture proves separate
  generated-package execution; Rust compiler metadata/source agreement remains
  04C and native Rust-versus-C crate differential proof remains 04D.
- Full release/historical-capacity gate and independent review passed below.
- Explicit native/boundary invocation
  `429fba27-d211-4e84-980c-431c3b02166e` ran exactly two named tests
  (two passed, not an empty filtered run). Across all eight compiler profiles,
  the three-package native path used 24–120 bytes against a conservative
  34416-byte certificate bound. The actual chain admitted 94 packages and
  rejected the next live frame when the composed bound exceeded 1048576 bytes.
- Full invocation `dd5b8e80-d6ef-44c9-b3e5-91498b05f38b` identified a
  missing explicit test-only annotation on the handwritten native consumer
  helper. It was interrupted for repair and is not full completion evidence.
  Added `#[cfg(test)]` to that helper; production import policy and its exact
  allowlist remain unchanged.
- Policy-repair gate `a41c1f5f-6eb7-475d-93e5-6fb8e1ca627d` passed all
  three documentation, formatting and source-policy targets.
- Final full gate `ffe5aa4d-fe13-4b46-82a4-2c9be14ad48a` passed all
  311 tests across 351 targets, including historical capacity, C/shared/Java,
  frontend, compile-negative and lint checks. Cached test results remain enabled.
- Fresh broad Sol Extra High review found no actionable core errors. A separate
  certificate-cycle DFS is not required: immutable private-constructor owning
  certificates cannot contain an exact self-reference; finite attempts to reuse
  a source crate through a distinct certificate are explicitly rejected. This
  does not waive call-graph cycle detection or the driver graph checks in 04C.
