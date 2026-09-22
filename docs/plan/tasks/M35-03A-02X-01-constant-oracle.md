# M35-03A-02X-01 — Independent character constant oracle

- Status: complete
- Parent: [02X](M35-03A-02X-character-constants.md)
- Depends on: [character source integration](M35-03A-02W-04-compiler-characters.md)
- Specification: [shared](../../specification/typed-generation/rust-character-constants.md)

## Contract

Add integer-only expectations for native compile-time Rust character values,
independent of target code and Unicode databases. Reuse the full-domain scalar
oracle without duplicating its exhaustive runtime corpus. Cover all existing
19 boundaries plus deterministic interior samples and explicit module, local,
inherent, alias and computed constant contexts.

## Definition of done and tests

Original Rust O0/checks-on and O2/checks-off agree exactly with independent
scalar numbers. Values must actually originate from const declarations, not
runtime conversions disguised as constant proof. Native faulty columns for
byte/UTF16 narrowing, replacement and changed value match independent faulty
models and disagree with truth. Reject malformed oracle inputs/protocol data.
Pin corpus counts and boundary membership; retain NUL, noncharacters,
unassigned/private-use and supplementary values. Add focused Bazel native,
Clippy and rustfmt targets. Full release/lint, fresh broad review and unchanged
prior output/WIP are required before a separate commit/push. No production
constant admission changes belong in this checkpoint.

## Verification evidence

The native fixture's static array is initialized by a const function; twelve
additional module/private/local/inherent/alias/computed values are const reads.
Its 4,127 observations cover all 19 established boundaries plus 4,096 distinct
interior samples. Four actual fault columns cover byte/code-unit truncation,
ASCII replacement and a changed value. Four invalid const conversions reject.
Both optimization/check profiles match independent integer expectations.
Invalid oracle inputs and six malformed native-output controls reject.

All five focused native/Clippy/rustfmt/Bazel/docs checks pass. Implementation
tree 8fe2cf35746e9abf3e10b3f68ec939441a96665c passes all 1,027 full Linux
release/lint targets (ten executed, 1,017 cached), invocation
b4e17ec6-ea6e-4fa5-8fe8-e777a88c3879. Fresh broad Sol Extra High review
of that exact 15-file checkpoint found no actionable core defects.

The reviewer noted optionally tagging each invalid conversion's protocol row.
This is not a missing correctness check for this fixed corpus: the four inputs
are explicit in the const initializer, exact row counts/results are checked,
and the preceding full-domain oracle already verifies every surrogate plus
out-of-range inputs. No dynamic input-to-row association is claimed here.
Keep this optional extension out of the completion criteria; future dynamic
protocols must preserve their input identities explicitly.

Closure removes one unused Python import and clarifies that excluded aliases
mean source type-alias uses, not the supported constant aliases. It receives
the full Linux release/lint gate again before a separate commit/push. No target
or compiler-source production admission has changed. C/Java foundations and
checked constant integration remain separate unfinished checkpoints.
All 530 previous generated output hashes and 38 unrelated WIP hashes are
unchanged at closure. Generated artifacts are not committed.
