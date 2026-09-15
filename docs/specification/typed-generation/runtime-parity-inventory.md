# C/Java runtime migration inventory

- Status: baseline recorded; replacement parity incomplete
- Contract: [ordinary generated packages](runtime-free-packages.md)
- Plan: [M35-03A](../../plan/tasks/M35-03A-runtime-free-parity.md)

This is a coverage map, not a claim that all legacy capabilities already work
through Rust source. The machine-readable inventory is
`experiments/rustc-frontend/test/runtime_parity_inventory.json`. Its drift test
accounts for all 42 registered Java capability mappings, 38 explicitly admitted
C intrinsic operations, nine Java helper families and 17 C template sections.
Java helper variants must match the executable ALL catalogue registrations,
just as capability slots must match their consuming builder registrations.
These counts describe the current implementations, not a shared language spec.

## Current replacement coverage

Both targets currently share the closed no-heap source subset below. Neither
target has full replacement evidence for any broad legacy capability family.

| Functionality | Rust-source C | Rust-source Java | Remaining work |
| --- | --- | --- | --- |
| Values and comparisons | i32/bool literals, scalar comparison, immutable places, built-in bool negation | Same source subset | i64/f64/char/unit, constants, aliases, lazy and/or, integer and float operations: M35-03A-02 |
| Functions and modules | Closed typed signatures/direct calls, crate-owned headers and implementations | Closed typed signatures/direct calls, crate-owned Java packages | Wider signatures, methods and migrated consumers: M35-03A-03/06 |
| Records and control | Closed scalar-field records/shared borrows, local bindings, structured branches | Same source subset | Owned shapes, enums, interfaces, loops and patterns: M35-02 then M35-03A-03 |
| Text, Unicode and bytes | No general replacement mapping | No general replacement mapping | All legacy operations and explicit encoding/indexing policies: M35-03A-04 |
| Lists, options and results | No general replacement mapping | No general replacement mapping | Container operations, nested values and failure propagation: M35-03A-05 |
| Portable tests, CLI and corpus | Legacy portable generator still active | Legacy portable generator still active | Move consumers only after their feature proofs pass: M35-03A-06 |

Compiler observations of Box/owned-record operations are not executable C or
Java heap support. Their work remains in M35-02; do not count them as parity.

[M35-03A-02A](../../plan/tasks/M35-03A-02A-boolean-negation.md) adds partial
JavaBooleanLogic coverage: built-in bool negation only. Its
`boolean_negation_native_test` checks 8,204 inputs and 13 results against Rust
and independent truth values, with Java lint and GCC/Zig O0/O2 consumers.
`boolean_negation_rejection_test` covers unsupported/invalid operands and
atomic publication; `boolean_negation_contract_test` checks registration and
input privacy. Lazy and/or remain unsupported, and full_features stays empty.

Evidence anchors for the current subset are the executable registrations in
`src/c_lower/capabilities/mod.rs` and `src/java_lower/capabilities/mod.rs`, their
object/literal type mappings, and the crate native differential targets under
`experiments/rustc-frontend`. The new `runtime_free_bundles_test` checks actual
four-crate bundles produced by these paths. It does not prove semantic parity
or detect every possible support routine inlined into a source-owned file.

## Legacy-specific limits and disposition

C's callable-container validator admits List<String> and Option<I64>, not all
container shapes. It emits enum ABI declarations but does not provide general
callable enum lowering. Its callable blocks reject statements and bounded
iteration. The intrinsic inventory is the validator's allowlist, not every
Intrinsic enum variant or occurrence in the generator. C ABI/type handling and
constant restrictions must also be audited when their replacement tasks start.

Java registrations cover a broader portable model, including interfaces,
enums, collection/result operations and test lowering. A registration is not
proof that every nesting, ownership or generic shape is supported. Each task
must enumerate its successful legacy cases and add corresponding source and
native tests before declaring that family complete.

Every machine-inventory entry has exactly one primary parity task. Helper
families may serve several kinds of operation: for example, C allocation and
ownership template sections are filed under text/bytes for retirement tracking,
but their replacements also depend on M35-02 and collection work. This grouping
does not authorize deleting a shared helper when just one caller is migrated.

## Baseline checks and limitations

`runtime_parity_inventory_test` compares lexical registrations/allowlists and
template markers against the checked-in inventory. It rejects injected missing,
extra and duplicate entries. It deliberately is not a Rust parser or a semantic
admission checker; implementation refactors must update its extraction rules.

`runtime_free_bundles_test` requires exactly the expected source-owned files,
manifests and root identities, with no extra runtime or arbitrarily named
support file. Fault controls remove source, duplicate crates, add files and
introduce legacy and renamed support references. Exact index/member/manifest
key sets and manifest versions reject hidden support metadata. C imports must
identify an external definition in the declared owner's header, and this fixed
corpus has an exact include inventory. Java dependency owners must be unique,
non-self and in-bundle; this corpus needs no Java import declarations.

These are fixed-corpus lexical/metadata regression checks, not a general source
parser. Fully qualified Java expressions, C declarations, inlined routines and
arbitrary program behavior remain the typed compiler/native/privacy gates'
responsibility. No guarantee is inferred from the absence of an import line.

Future coverage updates must cite feature-specific positive, negative and native
tests. Preserve the baseline inventory until each old implementation and all
its consumers can be removed together with reviewed replacement evidence.
