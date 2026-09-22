# M35-03A-02T-02 — C signed-widening foundation

- Status: planned
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [oracle](M35-03A-02T-01-widening-oracle.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-signed-widening.md)

## Contract

Admit only the exact internal I32-to-I64 numeric-conversion shape needed for
lossless source widening. Preserve recursive operand certification, original
dependency authority, range/loss tracking and pinned platform proof.

## Definition of done and tests

Certify separately compiled producer, forwarder and external client. GCC14/Zig
O0/O2 and GCC UBSan agree with the independent oracle; headers compile alone.
Safe compiling zero-extension, premature-narrowing and disconnected-result
controls disagree. Reject incorrect source/result widths and unsigned/floating
substitutions; preserve existing guarded conversions and ordinary negative tests.
Exercise nested unsupported operands, original-call authority and resource bounds.
No new source admission, renderer special case or runtime. Full gate, independent
review and previous-package/WIP preservation precede a separate commit/push.
