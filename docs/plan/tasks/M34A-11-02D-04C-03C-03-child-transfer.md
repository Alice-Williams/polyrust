# M34A-11-02D-04C-03C-03 — Subtree transfer and destruction

- Status: planned
- Depends on: M34A-11-02D-04C-03C-02

## Goal

Preserve entire owned subtrees through actual transfer and exactly-once cleanup.

## Definition of done

- Child attach/detach and local/subtree move require actual source clear and
  exclusive destination. Preserve numeric/allocation identity and owning links,
  retire external aliases throughout the subtree without restoring raw copies.
- Detaching required children marks parents partial/dismantling; no incomplete
  parent escapes or commits. Release only after children are accounted for.
- Active-arm changes, scope/return/cleanup, joins and solver fallback retain all
  descendant obligations and reject skipped/double release.
- Default original allocator remains authenticated; helper summaries and
  incoming custom allocator contracts remain the already specified later stages.

## Tests and proof

- Nested moves, parent/child detach/drop chains and active-union cleanup.
- Stale descendant aliases, shallow copies, missing resets, crossed scopes,
  live-child parent release, wrong arm, skipped/double child and early exits.
- Exact numeric retention, private transition/join controls, full cached
  focused/tracked/release/lint/deterministic gates and uncapped review.

## Commit gate

Record evidence, commit and push; continue rollback/clone proof in 03C-04.
