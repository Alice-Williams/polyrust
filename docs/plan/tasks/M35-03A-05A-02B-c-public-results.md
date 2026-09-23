# M35-03A-05A-02B — C public scalar-result nominal ABI

- Status: planned
- Parent: [C results](M35-03A-05A-02-c-results.md)
- Depends on: [private transport](M35-03A-05A-02A-c-private-results.md)
- Specification: [C17](../../specification/typed-generation/languages/c/rust-scalar-results.md)

## Contract

Issue certified aggregate API identities, import their exact complete layout
into consumer registries and preserve original defining headers through relay
packages. Only then admit result-bearing public signatures. No reconstruction
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
