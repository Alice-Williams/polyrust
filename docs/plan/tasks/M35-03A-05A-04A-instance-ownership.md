# M35-03A-05A-04A — Checked instance identity and placement plan

- Status: in-progress
- Parent: [compiler results](M35-03A-05A-04-compiler-results.md)
- Depends on: [Java transport proof](M35-03A-05A-03C-java-result-proof.md)
- Specification: [canonical owners](../../specification/typed-generation/rust-canonical-type-owners.md)

## Contract

Separate compiler-authenticated instance witnesses, bounded descriptive instance
keys, source-crate ownership and generated type ownership. Complete the normative
encoding/profile decision before production edits. Extend the observation-only
compiler experiment to prove original normalized definition/argument/variant/field
identity across a two-producer fan-in graph; do not open source lowering yet.

A private graph interner assigns deterministic placement when a checked instance
is first encountered and reuses its exact authority after full-fact reconciliation.
The complete graph freezes and reserves capacity before publication. Its public
descriptive projection cannot mint compiler or backend certificates. This is a
same-graph contract; independent processes must recheck a merged graph.

## Checkpoints

1. [04A-01 — descriptive identity foundation](M35-03A-05A-04A-01-instance-model.md) — complete; 1,043-target release/lint gate and independent reviews pass.
2. [04A-02 — authenticated graph observation](M35-03A-05A-04A-02-instance-probe.md) — complete; 1,047-target release/lint gate and exact-tree review pass.
3. [04A-03 — original error state](M35-03A-05A-04A-03-error-state.md) — planned;
   required after the pinned compiler exposed a nonzero error payload.

Each checkpoint receives its own release/lint gate, review and commit. No
checkpoint alone completes this parent or opens compiler Result admission.

## Implementation and definition of done

1. Specify exact enum variants, canonical name encoding, profile compatibility,
   original opaque error identity and graph registration/publication stages.
2. Add focused shared identity/placement modules and separate Bazel tests at their
   dependency boundary; keep caller-supplied metadata distinct from proof.
3. Derive compiler-session witnesses from actual checked Rust types. Reconcile
   facts across the complete graph without retaining invalid rustc lifetimes.
4. Test normalized aliases in the probe only, same instance across two producers,
   wrong variant/field/error identity, distinct same-shaped instances, conflicting
   facts, cycles, missing owners and exact/one-over plan bounds.
5. Permute graph traversal and consumer additions; compare deterministic owner
   placement. Compile-fail tests reject forged witness construction.
6. Preserve current source rejection behavior/output, run full Linux Bazel
   release/lint and independent GPT-6-SOL review, then scoped commit/push.

The implementation cannot claim target owner support or full source admission;
those have separate gated tasks.
