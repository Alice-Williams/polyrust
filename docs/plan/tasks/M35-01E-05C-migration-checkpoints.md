# M35-01E-05C — Commit and publish verified migration checkpoints

- Status: complete
- Parent: [M35-01E-05](M35-01E-05-java-native-proof.md)
- Depends on: M35-01E-05B

## Implementation contract

- Commit C and Java milestone completions separately with their milestone IDs.
  The committed contents must equal the reviewed, tested proposed trees.
- Use host Git only; do not copy credentials into the container. Push normally,
  never force-push. Reconcile any remote movement without overwriting user work.
- Preserve unrelated dirty work and staged content. Verify the remaining local
  diff against the pre-isolation inventory, including split shared files.
- Record final local commits and verified remote refs. Reuse existing scheduled
  CI monitoring if applicable rather than repeated short-interval polling.

## Definition of done and tests

- Separate milestone commits exist and their tree IDs match tested snapshots.
- A normal push succeeds, and read-only remote-ref inspection confirms its tip.
- No generated/cached artifacts or unrelated ownership work are committed.
- The final local tree remains usable and unrelated user changes are intact.
- Close E05 and its Java migration parent only after their implementation,
  review, example, tree-proof and publication requirements are genuinely met.

## Publication evidence

- C commit: `04aeb1b8d5c04668dd4bdb083e5d72f0c2b005dc` (M35-01D).
- Java commit: `adcaa044f3d41853325e6a2f94bb63b2d32be384` (M35-01E).
- Each commit's tree exactly equals its reviewed, tested E05B tree. They were
  created separately and pushed together using host Git without force-pushing.
  `git ls-remote --heads origin main` confirmed the Java commit on GitHub.
- All 24 preserved unrelated/shared working-tree files matched their recorded
  SHA256 hashes, including the original child-graph module registrations.
  Unfinished ownership work, its task record, unrelated examples and Python
  bytecode were not committed. The initially empty user staging index was
  preserved during preparation and refreshed only when publishing the commits.
- No container received credentials. No CI polling loop was started. This is
  local exact-tree and remote-ref evidence, not a claim that hosted CI has passed.

This post-publication documentation record is tested and committed separately
so the earlier exact implementation-tree evidence is not made self-referential.
