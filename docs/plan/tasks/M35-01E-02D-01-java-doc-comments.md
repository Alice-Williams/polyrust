# M35-01E-02D-01 — Opaque Java documentation text

- Status: complete
- Parent: [M35-01E-02D](M35-01E-02D-java-source-documentation.md)
- Depends on: completed M35-01E-02C

## Implementation contract

- JavaDocComment owns normalized text behind private fields. Its only public
  construction path encodes untrusted attribute text; no raw constructor.
- Normalize CRLF/CR to LF. Encode backslashes so Java's early Unicode-escape
  processing cannot introduce tokens or line terminators. Encode stars so an
  attribute cannot close or open a block comment.
- Encode HTML markup and doc-tag introducers as text, with control characters
  represented explicitly. Preserve ordinary Unicode and line order.
- Structural rendering owns the fixed comment delimiters and line prefixes;
  source attributes never supply template syntax or preformatted Java.
- Expose encoded byte length for the subsequent package resource accounting.

## Definition of done and tests

- Exact expected encodings cover delimiters, repeated backslashes, literal
  Unicode-escape spellings, CR/LF variants, markup, tags, controls and Unicode.
- Deterministic adversarial input corpus always has no raw backslash, star or
  carriage return in encoded payloads and retains expected line boundaries.
- Native Java 21 compiles and executes fixtures containing the encoded corpus;
  raw malicious controls independently demonstrate that the harness would catch
  token injection or lexical failures without encoding.
- Rust/Bazel lint, focused Java tests and full gate pass with clean review.

## Scope boundary

This leaf proves lexical comment safety only. Source ownership, package budgets
and production attachment/render integration remain in its parent; merely
constructing a JavaDocComment does not certify a package or prove rustc analysis.

## Evidence

- Full container gate `5783eb4d-f89d-44d6-ab4d-8a68632805a3`: 343/343 tests
  passed, including Java native injection controls, private-field compile-fail
  test, Rust/Bazel lint, historical backends and C/compiler regression tests.
- Fresh Sol Extra High read-only review found no core errors. It checked the
  private construction boundary, exhaustive Unicode corpus and the independently
  failing raw comment/Unicode-escape injection controls.
