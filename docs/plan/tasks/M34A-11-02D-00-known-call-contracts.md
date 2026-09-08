# M34A-11-02D-00 — Closed C library-call foundation

- Status: complete
- Depends on: M34A-11-02C

## Goal

Give the safety verifier one authoritative, non-forgeable source of library
signatures and obligations before it reasons about calls. This moves the
library-contract prerequisite out of 03; it does not implement linking.

## Definition of done

- Implement the exact initial inventory in c/known-call-contracts.md under
  dialect/catalogue. Closed identities own signatures, headers, native library
  requirements and operand-specific obligations; no caller-authored effects.
- Extend callable AST construction/reconstruction with a distinct known-call
  alternative. A generated function with the same spelling or prototype cannot
  acquire a known contract. Generated indirect provenance remains independent.
- Retain every argument, type, qualifier and actual result category. Known
  integer predicates return Int, not Bool; macros cannot become function-address
  values. No variadic, arbitrary foreign or raw-source call route is introduced.
- Reuse existing known object/constant identities and scalar model rather than
  authoring duplicate type metadata. Do not register library bodies as generated.
- Every new callable variant is visited by structural, lexical, completeness,
  initialization and later safety traversals. Metadata is an obligation, not
  successful ownership, range, initialization or resource evidence.
- Keep CDialect/shared bindings, header emission and native dependency projection
  in 03, and actual generated runtime function bodies in 05/06.

## Tests and proof

- Exhaustive expected signature/qualifier/result/header/library inventory.
- Positive exact calls; wrong arity/type/void-value use; same-signature generated
  substitution; crossed brands; private cached child mutation; macro-address
  category rejection and independent known-call reconstruction controls.
- Both pinned C compilers check the callable type/predicate-result inventory;
  repository-owned probes are not certified generated packages.
- Rustdoc compile-fail, Rust/Bazel lint, full cached tracked/release gates and
  eight-target deterministic conformance. Record actual labels and invocation IDs.

## Implementation and evidence

dialect/catalogue owns ten closed CKnownCall identities, exact signatures,
CHeader/CSystemLibrary metadata and CKnownOperands views bound to the actual
ordered CCall arguments. Each production catalogue module is below 125 lines.
The callable enum now owns Direct(generated function), Indirect(pointer plus
generated contract function), or Known(identity); no optional fake generated
function or caller-authored standard signature accompanies a known call.
The callable's signature is borrowed from its generated reference or derived
from the one known catalogue. CCallContract distinguishes generated body
obligations from named-role standard operands; neither is verified effect state.

Construction and contextual reconstruction cover all three alternatives.
The shared access walk visits every known argument; the macro variants have no
function-address/indirect entry. Statement evaluation authenticates the callable
brand as well as generated references. Known predicates retain Int and require
explicit Bool conversion. Existing known FILE/constant/scalar identities are
reused, with no new external dependency or executable source route.

The seven new Rust tests cover all ten signatures, qualifiers/results, every
argument's arity/type/brand/visibility/initialization, actual argument-role
projection, private cached-child corruption, and same-signature/name generated
substitution. A byte-copy contract does not initialize addressed storage before
the later effect proof. Existing generated callable tests retain both direct
and indirect contracts. Rustdoc proves standard identities cannot become
generated function addresses or indirect generated contracts.

Repository-owned test/known_calls_probe.c checks all ten native prototypes or
macro result categories, FILE pointer identities and actual runtime operations.
The Zig targets run at O0/O2; GCC checks version 14.2.0 and runs both modes,
including a rejected Bool predicate-result control. The probe is an exact
source-policy test-infrastructure exception with adjacent/copy/src negative
controls, not a production AST or generated body. The GCC script is executable
in Git. All three labels are permanent members of //:release_gate.

The initial unit run 77a4e623-9d78-4d4f-83c0-6cd538d739a4 failed two existing
fixture assumptions after callable checks moved earlier: contextual private
corruption must be injected after public construction, and statement errors now
wrap the expression's CrossRegistry diagnostic. The fixtures retain the same
semantic rejection controls. The first full run
e695b780-bcb5-4d34-8686-a11f3003346e caught a Clippy slice-clone issue and the
missing exact native-probe policy registration. Both are corrected without
lint suppression or broad policy relaxation. These failed runs are not gates.

An independent uncapped Sol Extra High review identified that the native probe
targets initially lacked release-suite membership. Accepted and fixed by adding
the three exact labels; the final release count increases from 251 to 254.
Production source stayed unchanged during review. The independent reviewer
completed the uncapped audit and returned Pass on the repaired worktree against
base ea9ee0e311aaf2b69e59f74c911820d0d55e6404: no remaining production,
semantic or proof-boundary defect, and no optional additions required. This used
an existing Sol Extra High reviewer for a new independent 02D-00 assignment,
not a claim to have spawned a fresh agent context. The reviewer read all
in-scope source/tests/policies, did not run builds, and checked the supplied
execution records against the final tree. The complete safety stage remains open.

| Gate | Invocation | Result |
| --- | --- | --- |
| C unit, compile-fail and all three native probes | 5054ac29-0258-4928-adef-2da93055a1cd | Five targets pass; 180 unit tests; both native compilers O0/O2 |
| Every tracked Bazel rule including Rust/Bazel lint and policy | 6f528f97-97dc-4ec5-af39-9c82ff81f491 | 442 rules; all 317 test targets pass, one executed |
| Cached release with permanent new probe membership | 5970e9ad-939d-4dd6-885e-31673db25ffb | All 254 test targets pass |
| Eight-target conformance and deterministic manifests | d754be50-f1b2-4878-865c-6b37e8f029a9 | 50 cases and one portable test; eight targets agree; repeated manifests byte-identical |

All commands run in polyrust-dev-step0 at /workspace, with Bazel's pinned
toolchains and normal action/test caches. The tracked gate queries every rule
in git-tracked BUILD.bazel packages, excluding the untouched user-owned
examples/real-world/stdlib-abs tree. This checkpoint does not expose CDialect,
Supports, a verified package or any renderer; 02D-01 onward remains required.

## Commit gate

Commit and push this completed slice with M34A-11-02D-00. No Supports advertisement,
verified package or legacy-source certification follows from this foundation.
