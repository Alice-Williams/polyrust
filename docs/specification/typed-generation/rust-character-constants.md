# Checked Rust Unicode scalar constants

- Status: independent oracle, target foundations and checked source integration complete
- Plan: [02X](../../plan/tasks/M35-03A-02X-character-constants.md)
- Prerequisite: [character values](rust-character-values.md)
- Targets: [C17](languages/c/rust-character-constants.md), [Java21](languages/java/rust-character-constants.md)

## Original constant authority

Use rustc constant evaluation after canonical declaration, nongeneric owner,
normalized Char type and exact four-byte scalar checks. Safely decode the bits
to Rust char, rejecting surrogates and out-of-range values; never use unchecked
construction. Add Char(char) to the distinct ScalarConstantValue domain. Reuse
the existing executable constant capabilities instead of adding support flags.

Support the existing bounded constant grammar: module/private/inherent reads,
block-local constants, public declarations, named dependency imports and
authenticated public constant aliases. Evaluating a constant expression does
not admit its operations as runtime source expressions. Keep existing rejection
of generic/trait owners, borrowed constant storage and source type-alias uses.

## Typed lowering and metadata

Each source declaration's original identity, Char kind and exact value must
survive lowering and publication. Target constant inventories describe target
types and values, not original Rust types. Extend bounded source facts where
needed and authenticate them from compiler declarations. Reconcile their exact
declaration set, kind and value with certified target constants. At dependency
reads join the producer's retained source facts with the consumer's original
compiler declaration before granting a typed import.

RustSourceTypes carries an exact emitted-public-constant map keyed by original
RustDeclarationId. RustConstantValue is a closed descriptive enum: Bool(bool),
I32(i32), I64(i64), F64Bits(u64), Char(char). The variant determines source kind;
there is no independently mutable kind/value pair. Floating bits preserve exact
representation without a new dependency; constructing descriptive facts does
not admit NaN constants or authenticate source analysis. Character values use
Rust char so an invalid Unicode scalar cannot inhabit that variant safely.
with_constants checks ownership, cross-inventory/field-owner collisions and
combined scalar budgets. Private/local/inherent folded constants are excluded
when no target declaration is emitted. Both adapters recollect and authenticate
the complete inventory from rustc before target reconciliation. Reconciliation
and foreign joins compare exact original values as well as source kinds.

Constant-only character crates must expose the character-aware metadata even
without character functions or record fields. Public aliases retain original
definition/owner identity, docs and visibility. Serialized descriptions cannot
manufacture dependency authority. Reserve metadata/output resources before
publication and preserve old non-character package bytes.
Omit the serialized constant source-facts section when its inventory is empty
so previously generated character-value packages also retain identical bytes.

## Evidence boundary

Reuse the completed full scalar-domain oracle; add independent integer truth
for named/computed constant contexts, boundary values and deterministic interior
samples. Prove certified target declarations and reads with strict separately
compiled native consumers. Java controls must recompile consumers after producer
changes to account for javac constant inlining. Detect actual narrowing,
replacement and wrong-value faults, including noncharacters and supplementary
scalars. Source probes must reject same-target-representation Char/I32 forgery,
wrong owners/values and invalid scalar metadata before atomic publication.

Require actual producer-change/restored-input Bazel evidence, exported examples,
full release/lint and broad review. This is partial constant parity only;
no runtime, renderer escape hatch or third-party dependency is introduced.
