# M35-01E-02D — Source documentation and export metadata

- Status: complete
- Parent: [M35-01E-02](M35-01E-02-java-source-identity.md)
- Depends on: completed M35-01E-02C

## Goal

Preserve resolved Rust doc attributes through checked, owner-specific Java
documentation attachments, without executable source strings or forged origins.

## Ordered work

1. [M35-01E-02D-01](M35-01E-02D-01-java-doc-comments.md): opaque, safely encoded
   Java documentation text with independent lexical/native negative controls.
2. [M35-01E-02D-02](M35-01E-02D-02-source-documentation-metadata.md): validate bounded source module ancestry, export inventory and shared payload
   consistency across all source type/callable/field owners. Metadata remains
   non-authenticating; compiler analysis supplies authority in M35-01E-03.
3. [M35-01E-02D-03](M35-01E-02D-03-java-documentation-projection.md): derive typed documentation owners/attachments during file-item resolution.
   Keep the map in ResolvedJavaFileItem, and rely on existing post-link exact
   rederivation before certification. Source aliases must not duplicate docs.
4. Render only normalized comments at declaration/component/module locations;
   account for normalized text and total attachments in package resource checks.

## Definition of done and tests

- Crate/module/type/function/field doc attributes retain content and source order
  under the documented safe encoding and exact owner mapping.
- Shared ancestor docs appear once at their designated package owner. Conflicting
  roots, parents, ancestry, export inventories and duplicate owners reject.
- Hostile comment terminators, Unicode escapes, line endings, control characters,
  doc tags/markup and non-ASCII text cannot produce executable Java tokens.
- Exact/one-over document count, input text and normalized output budgets reject
  before certification; no unbounded per-owner expansion of shared metadata.
- Tampered/missing/extra resolved attachments reject by post-link rederivation.
- Generated source compiles under pinned Java 21 with warnings denied and its
  scalar behavior is unchanged; three renders and historical output are stable.
- Full migration/lint/docs gate and fresh independent review pass.

## Scope boundary

No custom Rust parser or new generic AST. The compiler still owns resolved doc
attributes, source identity and visibility. This checkpoint validates metadata
coherence and target-safe presentation, not source compilation independently.

## Completion evidence

All three child checkpoints are complete with fresh independent reviews.
Final full gate `1d9bfd30-4a69-4830-9406-7dca023ca0db` passed all 343 tests,
including Java's 248 unit/native cases and all lint/policy targets. Compiler
extraction and Java HIR mapping remain the next checkpoint, M35-01E-03.
