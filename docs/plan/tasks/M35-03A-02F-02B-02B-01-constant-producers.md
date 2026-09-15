# M35-03A-02F-02B-02B-01 — Certified C constant producer APIs

- Status: complete
- Parent: [C constant imports](M35-03A-02F-02B-02B-c-imported-constants.md)
- Depends on: M35-03A-02F-02B-02A

## Contract

Implement the producer half of the [C constant specification](../../specification/typed-generation/languages/c/rust-public-constants.md).
Expose read-only certified constant views and opaque CDependencyConstant
witnesses containing the actual producer certificate, Rust declaration identity,
registered const object, exact literal, unqualified read type, allocated symbol
and owning header/source. Reuse CObjectRef, CObjectType and CLiteral; no
caller-authored name/signature/value can create a witness.

Reconstruct one complete function-and-constant export inventory. Constants-only
packages must not require a first function or a fabricated stack frame.
Effective visibility, aliases, declaration IDs, crate metadata, owning files,
readonly type, initializer and actual definition must agree. Synthesized or
private constants cannot be disguised as Rust public dependency declarations.

Replace the interim object-bearing-producer rejection only when the complete
inventory and native collision proofs pass. Existing function imports from mixed
producers must reserve every constant symbol, including unreferenced exports.
Consumer constant registration and reads remain the next child.

## Definition of done and tests

- Constants-only and mixed APIs retain exact bool/i32/i64 identities and values,
  const storage/unqualified read types, allocated symbols, provenance and files.
- Certified read-only views enumerate each definition once, independent of file
  order. Empty function inventories are valid and have zero function frames.
- Missing/wrong exports, reachability, source provenance, type/value/file changes,
  duplicate declaration identities and conflicting metadata reject.
- Private-field/constructor compile-negative tests prevent forged producer
  witnesses. Same-source/different-certificate identity remains distinguishable.
- Function-only imports from a mixed producer retain complete constant symbols;
  direct and transitive unselected-constant collisions reject. Independently
  compiled GCC/Zig O0/O2 controls demonstrate the corresponding native conflicts
  and positive non-conflicting linkage.
- Focused modules, updated specification/status, reviewed evidence, full isolated
  Bazel/lint gate and a scoped commit/push. No legacy deletion.

## Boundary

No CImportedValue or imported object registration is enabled by this child.
Rust HIR admission, producer metadata serialization and bundle publication remain
later tasks. Completing producer APIs alone does not complete constant imports.

## Implementation and proof inventory

- CDefinedConstant is a read-only view of actual definitions in the certified
  source, not a caller-authored object/name/value pairing. Header declarations
  are not counted twice; unqualified read types use the existing C conversion.
- CDependencyConstant retains the original immutable certificate authority,
  declaration ID, registered object, resolved symbol, literal, read type and
  owning files. Private fields and the certificate-only view entry point have
  compile-negative controls. Clones retain authority after the API is dropped;
  separately certified values under the same registration remain distinct.
- The producer inventory includes every public function and scalar constant.
  Constants-only packages need no synthetic callable. Missing/extra exports,
  aliases, namespaces, reachability, visibility, provenance and conflicting
  identities are covered, including diagnostics at earlier projection boundaries.
- Complete dependency symbol inventories now include constants, even when only
  an unrelated function is imported. Called and unused direct/transitive imports
  reject collisions with owned functions. Native GCC/Zig O0/O2 controls separately
  compile valid packages and then demonstrate header or external-link collisions;
  transitive controls update the driver too, avoiding false undefined-name proof.
- Native positive examples are exported before mutation as Bazel undeclared
  outputs. The inspected host copy is ignored at
  generated/examples/c-constant-producers-dcefab8, with Direct/Transitive producer,
  consumer and independent driver sources. No generated output is committed.

## Verification evidence

Tree 97aa9f97afe4ec5d4fb63da44ee343fd1848ac03 passed all six preflight targets:
727 ordinary C tests, Rust Clippy, rustfmt, buildifier, documentation and source
policy. Invocation 80833aa1-9e18-4a1e-a9e5-3c593734d9d7; 60.145 seconds.
Tree dcefab89a5cf23897b6dc19d6e3079211684d80f adds inspectable example export
and specification updates. Its isolated full release/integration gate passed all
624 test targets across 816 targets in 483.388 seconds, invocation
e47216e5-dcdb-42ab-8d51-8fc1da7596aa. All 2,544 archived Git blob bytes and
executable modes were verified. The run includes 727 ordinary C cases, all five
heavy C capacity shards, 64 C documentation/compile-negative cases, release_gate,
all rustc-frontend experiments, C/Java integration, shared codegen, documentation
and Rust/Bazel/source-policy lint gates. Existing caching remained enabled;
no test or legacy implementation was disabled or removed.

A fresh independent Sol Extra High review of the exact dcefab89 implementation
tree found no actionable current-scope errors. It reviewed all changed files and
the surrounding certification, source inventory, authority/lifetime, complete
symbol reservation and native/compile-negative proof boundaries. Consumer value
registration and zero-frame dependency acceptance remain explicitly child02.

The final documentation tree is gated again before its scoped commit/push.
