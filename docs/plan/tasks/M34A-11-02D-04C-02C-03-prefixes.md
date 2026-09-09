# M34A-11-02D-04C-02C-03 — Initialized buffer prefixes

- Status: planned
- Depends on: M34A-11-02D-04C-02C-02

## Goal

Prove ordered dynamic-buffer construction and safe partial reads/cleanup from
actual loop write coverage, then close parent 02C and allocated-storage 02.

## Definition of done

- Derive element/field coverage on every continuing actual loop path, including
  backedges and continues, and relate the prefix to the real counter phase.
- Complete-element writes grow only the proved prefix; joins intersect coverage.
  Skipped/conditional/reordered writes cannot establish a complete buffer.
- Early exits retain only actual initialized coverage. Release/reactivation and
  conflicting writes invalidate stale prefix and numeric observations.
- Parent definitions of done are re-audited; no remaining dynamic-storage
  obligation is deferred under a completed status.

## Tests and proof

- Runtime-length scalar/aggregate construction, guarded prefix reads and partial
  cleanup with zero, maximum-safe and nested-layout cases.
- Missing fields, skipped/reordered writes, continue/break/cleanup, wrong counter,
  stale count, join and expired/reactivated allocation controls.
- Full cached focused/tracked/release/lint/eight-target gates and uncapped
  independent review. Record exact evidence, close parents, commit and push.
