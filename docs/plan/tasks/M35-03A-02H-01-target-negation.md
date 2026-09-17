# M35-03A-02H-01 — Certified wrapping-negation target foundation

- Status: complete
- Parent: [wrapping negation](M35-03A-02H-wrapping-negation.md)
- Specifications: [C](../../specification/typed-generation/languages/c/rust-wrapping-negation.md),
  [Java](../../specification/typed-generation/languages/java/rust-wrapping-negation.md)

## Contract and implementation

1. Admit existing typed signed Negate in the C closed source profile and
   local-only callable effect traversal. Admit scalar conditional expressions,
   traversing their condition and both values; do not bypass numeric-flow proof.
2. C uses a materialized value x and x == MIN ? MIN : -x, normalized to the
   original exact width after C promotions. Existing path-sensitive numeric
   proof must reject any path that can actually negate MIN. No unsigned
   reinterpret cast or compiler overflow flag substitutes for proof.
3. Java uses primitive int/long Negate. Extend dependency admission and bounded
   source reservation, retaining exact operand/result types and traversing calls.
4. Test real typed producer/consumer certificates without changing compiler
   source admission, builder slots or manifest versions in this checkpoint.
   Split fixtures and native drivers into focused modules.

## Required proof and definition of done

- C I32/I64 guard shapes certify. Removing/inverting/changing the guard or
  guarding a different value rejects when MIN can reach negation. Direct
  unconstrained signed negate remains uncertifiable; proven safe literals work.
- C conditional/negation traversals retain function dependencies, original
  certificate authority, source bounds and numeric safety. Wrong callable
  signatures and missing/recertified producers remain rejected.
- Java int/long negation certifies, returns MIN for MIN, preserves result width,
  and accounts for nested calls/source bytes. Other numeric families are not
  enabled by these signature rules.
- Separately compiled generated producers and clients run with GCC/Zig O0/O2
  and Java 21 strict lint. Independent truth covers MIN, MIN+1, MAX, zero,
  +/-1, power-of-two neighbors and a deterministic broad sample for both widths.
  GCC undefined-behavior sanitizer also accepts all guarded cases.
- Typed shape and native mutation controls detect a removed/incorrect guard,
  changed operation or lost width. No custom runtime/support artifact appears.
- Full exact-tree Linux Bazel release, Rust/Bazel linters and fresh independent
  Sol Extra High review pass before the scoped commit and push.

This target foundation does not yet enable Rust wrapping_neg source.

## Review decisions and proof corrections

- The first focused run exposed missing structural C renderer cases after
  grammar admission. Negate and Conditional now print fully parenthesized typed
  children; no operator fragment or helper runtime is introduced.
- Accepted the unused-parameter fixture correction: safe literal negation
  explicitly discards its parameter rather than weakening inventory checks.
- Accepted the missing exact-width proof finding. A typed shape checker detects
  a deliberately omitted I32 normalization even though C ABI compatibility can
  legally accept Int at that return boundary and native values remain equal.
- Accepted the missing nested-call proof finding. Negate and all three
  conditional child positions now contain direct calls in independent tests;
  an absent leaf prevents both the nested caller and its transitive caller from
  acquiring closed-call evidence.
- Kept the local purity walker broader than the source admission grammar. It
  proves call closure, not rendering eligibility (as already true for Binary).
  The separate closed source profile still rejects pointer/record conditionals.
  Duplicating admission rules in that effect analysis would add a second policy
  surface without closing a demonstrated safety hole.
- Test-only handwritten native consumers are explicitly marked cfg(test), so
  the existing import policy stays enabled and unchanged.

- The fresh review independently confirmed the Java negative-literal token
  collision. Negate now parenthesizes its operand; Java 21 native fixtures cover
  MIN, MIN+1, -1, zero, one and MAX at both widths, including nested negation.
- Accepted the unary-precedence metadata contract finding. Dependency admission
  rejects mismatched unary metadata; structural rendering remains fully
  parenthesized and does not trust that metadata for syntax.
- Expanded native truth to both neighbors of positive and negative powers of
  two: 4,482 rows per width/target, plus the Java literal-expression fixture.

## Completion evidence (2026-09-17)

- Exact implementation tree: df34d46cd544909547e2e6d328a8f3bbe0511a4a.
- Linux dev-container Bazel: test //... //:release_gate, 1,140 targets,
  760/760 test targets passed (97 executed, 663 cached), 555.071 seconds.
  Invocation: d61b60d7-5030-46c2-9433-e61869798896.
  Rust formatting/Clippy, Bazel formatting and source-import policy remain enabled.
- Full backend suites: 777 C tests plus five separately gated C stress tests;
  356 Java tests. Focused gate independently ran five C and five Java tests,
  not a zero-test filter (2c982e0c-f6a2-4bb4-ac0c-1cd76acf8415).
- Native evidence: separately compiled producer/consumer owners, 4,482 rows at
  each width, GCC and Zig O0/O2, GCC UBSan and strict Java 21. Java additionally
  compiles/executes 24 literal/nested-negation cases. Wrong-operation and typed
  guard/width/authority/precedence controls are permanent regressions.
- Fresh Sol Extra High review wrapping_target_clean_review reported no findings
  on that exact tree after the documented corrections. No competing review
  builds or implementation delegation were used.
- All changed Rust files are under 500 lines. Existing unrelated ownership work
  was preserved. No generated artifact, runtime removal, compiler capability
  admission, manifest-version change or third-party dependency was added.
- Next checkpoint is M35-03A-02H-02; this receipt does not claim Rust source
  wrapping_neg is enabled.
