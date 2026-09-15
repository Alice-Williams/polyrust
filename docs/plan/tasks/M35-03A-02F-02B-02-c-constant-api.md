# M35-03A-02F-02B-02 — Certified C constant declarations and imports

- Status: in-progress
- Parent: [public constants](M35-03A-02F-02B-public-constants.md)
- Depends on: M35-03A-02F-02B-01

## Contract

Implement the [C constant specification](../../specification/typed-generation/languages/c/rust-public-constants.md).
Reuse const-qualified registered objects and paired declaration/definition
nodes. Add bounded source certification, opaque producer/import witnesses,
complete export collision checks, typed reads and resource obligations.

## Definition of done and tests

- Typed constants-only and mixed packages certify; missing/duplicate/mutated
  objects, qualifiers, owners, files, types, values and initializers reject.
- Strict independent GCC/Zig O0/O2 producer/consumer tests prove exact values;
  public const assignment must fail native compilation.
- Values-only and mixed imports deduplicate correctly; forged/retargeted
  dependency and export metadata reject. Keep all existing C gates enabled.
- Full isolated Bazel/lint gate, reviewed fixes, examples and dedicated push.
  HIR support remains pending child04.

## Ordered implementation

1. [02A — Owned readonly C scalar objects](M35-03A-02F-02B-02A-c-owned-constants.md) — complete.
2. [02B — Certified C constant imports](M35-03A-02F-02B-02B-c-imported-constants.md) — in progress.

Each child has its own native/negative evidence, full gate, review and checkpoint.
The parent is complete only when both owned and imported behavior are proved.
