# M35-01D-04C-03-01 — Compiler-to-certificate foreign bindings

- Status: complete
- Depends on: [M35-01D-04C-02](M35-01D-04C-02-declared-crate-driver.md)
- Parent: [M35-01D-04C-03](M35-01D-04C-03-foreign-bindings.md)

## Outcome

Lower actual compiler-resolved foreign scalar calls through existing typed C
dependency witnesses, retaining one separately certified package per source
crate. This is an in-memory integration checkpoint, not bundle publication.

## Implementation

1. Expose a private-field, compiler-invocation-scoped dependency context only
   after source/loaded-artifact agreement. Associate each loaded compiler crate
   with the exact previously checked owning result; do not expose unchecked
   descriptors or let callers populate the association.
2. Reconstruct each owning C package in the same checked compiler invocation.
   Run projection, verification, linking and resource certification unchanged,
   then derive its existing CDependencyApi. Retain owned results only.
3. Reuse the FunctionSignatures capability for compiler DefIds, local or foreign.
   Resolve ordinary direct calls to an explicit local/foreign distinction.
   Continue to reject generic/indirect/adjusted/unsafe/unsupported signatures.
4. For a foreign call, join the compiler's StableCrateId/DefPathHash and mapped
   signature to the exact public CDependencyFunction in that checked owner.
   Import it with CRegistry::import_function; never construct an external
   prototype from names, signatures, paths or descriptive manifests.
5. Keep owned-function traversal separate from imported references. Queue only
   local bodies, deduplicate foreign definition identities, and register the
   complete required callable inventory before lowering bodies.
6. Keep single-crate entry/package behavior unchanged. Unsupported foreign
   public re-exports, heap/reference signatures and arbitrary libraries diagnose.

## Tests and proof

- Real two-crate, transitive and diamond Rust fixtures lower and certify in the
  container, including renamed/repeated aliases and equal names with distinct
  defining keys.
- Compare foreign compiler identities and mapped signatures against certificate
  witnesses; reject wrong owner, absent/private declaration and signature
  substitutions without fallback.
- Assert each package owns only its local definitions and calls use
  consumer-branded certified imports. No foreign HIR body is visited as local.
- Preserve existing source/metadata mismatch, privacy and unsupported-call
  negatives. Verify rustc analysis is not bypassed by the new integration path.
- Existing C resource composition, header discovery and exact certificate
  identity checks remain mandatory; foreign calls are not zero-cost leaves.
- Full release/frontend/C/shared/Java and lint/format/policy gates pass, followed
  by fresh broad independent review with all actionable core findings resolved.

## Implementation evidence

Added private-constructor, invariant-compiler-lifetime CheckedDependencies
binding actual loaded CrateNums to exact prior owning results. The real compiler
callback test now also compares each bound result's StableCrateId to its
invocation-local CrateNum. Its four-target gate passed in
`4387f23d-1e64-45fc-a140-5a7ffd9da3b2`.

The adapter's new `--check-crates` operation lowers each crate in the same
checked analysis, creates a CDependencyApi from its certified package, and
retains separate owning results. The private foreign lookup authenticates owner
and exact declaration; the existing FunctionSignatures mapping verifies the
foreign signature before typed import registration. Local and foreign call
variants have separate inventories; only local bodies are traversed.
The legacy public lowering entry does not accept a foreign lookup.

Actual compiler/C certification fixtures passed in
`5ee382c4-86db-4bdc-a1e3-780a91c122f7`: bool/i32 calls, private local
helpers, transitive/diamond/repeated aliases, unused dependencies and same-name
versions. Private access, arbitrary libraries, generic calls, foreign public
re-exports, unsupported signatures/expressions/coercions and stale/swapped
metadata reject without output. Boolean negation is expressed with the already
supported conditional shape; unsupported logical-not syntax remains an explicit
negative rather than an accidental language extension.

Three test-only compiler builds deliberately substitute a wrong owning
certificate, another public declaration with the same signature, or a wrong
mapped signature while retaining the expected declaration. All are rejected
at their intended joins. These runtime tests passed in
`65c6e181-fe90-4d10-81a4-995cf7eac8d4`; only Bazel call formatting
needed correction before the full gate. Full integration and fresh review
remain required. No bundle, imported manifest or native equivalence claim is
made by this in-memory checkpoint.

Full integration `a5b3406e-82d9-4537-b8d8-d1a43f8e07e9` passed all
335 tests across 389 targets, including the mutation builds, legacy standalone
compiler probes, capability compile-fail contracts, C/shared/Java suites and
Rust/Bazel lint gates. Fresh broad independent review is in progress.

The expanded gate including the independent publication platform probe passed
all 336 tests across 391 targets in
`2b4dcbfb-f324-4a92-9e2e-6558f9501b0b`. A further C fixture explicitly
exercises zero-parameter and mixed bool/i32 multi-parameter foreign calls with
nested argument evaluation, complementing the existing one-parameter cases.

## Definition of done

All assertions above have test coverage and recorded gate/review
evidence. No generated files or serialized proof authority are introduced.
Publication remains blocked until 04C-03-02 completes.

Fresh independent Sol Extra High review found no actionable findings. Owned and
imported inventories are structurally separate and independently revalidated by
certification; the black-box check command deliberately publishes only a final
count. Direct manifest inventory assertions belong to the next checkpoint.
The final arity/docs gate was `4d423497-a282-424c-b28a-7b493e5e20d8`.
Commit/push remains held for the requested C/Java integration gate; this status
records implementation and verification, not a claim of remote publication.
