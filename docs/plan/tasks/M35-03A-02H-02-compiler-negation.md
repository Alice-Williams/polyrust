# M35-03A-02H-02 — Checked Rust wrapping-negation mapping

- Status: planned
- Parent: [wrapping negation](M35-03A-02H-wrapping-negation.md)
- Depends on: [target proof](M35-03A-02H-01-target-negation.md)

## Contract and implementation

Add a target-independent WrappingNegation capability with private checked input
and an exact-width enum. Resolve canonical compiler-owned HIR and its actual
standard-library inherent method DefId; never authorize an operation merely
from spelling. Require safe nongeneric i32/i64 by-value receiver/result signatures,
no unsupported adjustments, and unchanged exact width. Support method syntax
and explicit associated-function syntax only after both have identity proof.

Register executable C/Java mappings through consuming typed builder slots.
Materialize the receiver exactly once. C emits the proved guarded typed form;
Java emits typed primitive negation. Standard primitive operation discovery must
not create a fake foreign package dependency; ordinary function inventories and
original producer closure remain unchanged. Reject rather than partially emit
unrecognized methods, trait overloads, generic/indirect calls or other widths.

## Required proof and definition of done

- Independent checked-input probes authenticate canonical HIR, exact built-in
  identity, width, signature and adjustment policy. Lookalikes/shadowing and
  invalid forms reject atomically.
- Missing/duplicate/wrong capability/context/input/output and private input
  construction have intentional compile failures; positive complete builders
  pass. Existing missing-slot controls still omit only their named capability.
- AST probes prove the C guard and normalization, Java negate/result type,
  one receiver evaluation and original direct-call identity; probe/production
  bytes agree.
- Real multi-crate Rust/generated C/Java fixtures and independent arithmetic
  truth agree at boundaries and broad deterministic inputs. Native copied
  instrumentation detects dropped/duplicated receiver calls.
- Old manifest schemas, public/private names, docs and dependency authority
  remain unchanged. Export actual examples outside Docker, without committing
  generated artifacts.
- Full Linux Bazel/native/lint gates and a fresh independent Sol Extra High
  review pass before a separate scoped commit/push.
