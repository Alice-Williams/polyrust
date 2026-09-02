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

## In-progress port

No port is currently in progress.

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

### [normalize-newline 5.0.0](ports/normalize-newline.md)

- Upstream: [sindresorhus/normalize-newline](https://github.com/sindresorhus/normalize-newline)
- Revision: `bc6982d73ebd62de3729435d9baf8731ca274f7a`
- License: MIT
- Typed API: overloads `normalizeNewline(string): string` and
  `normalizeNewline(Uint8Array): Uint8Array`
- Selection evidence: 88,373 npm downloads for the measured week ending
  29 August 2026.
- Required PolyRust gap: literal global replacement over immutable arbitrary
  bytes, including an explicit empty-needle boundary rule.
- Completion: M29; all 13 valid official assertions, 31 portable vectors,
  9,338 differential text inputs and 31,847 differential byte inputs, eight
  native generated-package gates, deterministic regeneration, and immutable
  provenance checks.

### [has-flag 5.0.1](ports/has-flag.md)

- Upstream: [sindresorhus/has-flag](https://github.com/sindresorhus/has-flag)
- Revision: `63fde682532a6e0bb155125d03a66989e0b0ce24`
- License: MIT
- Admitted typed API:
  `has_flag(flag: String, argv: List<String>) -> Bool`; upstream's omitted
  `argv` default is an effectful host-adapter boundary.
- Required PolyRust gaps: well-formed UTF-16 code-unit length and structural
  first-index lookup returning `Option<I64>`.
- Completion: M31; all 11 official assertions, 25 portable vectors, 42,273
  differential comparisons, eight native generated-package gates,
  deterministic regeneration, sanitizers, and immutable provenance checks.

### [split-on-first 3.0.0](ports/split-on-first.md)

- Upstream: [sindresorhus/split-on-first](https://github.com/sindresorhus/split-on-first)
- Revision: `d6bf86163df4e6490b134c303477644a52736997`
- License: MIT
- Typed input API: `splitOnFirst(string: string, separator: string)`
- Exact runtime result: empty list when no split exists, otherwise a two-string
  list; the upstream `[string, string?]` declaration does not include its own
  officially tested empty result.
- Selection evidence: 3,071,139 downloads of version 3.0.0 and 18,534,394
  package downloads for the measured week ending 29 August 2026.
- Required PolyRust gaps: literal first-substring lookup returning an explicit
  optional scalar index, total half-open slicing by scalar offsets, and
  allocator-safe C17 construction of the already supported `List<String>`
  result family.
- Version boundary: v4's later arbitrary JavaScript `RegExp` overload is not
  part of the pinned v3 API and remains deferred until a portable regex subset
  is specified.
- Completion: M32; all six official assertions, 32 portable vectors, 58,274
  differential comparisons, eight native generated-package gates,
  three-generation determinism, C allocation-failure/sanitizer proof, and
  immutable provenance checks.

### [@stdlib/math-base-assert-is-negative-zero 0.2.3](ports/stdlib-is-negative-zero.md)

- Upstream:
  [stdlib-js/math-base-assert-is-negative-zero](https://github.com/stdlib-js/math-base-assert-is-negative-zero)
- Revision: `766200b9eeea46b7f827ac7d63effa6bea65d896`
- License: Apache-2.0
- Typed API: `isNegativeZero(x: number): boolean`
- Selection evidence: 274,168 npm downloads for the measured week ending
  29 August 2026.
- Required PolyRust gap: exact IEEE-754 negative-zero classification, distinct
  from both ordinary equality and generic sign-bit testing.
- Admission boundary: the declaration accepts only `number`. Official
  JavaScript non-number calls remain invalid-type evidence outside `F64`.
- Completion: M33; 22 exact-bit portable vectors, 86,018 exact-bit
  differential inputs, all eight native generated packages, public consumers,
  sanitizers, deterministic regeneration, and immutable provenance checks.

## Deferred candidates

- `juliangruber/balanced-match` is MIT licensed and attractive for a later
  parser case, but its current API accepts arbitrary JavaScript `RegExp` values.
  Native regex engines in the supported targets do not share one complete
  language or matching model. It remains deferred until PolyRust specifies an
  honest portable regex subset or implements one common engine.
- `sindresorhus/strip-final-newline` 4.0.0 is MIT licensed and has an attractive
  string/`Uint8Array` API, but its documented binary result is a mutable
  `subarray` view of the input. That observable aliasing conflicts with
  PolyRust's normative immutable `Bytes` value semantics.
- `inspect-js/is-negative-zero` 2.0.3 is MIT licensed and widely used, but its
  declaration accepts `unknown` and its official API includes dynamic strings,
  objects, arrays, functions, booleans, `null`, and `undefined`. A numeric-only
  port would be incomplete. M33 instead uses stdlib's number-specific API for
  the same reusable IEEE predicate.
