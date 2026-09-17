# M35-03A-02L-01 — Certified binary64 conditional target foundation

- Status: complete
- Parent: [absolute value](M35-03A-02L-floating-absolute.md)

## Contract

Use the existing CExpressions::conditional node. In the shared package profile,
admit exactly Bool condition and F64 then/else/result, separately from existing
integer promotion cases. Preserve sequencing, original callable authority,
ownership and dependency traversal. C scalar-call shape already traverses
conditionals; verify it retains all nested identities. Java's closed dependency
profile already traverses conditionals; prove exact Double branches/result,
Boolean condition under its expression verifier, and explicitly require Conditional
precedence in dependency body admission.
No source capability or renderer change in this checkpoint.

## Definition of done and tests

- Positive same-owner and separately imported-owner target ASTs certify and
  compile under GCC14/Zig C17 O0/O2 and Java21 strict lint.
- Select either input branch over both Boolean conditions, signed zeros,
  subnormals, finite boundaries, infinities and NaN categories. For non-NaNs,
  selected bits equal the selected original operand exactly.
- Nested zero/negative selection expresses the proposed absolute-value shape;
  detect swapped branches, omitted zero normalization and wrong condition.
- Calls stay in valid C full-expression positions. Record actual condition/
  operand evaluation traces; no hidden eager evaluation or duplicate calls.
- Incorrect condition/branch/result types and Java precedence reject; original
  imported authority cannot be replaced by spelling or forged references.
- Full Linux release/lint gates, unchanged existing source artifacts, explicit
  review decisions and clean independent review precede commit/push.

## Implementation and proof

- Exact implementation tree 7c8980260a244312c96ea2691554cf5213f9d9b5 passed
  Linux Bazel test //... //:release_gate: 822/822 test targets across 1,251
  targets, 124 executed and 698 cached, 545.629 seconds.
  Invocation: 6720209f-3c5e-46bf-b077-f2a3311a7f78.
- The preceding complete backend suites passed 790 C and 370 Java Rust tests
  (C's five capacity cases run in their separate release targets).
  Invocation: acb49593-3125-4e5f-9cc9-b6303a228ca2.
- Native proof selects four expressions in each of two original/imported
  owners over 278 binary64 inputs: 2,224 results per native configuration,
  GCC14/Zig C17 O0/O2 and Java21 strict lint. Five deliberate faults cover
  swapped branches, missing positive-zero normalization, wrong condition,
  dropped calls and duplicated calls. Value and trace oracles are independent.
- The initial gate exposed missing Java Conditional precedence validation
  and a test-only C fault's assumption about protected parameter names.
  Both were fixed; no tests were disabled. A review finding added exact
  scalar-call edge assertions for each conditional child, with missing-definition
  transitive rejection controls. Materialized receiver native proof is separate
  from the private walker's child-traversal proof.
- Sol Extra High library_import_review reviewed the corrections clean.
  Fresh independent binary64_targets_review found no core defects or required
  proof gaps. Optional general side-effecting Java branch-call and stage-specific
  diagnostic tests are deferred: this checkpoint's receiver is materialized
  before the conditional, and the existing rejection test already protects
  the required dependency-authority boundary. These are coverage extensions,
  not missing absolute-value requirements.
- All 138 files in the earlier 18 source bundles, both floating-negation
  bundles and both NaN-classification bundles remain unchanged.
- No source abs capability, runtime helper, renderer semantics or sequencing
  relaxation was added. Source integration belongs to child 02L-02.
