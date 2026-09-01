# Real-world compatibility ports

This track grows the portable language from evidence supplied by existing,
widely used software. It is not a source-to-source translation benchmark:
generated code may use different structures as long as its public behavior is
the same.

## Admission and completion policy

A candidate must use an MIT or Apache-2.0 license at the pinned revision. Before
implementation, record its immutable commit, license, public typed API, official
tests, and any semantic boundary that cannot be represented honestly.

Only one port may be in progress. A port is complete when:

1. its valid typed input domain is represented without silently dropping an API;
2. official vectors and a broader differential corpus agree with the pinned
   upstream implementation;
3. the reference evaluator agrees with generated Rust, TypeScript, Python, and
   Go native tests;
4. every generated package passes native formatting, static checks, compilation,
   and tests from clean output;
5. regeneration is byte-identical; and
6. provenance, gaps filled, and reproduction commands are documented.

## Completed ports

### [escape-string-regexp 5.0.0](ports/escape-string-regexp.md)

- Upstream: [sindresorhus/escape-string-regexp](https://github.com/sindresorhus/escape-string-regexp)
- Revision: `cbc42403142c96923b482604e1f3d627b1956aff`
- License: MIT
- Source language: JavaScript with a TypeScript declaration
- Typed API: `escapeStringRegexp(string: string): string`
- Selection evidence: 597 GitHub stars and 361,042,967 npm downloads during the
  measured week on 1 September 2026.
- Required PolyRust gap: literal, global, non-overlapping string replacement
  with replacement text interpreted literally.
- Completion: M17; 18 shared portable vectors, 3,750 differential inputs, four
  native generated-package gates, deterministic regeneration, and the complete
  repository release gate pass.

### [trim-newlines 5.0.0](ports/trim-newlines.md)

- Upstream: [sindresorhus/trim-newlines](https://github.com/sindresorhus/trim-newlines)
- Revision: `6980540ee683a660fd82cb1bda37bf1ebd989179`
- License: MIT
- Source language: JavaScript with TypeScript declarations
- Runtime API: `trimNewlines`, `trimNewlinesStart`, and
  `trimNewlinesEnd`, each `String -> String`
- Selection evidence: 49 GitHub stars, 583 npm dependents, and 16,782,947
  weekly npm downloads measured on 1 September 2026
- Required PolyRust gap: linear trimming from either boundary by an explicit
  set of Unicode scalar values
- Completion: M18; 31 shared portable vectors, 107,851 differential inputs,
  323,553 three-function comparisons, four native generated-package gates,
  deterministic regeneration, and the complete repository release gate pass.

### [slash 5.1.0](ports/slash.md)

- Upstream: [sindresorhus/slash](https://github.com/sindresorhus/slash)
- Revision: `98b618f5a3bfcb5dd374b204868818845b87bb2f`
- License: MIT
- Typed API: `slash(path: string): string`
- Selection evidence: 340 GitHub stars, 3,884 npm dependents, and 89,686,202
  weekly npm downloads measured on 1 September 2026
- PolyRust coverage: reuses prefix checks, if expressions, and global literal
  replacement without a project-specific intrinsic
- Completion: M19; 15 portable vectors, 55,994 differential paths, four native
  generated-package gates, deterministic regeneration, and the complete
  repository release gate pass.

## Deferred candidate

`juliangruber/balanced-match` is MIT licensed and attractive for a later parser
case, but its current API accepts arbitrary JavaScript `RegExp` values. Native
regex engines in Rust, TypeScript, Python, and Go do not share one complete
language or matching model. It remains deferred until PolyRust specifies an
honest portable regex subset or implements one common engine; a literal-only
partial port would not satisfy this track's completion policy.
