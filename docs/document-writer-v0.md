# v0 structured document writer

## Algebra

portable_codegen::Document is an immutable, cloneable document tree with these
target-neutral nodes:

- empty and text;
- soft line, which is a space in flat mode and newline in broken mode;
- hard line, which always breaks;
- concatenation and separator-based join;
- fixed-space indentation;
- width-selected group; and
- if-break selection for punctuation or other layout-only differences.

The writer has no identifier, keyword, escaping, import, or language syntax
table. Backends own those decisions and compose the resulting text as documents.
The same nodes can express indentation-sensitive or delimited layouts.

Document::text rejects every Unicode control scalar and returns TextError with
the scalar index. A backend that deliberately needs raw controls must construct
RawText explicitly. Raw CRLF and lone CR are normalized to LF during rendering.

## Layout

Groups are flattened when their Unicode-scalar width fits the remaining line;
otherwise their soft lines break. Width counts Unicode scalar values so it is
deterministic and independent of host locale. A token is never split merely to
meet the width.

Indentation applies to line nodes inside an indented document. FinalNewline has
three exact policies:

- Preserve retains the rendered ending;
- Always produces exactly one trailing LF; and
- Never removes trailing LFs.

All host output therefore uses LF, and repeated renders of the same document
and options produce equal text and statistics.

## Iterative safety and limits

Rendering and flat-fit simulation use explicit frame vectors rather than
recursive traversal. RenderLimits bounds:

- structural depth;
- total nodes visited, including fit simulation; and
- output bytes.

The default supported maximum depth is 4,096. Exceeding a limit returns
DepthLimit, NodeLimit, or OutputLimit without a partial success or process-stack
overflow.

RenderStats records total visits, output bytes, peak output String capacity, and
peak pending frames. These make benchmark memory behavior observable without a
platform-specific allocator.

## Verification and baseline

The focused suite covers flat and broken groups, conditional layout, nested
indentation, empty joins, long tokens, Unicode, normalized raw lines, every
final-newline policy, exact width boundaries, repeated determinism, all limits,
the supported 4,096-level depth, and both toy layout styles.

The optimized benchmark builds and renders 10,000 representative declaration
documents ten times at width 88. On the pinned Linux development image on
2026-08-31, its best observed baseline was:

    best_us=3641
    output_bytes=297780
    peak_output_capacity_bytes=425984
    peak_pending_frames=20003
    nodes_visited=140000

The baseline is observational, not a timing assertion. Re-run it with:

    cargo bench -p polyrust-codegen document
