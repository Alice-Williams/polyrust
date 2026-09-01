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
3. the reference evaluator agrees with generated Rust, TypeScript, JavaScript,
   Python, Go, Java, C++, and C native tests;
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
- Completion: M17 plus M21/M22 expansion; 18 shared portable vectors, 3,750 differential inputs, eight
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
- Completion: M18 plus M21/M22 expansion; 31 shared portable vectors, 107,851
  differential inputs, 323,553 three-function comparisons, eight native generated-package gates,
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
- Completion: M19 plus M21/M22 expansion; 15 portable vectors, 55,994 differential paths, eight native
  generated-package gates, deterministic regeneration, and the complete
  repository release gate pass.

### [strip-bom 5.0.0](ports/strip-bom.md)

- Upstream: [sindresorhus/strip-bom](https://github.com/sindresorhus/strip-bom)
- Revision: `b80d7bc94e79b4744d92a2dc6328c91d9afe9775`
- License: MIT
- Typed API: `stripBom(string: string): string`
- Selection evidence: 114 GitHub stars, 2,056 npm dependents, and 147,115,411
  weekly npm downloads measured on 1 September 2026
- Required PolyRust gap: exact removal of one leading substring plus valid Go
  serialization of U+FEFF and collision-free Go local identifiers
- Completion: M20 plus M21/M22 expansion; 18 portable vectors, 55,991 differential strings, eight
  native generated-package gates, deterministic regeneration, and the complete
  repository release gate pass.

### [html-escaper 3.0.3](ports/html-escaper.md)

- Upstream: [WebReflection/html-escaper](https://github.com/WebReflection/html-escaper)
- Implementation revision: `c6e2b50d7b6f486afb3ddc92bfcfec89857b75d7`
- Type declaration revision: `cd61c555bfc93e985b313263a42ed78074570d08`
- License: MIT
- Typed API: `escape(str: string): string` and
  `unescape(str: string): string`
- Selection evidence: 112 GitHub stars and 88,831,960 npm downloads for the
  measured week ending 29 August 2026.
- Required PolyRust gap: ordered simultaneous literal replacement whose output
  is never recursively rescanned.
- Completion: M23; all four official assertions, 42 portable vectors, 108,498
  differential function/input comparisons over 54,249 unique strings, eight
  native generated-package gates, deterministic regeneration, and the complete
  repository release gate pass.

### [truncate-utf8-bytes 1.0.2](ports/truncate-utf8-bytes.md)

- Upstream: [parshap/truncate-utf8-bytes](https://github.com/parshap/truncate-utf8-bytes)
- Implementation/license revision: `4212839ea184e74fb81f1e4e633e1db794ebe4f4`
- Type declaration revision: `451dc8fc19383bc12af59522020e571957f1684e`
- License: MIT (dual-licensed upstream)
- Typed API: `truncate(string: string, byteLength: number): string`
- Selection evidence: 8,395,462 npm downloads for the measured week ending
  29 August 2026.
- Required PolyRust gap: Unicode-scalar-safe truncation by a UTF-8 byte budget,
  retaining fractional, infinity, and NaN behavior from the typed JavaScript
  `number` input.
- Completion: M25; 30 portable vectors, 25,303 differential comparisons over
  486 strings including the complete official corpus, eight native generated
  package gates, deterministic regeneration, immutable provenance checks, and
  the complete repository release gate pass.

### [parse-ms 3.0.0](ports/parse-ms.md)

- Upstream: [sindresorhus/parse-ms](https://github.com/sindresorhus/parse-ms)
- Revision: `49dab09236deeea5d2c082182e2c73e7a79763a8`
- License: MIT
- Typed API: `parseMilliseconds(milliseconds: number): TimeComponents`
- Selection evidence: 24,963,960 weekly npm downloads and 171 dependents
  measured on 1 September 2026.
- Required PolyRust gaps: truncation-toward-zero, exact nested-F64 portable
  expectations, typed record results, and lossless IEEE-bit transport.
- Completion: M27; 30 portable vectors, 10,105 differential inputs and 70,735
  exact field comparisons, eight native generated-package gates, deterministic
  regeneration, immutable provenance checks, and the complete repository
  release gate.

### [is-fullwidth-code-point 3.0.0](ports/is-fullwidth-code-point.md)

- Upstream: [sindresorhus/is-fullwidth-code-point](https://github.com/sindresorhus/is-fullwidth-code-point)
- Revision: `80e5e314d86e5f76bd1b0573aa9d33e615a372db`
- License: MIT
- Typed API: `isFullwidthCodePoint(codePoint: number): boolean`
- Selection evidence: 265,568,331 npm downloads for the measured week ending
  29 August 2026 and 51 GitHub stars.
- Required PolyRust gap: an explicit IEEE-754 NaN predicate; the complete range
  classifier otherwise composes existing comparisons and Boolean operations.
- Completion: M28; 89 portable vectors, 22,409 differential inputs, eight
  native generated-package gates, deterministic regeneration, and immutable
  provenance checks.

## Deferred candidate

`juliangruber/balanced-match` is MIT licensed and attractive for a later parser
case, but its current API accepts arbitrary JavaScript `RegExp` values. Native
regex engines in Rust, TypeScript, JavaScript, Python, Go, and Java do not share one complete
language or matching model. It remains deferred until PolyRust specifies an
honest portable regex subset or implements one common engine; a literal-only
partial port would not satisfy this track's completion policy.
