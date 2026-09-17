# M35-03A-02H-02 — Checked Rust wrapping-negation mapping

- Status: complete
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
  identity, width, signature and adjustment policy. Lookalike methods and
  invalid forms reject atomically. An ordinary function with the same spelling
  remains an ordinary direct call, never a primitive-operation witness.
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

## Implementation and review decisions

The compiler authenticates the standard core crate through its actual Copy
language-item DefId and normalizes inherent owner/signature metadata using the
pinned rustc API. Canonical HIR pointers and original TypeckResults bind the
private input to the function being lowered. Each backend registers the
twenty-first executable builder slot; neither renderer receives Rust HIR.

Both mappings materialize the receiver once. Function inventory skips the
authenticated primitive itself but still visits its receiver's original
function calls. Unrelated methods and associated functions keep their existing
unsupported-call boundary; names are an exclusion filter, never positive
authority.

Accepted independent review corrections:

- Resolve argument-bearing method identity before filtering unrelated names,
  so wrapping_add and other methods retain their existing unsupported-expression
  behavior. A same-spelled lookalike with extra arguments still rejects.
- Public-constant missing-slot fixtures previously also omitted UnitEffects.
  Register UnitEffects and WrappingNegation in their common tail. Six positive
  compiler controls now restore only the deliberately omitted slot in the very
  same fixture, preventing extra missing slots from masking the tested failure.

The full regression gate also exposed two standalone compiler probes missing
the new rustc_abi declaration. Their dependency declarations and compile-only
capability exercise are updated; no warnings or existing rejection tests are
disabled.

## Implementation gate evidence (2026-09-17)

- Exact implementation tree: f734150cbfc5d8a8ad559ed55f9a4f6b3856d76a.
- Linux dev-container Bazel test //... //:release_gate: 1,173 targets,
  778/778 tests passed (61 executed, 717 cached), 241.955 seconds.
  Invocation: eb9e771f-b1eb-469b-a5c6-c7cbd6b2e3b2.
  Rust formatting/Clippy, Bazel formatting, import policy and all legacy
  regression checks remain enabled.
- wrapping_native_test: 8,750 boundary/deterministic inputs and 52,500 expected
  result values across Rust, separately compiled C owners/clients with GCC/Zig
  O0/O2 plus GCC UBSan, and Java 21 with strict lint. Value-preserving dropped
  and duplicated receiver calls fail the separate trace oracle.
- wrapping_ast_test: 12 actual mapped operations per target, both widths/forms,
  copied-HIR and wrong-context rejection, exact guard/normalization/type and
  original-call identities; probe and production package bytes agree.
- wrapping_contract_test: 14 intended compile failures. Six restored-slot
  public-constant positive compiler controls compile from the same fixtures.
  wrapping_rejection_test: 84 atomic rejections from valid unsupported Rust,
  preserving absent/existing output.
- Actual unmodified examples exported outside Docker to
  generated/examples/wrapping-negation-f734150/README.md, including Rust inputs,
  both complete generated bundles and handwritten clients. Exported bytes are
  compared with the final native test artifacts; generated output is not staged.
- All changed Rust files remain below 500 lines; 19 unrelated preserved
  ownership files match their original hashes. No runtime removal, third-party
  dependency or manifest/schema-version change is included.
- Fresh Sol Extra High reviewer wrapping_compiler_clean_review examined that
  exact implementation tree and reported no core or optional findings. Both
  earlier findings were accepted and fixed; there are no rejected findings.
