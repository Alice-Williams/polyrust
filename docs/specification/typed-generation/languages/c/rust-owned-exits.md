# Consumer-visible structured ownership exits

- Status: complete; evidence-interface hardening only
- Plan: [M35-02B-03J](../../../../plan/tasks/M35-02B-03J-consumer-exit-projections.md)
- Parent: [owned values](rust-owned-values.md)

## Required projection

Every admitted whole-body or per-path ownership certificate exposes its actual
canonical HIR exit and authenticated normal MIR return location. The existing
SourceExit enum distinguishes a tail expression from an explicit return expression
and its value. Its expression references belong to the same compiler session and
body owner as the operation evidence. No strings, source offsets or debug names
stand in for these identities.

For tail-nested scopes, the exit is the innermost admitted value expression. The
existing scope projection retains the enclosing block chain, allowing structured
lowering to preserve the wrappers. Do not relabel an inner value as a root tail.
For branches, each outcome retains its own canonical source exit, even where the
compiler merges the normal MIR return into a shared block.

The exposed MIR Location identifies the Return terminator, including its actual
statement index. It is evidence, not an instruction to render MIR control flow.
The backend still emits the corresponding structured HIR tail/return and cleanup.

## Construction and consumers

Retain facts already established by canonical source containment and complete
normal-flow correspondence. Do not query optimized MIR, reparse source, or allow
a consumer to supply replacement exits/locations. All evidence fields remain
private and the public construction boundary remains query-only.

LinearOwnedBody retains ScopeEvidence's certified tail and its relation's normal
return. MultipleOwnedBody retains ScopeFacts' exit and its already-checked return,
including when embedded in explicit-return or branch wrappers. RecordOwnedBody
retains its tail/return distinction and the final return checked after its ordered
partial-field cleanup. Existing dedicated return, guarded, early, selection,
boxed-record and call-graph projections continue using the same compiler facts.

Ordinary consumer tests compare canonical expression pointers, source exit kind,
scope identity and the full MIR Return location. Their assertions must not access
private proof fields. Both branch outcomes and both tail/explicit forms require
coverage where admitted; a getter's existence alone is insufficient evidence.

## Scope boundary

This change neither admits new Rust forms nor enables C or Java heap generation.
It makes existing proof facts usable by the later typed C mapping. Clone mapping,
allocator behavior and native cleanup equivalence remain separate obligations.

## Implementation status

The three missing body projections and shared consumer checks are implemented.
The canonical Exit is retained from existing scope certification; normal returns
are retained from the existing complete MIR relation. All ten proof-family runtime
targets pass with the shared checker. The full isolated gate passed all 469 tests
across 599 targets, including exact privacy diagnostics. Two independent broad
reviews found no remaining core issues after the fixture repair; the linked task
records the reviewed tree and gate evidence. Target heap support remains separate.
