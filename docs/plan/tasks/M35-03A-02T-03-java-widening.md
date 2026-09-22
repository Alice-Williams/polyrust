# M35-03A-02T-03 — Java signed-widening foundation

- Status: planned
- Parent: [02T](M35-03A-02T-signed-widening.md)
- Depends on: [C foundation](M35-03A-02T-02-c-widening.md)
- Specification: [Java21](../../specification/typed-generation/languages/java/rust-signed-widening.md)

## Contract

Certify Cast with primitive Int operand, primitive Long target/result and Unary
precedence. Recursively certify its operand. Do not admit narrowing, boxing,
floating/reference casts or casts based only on their declared output type.

## Definition of done and tests

Separate strict Java21 producer, forwarder and client agree with independent
signed truth. Compiling zero-extension, premature narrowing and disconnected
results disagree. Reject all wrong primitive/reference/boxed/Boolean widths,
target/result disagreement and precedence variants. Preserve operand import,
arity, call-height, depth and source-byte gates, plus earlier negative shapes
except the newly admitted exact cast. Full Linux release/lint gate, independent
review and preservation checks pass before separate commit/push. Source admission
and the renderer remain unchanged.
