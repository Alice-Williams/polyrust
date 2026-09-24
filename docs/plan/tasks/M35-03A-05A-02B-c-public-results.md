# M35-03A-05A-02B — C public scalar-result nominal ABI

- Status: complete
- Parent: [C results](M35-03A-05A-02-c-results.md)
- Depends on: [private transport](M35-03A-05A-02A-c-private-results.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

1. [02B-01: owned public header/ABI](M35-03A-05A-02B-01-c-result-headers.md) — complete.
2. [02B-02: certified nominal imports](M35-03A-05A-02B-02-c-result-imports.md) — complete.

The final inactive-arm execution evidence promised below is tracked by
[02C](M35-03A-05A-02C-c-result-branches.md), now complete. Its measured native
traces and all 1,041 release/lint targets pass with clean independent review;
correct return values alone were not treated as execution evidence.

The first step must reject dependency API publication for every header carrying
an aggregate, even when the selected function is scalar. Otherwise merely
including its header could introduce unchecked tag-name collisions in consumers.
The second step must discharge that whole-header obligation before lifting it.

Issue certified aggregate API identities, import their exact complete layout
into consumer registries and preserve original defining headers through relay
packages. Only then admit result-bearing imported signatures. Owned public
signatures are separately certified in 02B-01. No reconstruction
from text names or caller-supplied layouts. Extend closure/capacity/dependency
proofs without weakening private transport or scalar imports.

## Definition of done and tests

Standalone headers and separately compiled producer/relay/consumer packages
agree under GCC14/Zig O0/O2 and UBSan. Reject forged imports, owner/field/layout
mismatches, missing/transitive headers, unsupported shapes and capacity excess.
Prove effects, value copies/returns, tag distinction and inactive-arm behavior
with compiling fault controls. Preserve old outputs/WIP; full Linux release/lint
and fresh review must pass before separate commit/push. Compiler admission waits
for 05A-04 even after this checkpoint.
