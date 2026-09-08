# M34A-11-01R — Repair shared admission findings from the C design review

- Status: complete
- Depends on: M34A-11-01

## Goal

Close demonstrated shared-contract holes before extending C contextual checks.
Keep these repairs distinct from advertising C capabilities or C certification.

## Definition of done

- Record values retain their exact recursive field type list. Equality is
  available only when every field is recursively equatable; interface-containing
  records remain valid values but cannot gain equality through record erasure.
- Preserve exact field brands, constructor arguments, method receivers and
  implementation bindings. Do not fix the hole by rejecting interface storage.
- Reconcile ordinary IEEE equality, portable expectation equality (all NaNs
  compare as expected values; signed zero remains distinct), and exact raw-bit
  representation auditing in the C and shared specifications.
- Evaluate all independent design findings with evidence. A proposed semantic
  change is not automatically accepted because it appears in review feedback.

## Tests and proof

- Demonstrate the equality compile-fail regression fails before the repair
  because Rust incorrectly accepts it, then passes after field-derived typing.
- Positive nested equatable record, constructor, field, implementation and
  interface-containing record storage checks; negative direct/nested equality,
  inequality and list-search compile tests; dynamic checker rejection agrees.
- Focused build/check tests, Rust doctests, linters, full tracked/release and
  evaluator/eight-target conformance gates in the Linux container.
- Record the separate portable-test comparator follow-up if Java requires a
  change; this task cannot claim that an untested Java repair is completed.

## Review disposition

The immutable dc55311 review identified unconditional `RecordValue:
TypedEquatable` although the checker rejects recursive interface equality.
Accepted: exact field type lists must reach the value marker.

The review also found the C grammar domain unspecified and interface equality
wording contradictory. Those documentation defects were corrected in 2e4c50b.

The evaluator/Java test comparator mismatch is accepted as a consistency defect.
The suggestion to make all portable NaN expectations payload-exact is rejected:
checker-v0, parse-ms and M27-02 explicitly establish NaN-class equality for
portable tests. Changing that would alter existing semantics unnecessarily.
Raw-bit preservation still requires separate exact representation tests.

## Commit gate

Record failing controls, successful reruns and any remaining follow-up explicitly.
Commit and push with M34A-11-01R. Keep M34A-11 open and C compliance Fail.

## Checkpoint evidence (2026-09-08)

- Pre-fix `7d68c201-5649-48b5-beb9-563f2309d812`: both direct and recursively
  nested interface-record equality examples compiled, deliberately failing
  their `compile_fail` expectations. An initial nested control used a wrong
  helper name; it was corrected before counting this counterexample.
- `33aee482-3345-4ffa-8a74-054f17e40cc0`: builder unit/compile-fail and
  Clippy checks pass after field-derived typing, including Unit/alias controls.
- `81fd835f-108c-4ae0-9bbd-417e4bad31f7`: all 436 tracked rules build and
  all 311 tests pass with normal caching.
- `7256de0f-a6a2-4922-9dde-e5484e4a2bea`: all 248 release tests pass.
- `39e6b79b-36dc-44fa-a7ac-d99cafd0f04f`: 50 cases and one portable test;
  evaluator/eight-target agreement and repeated manifests pass.
- Supplementary Linux `cargo +1.98.0 test -p polyrust-build --all-features
  --locked`: 28 unit tests, 48 compile-fail doctests and one positive doctest.

Record values and method witnesses retain exact invariant field type lists.
Sealed recursive field evidence excludes interfaces, including through aliases;
empty records, Unit, transparent equatable aliases and nested concrete records
remain comparable. Five new focused unit tests and seven compile-fail examples
cover the repair; the method-receiver negative has a valid positive twin.
Dynamic ListContains/ListIndexOf now enforce the same recursive exclusion as
Equal/NotEqual. No existing valid interface storage/projection is removed.

The complete C design review is recorded in M34A-11-00R. Java's distinct-NaN
portable expectation mismatch remains explicitly open in M34A-10AB, not hidden
by these green gates. The earlier C foundation 2e4c50b passed all eight hosted
jobs in run 34216745710.
