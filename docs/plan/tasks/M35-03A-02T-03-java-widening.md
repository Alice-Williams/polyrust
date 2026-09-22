# M35-03A-02T-03 — Java signed-widening foundation

- Status: complete
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

## Implementation and focused evidence

Tree 426d225d51bb3176c1a6ab9a2c351d521994af2c passes all seven selected widening
cases, d17217cd-0c20-49a6-b3f8-aee70b3ff9ff. The production dependency reader
admits only primitive Int to primitive Long Cast with matching result and Unary
precedence, and recursively charges/certifies the original operand. The existing
source-byte reader now charges the cast target and descends through its operand.
The structural renderer and compiler-source admission remain unchanged.

Private reader controls exhaust 14,000 input/target/result/precedence combinations,
including boxed and reference substitutions, independently of earlier AST checks.
Public certification separately checks annotations, wrong types, nested unsupported
operators and depth. An exact two-node visit-budget control fails with one node.
Original dependency calls require their actual registered certificate and correct
arity; independently recertified lookalike owners do not confer authority.

Actual three-owner packages cover both direct cast-of-call and materialized-local
forms. Their certified call heights increase from one to three, their real output
stays below the source-byte reservation, and they introduce no runtime, helper,
system call or import directive. Strict separate Java21 compilation and normal
plus interpreted JVM execution agree with 73,890 independent signed inputs and
both direct/forwarded outputs. Three safe compiling mutations (zero extension,
premature short narrowing and disconnected zero) match independent fault models
and disagree with truth in both forms. Original generated source bytes are checked
after all disposable mutations. The test harness has only an exact-path policy
exemption with adjacent-copy and production-path rejection controls.

Full release/lint evidence, independent review and old-output/WIP preservation
remain completion prerequisites. No legacy deletion or new source cast admission
is part of this foundation checkpoint.

Independent whole-scope Sol Extra High review of the focused tree is clean:
no substantiated core defect, required proof gap or optional finding. It traced
exact cast admission and the existing AST/link/resource visitors, checked the
type matrix and public controls, original dependency authority, recursive call
height/depth/visit accounting, source-byte charging, strict native compilation
and all fault models. BUILD data and exact policy exemptions were also checked.
No implementation change was requested; the full gate remains required.

## Completion evidence

Reviewed tree 426d225d51bb3176c1a6ab9a2c351d521994af2c passes all 969 Linux
release/lint targets, 013370a1-96c6-46bf-8809-f060f3d5b813 (111 executed,
858 cached). All 412 Java unit cases pass, including the seven new widening
cases. No implementation repair, disabled test or timeout relaxation was needed.

All 406 previous generated file hashes and all 38 unrelated WIP hashes match.
The scoped candidate is the reviewed implementation. Documentation-only closure
receives a final full cached gate and exact-index check before separate commit
and push. Both target foundations are now complete; checked Rust `as i64`
admission and source-level evaluation evidence follow in 02T-04.
