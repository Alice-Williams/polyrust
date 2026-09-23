# M35-03A-02X-04 — Checked Rust character constant integration

- Status: complete
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [Java foundation](M35-03A-02X-03-java-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-character-constants.md)

## Contract

Extend the private compiler constant witness with Char(char), checking original
type/width/domain after rustc evaluation. Reuse existing constant capability
mappings for declarations, reads, imports and aliases. Preserve original Char
facts, exact values and source owners across both target inventories, including
constant-only owners. Never infer source identity from a target integer.

## Definition of done and tests

Original multi-crate Rust and generated C/Java agree for public/private/local/
inherent constants, named imports and aliases. Include mixed same-valued Char
and I32 declarations, original docs/visibility and constant-only producers.
Typed probes corrupt kind/value/owner/declaration independently while preserving
positive controls; Java Char-to-I32 substitution must fail despite identical
target Int storage. Unsupported generic/trait/reference/type-alias forms and invalid
scalar facts reject atomically with absent/existing destinations unchanged.
Replace newly supported old negative cases with still-unsupported cases.

Measure real producer-value cache invalidation, failure against old truth,
success against updated truth and restored output hashes/cached native success.
Recompile Java dependents after value mutations. Export actual source-owned
examples, update partial parity only, preserve old outputs/unrelated WIP and
pass full Linux release/lint plus fresh broad review before commit/push.

## Implementation breakdown

1. Extend bounded shared source facts with an original constant-value enum:
   Bool(bool), I32(i32), I64(i64), F64Bits(u64), Char(char). These are descriptive
   compiler facts, not validity certificates; bit storage preserves signed zero
   and infinity without adding a dependency or treating NaN as admitted input.
   Derive the source kind from the enum, never from the target primitive. Keep
   the existing function/field constructor compatible and add a checked constant
   inventory attachment. Validate same-owner IDs, cross-inventory collisions,
   field-owner conflicts and combined resource limits. Char remains a Rust char,
   not unchecked u32. Split focused constant logic/tests into separate modules.
2. Collect exactly the emitted owned public constants from each lowerer's state,
   using rustc evaluation of their original definitions. Do not collect private,
   local or inherent constants that were folded into expressions and have no
   exported target object. Authenticate the complete retained facts at attachment.
3. Add the private ScalarConstantValue::Char(char) evaluator case only after
   normalized Char and four-byte-width checks and safe scalar decoding. Existing
   capability bindings lower it to C U32 and Java Int while retaining TypePlan::Char.
   Update exhaustive negative/probe matches without broadening runtime operators.
4. Reconcile complete original constant facts against exact C object and Java
   field inventories. Check both target type and exact value; keep Char/I32
   distinctions in the source facts even where Java syntax is identical. Consumers
   join original compiler values with retained producer facts before importing.
   Aliases continue to point to the original dependency owner, not a new field.
5. Serialize bounded constant source facts only for character-aware packages;
   include constant-only owners in character detection and preserve old output
   bytes (omit the new constants section when empty). Metadata text is descriptive,
   never a replacement for certificate identity. Keep C and Java formats aligned.
6. Add multi-crate fixtures with boundary/computed/public/private/local/inherent
   reads, a constant-only producer, same-valued Char/I32 declarations, aliases,
   Unicode docs and visibility controls. Add typed source/target/import corruption
   probes, strict native truth and atomic rejection tests, then actual cache
   invalidation/restoration and exported generated packages. No early parity claim.

The preceding Java checkpoint is complete and pushed as fa43bce; its GitHub CI
run 35796555366 passed. Shared constant facts, canonical compiler authentication,
both target reconciliations, source-aware dependency joins and versioned
metadata are implemented. The bounded source admission and its integration
criteria below are complete; wider scalar/runtime parity remains open.

## Verification history

The resumed LF-only snapshot 960bdb48 passes eight focused regression targets,
including 151 shared code-generation cases, source capabilities, existing
constant AST/local/import/native cases and character source authentication.
The expanded bd5da5ba snapshot passes 32 original/target constant-fact atomic
fault controls and 20 dependency-certificate controls. Same-valued Char-to-I32
substitution is rejected at Java's consumer source join. Existing character and
infinite-constant AST/rejection controls pass too.

The four-crate source fixture includes a constant-only producer, independently
owned equal values, an alias-only facade, and public/private/local/inherent
reads. Its Rust compilation, Clippy and rustfmt targets pass. Native integer
truth, producer mutations, exact metadata/docs and readonly/privacy checks pass.
Initial harness-only owner-key/absent-alias-field errors were corrected without
weakening generated-code checks. Standalone metadata probes now link the real
canonical evaluator; Java readonly rejection checks match Java21's diagnostic.

The full 1,033-target Linux Bazel release/lint gate passes on implementation tree
62ace4eac0ed696b94c8322b5d1acf5d1c528774 (12 executed, the rest cached), with
receipt /tmp/m35-character-constant-corrected-full.log. Native proof covers 18
original Rust reads at both profiles and 43 target observations/configuration,
separately compiled C owners/headers and Java owners/consumers. Four actual
producer faults are detected after recompilation. The 32 source/target-fact
and 20 dependency-fact atomic controls pass, as do 18 source rejection cases.

Actual test artifacts are exported to the ignored directory
generated/m35-character-constant-source-examples: 43 source/metadata/client files.
All 45 unrelated WIP hashes are unchanged. Broad GPT-6-Sol extra-high reviews
of b9d5eff3eaa3eeb7d2db98267cd3eb9ab70ddfa8 and the final implementation tree
62ace4eac0ed696b94c8322b5d1acf5d1c528774 are clean. An earlier review's
absent-alias-inventory harness finding was accepted and repaired; no core
finding was dismissed or test weakened.

## Completion evidence

The archive-isolated cache proof passes all nine phases. An ASCII constant
change rebuilds exactly seven affected metadata/bundle actions, leaves the
independent producer and compiler adapters cached, and changes every dependent
package. The old truth fails uncached; updated independent truth passes native
Rust/C/Java. Restoring both inputs restores every metadata/output hash and reuses
the passing native test result. Local receipts are in the ignored directory
generated/m35-character-constant-cache-proof (summary, BEP, actions and hashes).

All 530 files across the 70 preceding generated bundles are byte-identical to
an isolated build of fa43bcee52e9c20864dfca30adf8eec3dcb7f661. The 43 exported
example files are byte-identical to the native test artifacts. Neither examples
nor cache/proof receipts are committed. The final documentation snapshot receives
the full Linux release/lint gate again before this separately scoped commit/push.

This closes 02X only. Character conversions/methods/text, generic or trait
constants, type aliases and wider runtime parity are not admitted by this work.
