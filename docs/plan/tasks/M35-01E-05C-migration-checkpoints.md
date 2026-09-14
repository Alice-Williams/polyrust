# M35-01E-05C — Commit and publish verified migration checkpoints

- Status: planned
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
