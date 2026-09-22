# M35-03A-02V — Signed-infinity binary64 constants

- Status: complete
- Parent: [scalar parity](M35-03A-02-scalar-parity.md)
- Depends on: [finite constants](M35-03A-02U-finite-f64-constants.md)
- Specification: [shared](../../specification/typed-generation/rust-infinite-f64-constants.md)

## Contract and order

Admit exact positive/negative infinity through the existing constant capabilities
without weakening FiniteBinary64, admitting NaNs or adding copied runtimes.
Use typed standard C/Java constants and preserve original producer authority.

1. [02V-01 — Independent oracle](M35-03A-02V-01-constant-oracle.md) — complete; 1,004 release/lint tests pass and whole-scope/hardening reviews are clean.
2. [02V-02 — C foundation](M35-03A-02V-02-c-constants.md) — complete; all 1,005 release/lint tests pass and broad independent review is clean.
3. [02V-03 — Java foundation](M35-03A-02V-03-java-constants.md) — complete; all 1,006 release/lint tests pass and broad independent review is clean.
4. [02V-04 — Checked source integration](M35-03A-02V-04-compiler-constants.md) — complete; all 1,012 release/lint targets pass and broad independent review is clean.

Each checkpoint has its own full Linux release/lint gate, fresh independent
review and commit/push. Preserve old output/WIP. Source admission follows
both target foundations. NaN constants and wider parity remain separate.

The completed source proof observes 39 original Rust reads and 66 target
observations/configuration, with exact signs, original aliases/docs/privacy,
typed authority and atomic rejection controls, and three compiling value faults.
Actual producer-sign mutation rebuilds seven affected actions, preserves the
independent producer and restores original hashes plus a cached passing test.
Real source-owned C/Java packages are exported; old output/WIP is unchanged.
