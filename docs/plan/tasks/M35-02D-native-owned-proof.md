# M35-02D — Prove native cleanup and publish ownership checkpoint

- Status: planned
- Parent: [M35-02](M35-02-rustc-owned-values.md)
- Depends on: M35-02C

## Contract

Execute independent native Rust and generated C on the same boundary corpus for
construction, moves, clone, nested ownership, partial/conditional cleanup and
early returns. Instrument the selected allocation/deallocation boundary without
changing the production program's normal semantics or conflating test facts with
compiler authority. Publish real inspectable examples and precise limitations.

## Definition of done and tests

- Native results agree under pinned GCC/Zig O0/O2, ASan and UBSan.
- Explicit allocation/drop event assertions and a deterministic failure allocator
  prove the selected cleanup/failure contract, including initialization failure.
- Mutating away, duplicating or reordering required cleanup makes the appropriate
  oracle fail; native output equality alone is insufficient.
- Exact proposed commit trees exclude pending M34 work and pass the full release,
  compiler, C/Java, capability-negative, formatting, lint and documentation gates.
- Fresh broad review loops close all core findings. Commit/push only verified
  trees and record remote refs; do not retire old ownership checks before M35-03.
