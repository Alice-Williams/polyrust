# M35-03A-02F-02B-04 — Compiler public constants and constants-only packages

- Status: planned
- Parent: [public constants](M35-03A-02F-02B-public-constants.md)
- Depends on: M35-03A-02F-02B-03

## Contract

Implement the [checked source contract](../../specification/typed-generation/rust-public-constants.md).
Add private compiler inputs and executable PublicConstants/PublicConstantReads
bindings. Separate package registration from actual function Readers; admit
constants-only roots without fake functions. Register public declarations before
reads and preserve private/local folding and all public export bindings/docs.

## Definition of done and tests

- Real Rust/C/Java single-crate constants-only and mixed examples agree with
  independent exact values for bool/i32/i64 and computed constants.
- Actual mapping probes check declaration/reference identity, value/type,
  visibility, doc ownership and production-byte equality.
- Both new slots have per-backend missing/duplicate/wrong capability/context/
  output/input/private-construction compile-negative controls.
- Public aliases and private same-name declarations are distinguished;
  unsupported/invalid/generic/borrowed constants reject atomically.
- Replace old public-constant rejection only with positive equivalent evidence;
  preserve all read/local regressions. Full gate, review, examples and push.
