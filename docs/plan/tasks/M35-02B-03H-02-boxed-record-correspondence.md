# M35-02B-03H-02 — Authenticate boxed-record producers and field reads

- Status: complete
- Parent: [M35-02B-03H](M35-02B-03H-boxed-scalar-records.md)
- Depends on: M35-02B-03H-01
- Specification: [boxed scalar records](../../specification/typed-generation/languages/c/rust-boxed-scalar-records.md)

## Contract

Start with a safe nongeneric root-scope Rust function with immutable i32/bool
parameters. A complete local scalar-record literal initializes its fields from
those parameters; one standard Box constructor consumes that record binding.
Zero or more whole-Box local moves follow. The final scalar exit explicitly
dereferences the current Box and selects one authenticated i32/bool field.
Preserve tail versus explicit-return structure and the actual root cleanup scope.

Keep this direction separate from G's record of independent Box owners. Reuse
the H-01 constructor input and shared compiler type identities. Do not infer
payload origin, field selection or drop responsibility from names or equal types.

## Definition of done and tests

- Inspect pinned HIR and PostCleanup MIR before enabling a body certificate:
  scalar initializer producers/order, payload aggregate, constructor argument
  transfers, whole Box moves, payload pointer/field read and final drop.
- Retain the canonical literal, field DefId/FieldIdx/kind/type, parameter HirIds
  and compiler locals, payload storage/argument places, constructor identity and
  selected field projection. Compiler staging storage is not a new HIR binding.
- Authenticate every payload field producer by actual parameter identity and
  declared field index. Source initializer evaluation order and declaration
  order remain separate. Scalar copies are not invented ownership transfers.
- Require exactly one owning Box chain and its final exact drop after the scalar
  read and before return. Account for all calls, relevant payload/Box storage,
  assignments and normal blocks. Preserve the existing whole-body budget guards.
- Positive cases cover i32/bool results, reversed field initializers, aliases,
  same-spelled module records, shadowing/local Box moves and explicit returns.
  Mixed scalar fields cannot be interchanged through same-size representations.
- Wrong payload constructor/producer/field/index/type, missing/duplicate operands,
  wrong Box movement/read/drop and source-scope substitutions reject. Include
  invalid Rust, nonempty inventories, private construction and non-erasure tests.
- Previous scalar Box, record-field, constructor registration and native C/Java/
  lint gates remain green. Fresh review, isolated exact-tree proof, documented
  evidence and a dedicated checkpoint commit/push close this step.

Initially exclude implicit autoderef field syntax unless the pinned adjustment
sequence is explicitly authenticated in this same contract and its tests.
Also exclude nested/owned record payload fields, borrows, mutation, arbitrary
calls, custom Drop/allocators, unwind, conditionals and function-owned transfers.
This compiler-only step does not enable C/Java heap output.

## Pinned representation and implementation

Observation gate `3892e63d-e1ae-4af3-af92-64f2895ad65b` passed both
probe/format targets. Six actual PostCleanup bodies establish the following:

- Field scalar parameters are copied into staging locals in source initializer
  order, then those locals occur as aggregate operands in declaration order.
- The record binding is moved (non-Copy record) or copied (Copy record) into
  a distinct argument local. Box::new moves that argument local. Retain a closed
  transfer enum and check the compiler's Copy predicate; neither is a Box clone.
- Whole Box bindings move along a single chain. The last owner supplies the
  compiler's Unique/NonNull-to-payload-pointer cast. The result copies one
  declaration-indexed scalar field through that pointer, then drops that owner.
- Explicit return has the same observable cleanup route as the tail form.
  Implicit field syntax has the same MIR but different HIR; it is deliberately
  excluded from the first certificate, not silently treated as explicit syntax.

Implement focused source, aggregate, relation and evidence modules. Reuse the
existing bounded trace and independently authenticated scope facts. Extend only
the private payload-pointer helper; historical Box<i32> entry points stay closed.
No body certificate is complete until its negative mutation matrix, independent
review and full exact-tree gate pass.

## Current evidence

- Gate `e050bb02-7ae7-4b8c-bb6b-b9f8da17d257` passed all six focused
  targets in 18.082 seconds: runtime, Rustfmt and four exact compile-negative
  contracts (private body/field, non-erasure and query-only construction).
- Ten accepted source forms cover both scalar kinds, reversed initializers,
  Copy/non-Copy payload transfer, aliases, same-spelled module records, Box
  moves, shadowing and explicit return. Sixteen valid unsupported forms reject;
  the old scalar-Box fixture still executes its historical evidence/mappings.
- Forty-two altered MIR bodies reject independently, covering nominal/type/
  producer/aggregate/transfer/read/drop/assignment relationships, including a
  same-type i32 parameter substitution and both Copy-predicate directions. Three private
  source-plan substitutions reject (parameter, selected field, cleanup scope).
- Four invalid Rust controls fail with E0382/E0063/E0062/E0502 before any proof
  marker. Eight valid-source mutations defeat the fixture oracle, including an
  empty inventory, missing historical case, nominal alias substitution and
  explicit-return-to-tail replacement. The harness requires all proof markers.
- Initial exact-tree gate `8b6f4888-42ce-49e8-870d-f8e169be80b2` passed
  450 tests across 577 targets in 46.680 seconds after verifying 2,243 Git blobs
  and modes for `c3cbc6f1e33c0ecf97e5a62e850151401eedfb00`.
- Independent review found that canonical tail/return information was retained
  privately but inaccessible to the lowering consumer. This finding was
  accepted and fixed: the certificate exposes the existing SourceExit enum and
  authenticated terminal MIR return location. Consumer-side canonical-node and
  terminal-return assertions, plus the source replacement control, prove it.
  Gate `fd400fa1-d3e8-457f-882a-010540016abd` passed all six focused targets
  after this repair in 17.424 seconds.
- Review also identified missing direct negative evidence for MIR callee,
  instantiated constructor arguments and destination. Authentic local-wrapper
  and distinct-record call operands now exercise those checks, together with a
  wrong same-typed Box destination. Gate `08bad403-4eb6-4474-a2f9-0ec6d51b5bad`
  passed runtime/format targets in 16.075 seconds after these additions.
- Repaired exact-tree gate `d06affa7-7b43-421e-ad9d-e1d5f13d125e` passed
  all 450 tests across 577 targets in 39.534 seconds for
  `d8c229d5f76f06fdcf928e4183c09137f9a9be4a`, with all 2,243 blobs/modes verified.
  The original reviewer confirmed both findings resolved. A fresh independent
  Sol Extra High reviewer then found no remaining core defects across the full
  repaired design and test matrix. No findings were rejected as disagreements.
- Optional whole-body one/128-field cases and a same-typed selected-field
  corruption were classified as coverage additions, not correctness defects:
  the relation compares the precise FieldIdx and iterates the authenticated
  fields generically; H-01 separately gates payload-shape limits. These do not
  expand H-02's admitted grammar and can be added as subsequent hardening.
- The final closure tree receives the same isolated gate before commit/push;
  that commit records its exact tree and invocation. No target heap generation
  is enabled; broader M35-02 ownership/runtime obligations remain open.
