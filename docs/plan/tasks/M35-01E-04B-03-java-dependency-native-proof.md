# M35-01E-04B-03 — Independent Java dependency-call proof

- Status: complete
- Parent: [M35-01E-04B](M35-01E-04B-java-imported-callables.md)
- Depends on: M35-01E-04B-02

## Implementation contract

- Use independently certified owner fixtures and separately authored consumer
  expectations. Do not use emitted Java or the same lowering routine as the oracle.
- Generate each owner exactly once and a consumer that imports opaque handles
  through its own scope. Include two owners with identical method/facade names,
  zero/four parameters, both scalars, direct calls and unused registrations.
- Compile the actual generated paths with Java 21 --release 21 -Xlint:all -Werror;
  execute boundary-value cases and prove private helpers/records cannot be called.
- Freeze a snapshot for full regression and fresh independent Sol Extra High
  review. Evaluate every finding; fix core defects and repeat with a fresh
  reviewer. Record reasoned rejections and distinguish optional feature requests.

## Definition of done and tests

- Native Java consumers pass; invalid private consumers fail for the expected
  access reason. Same-name owners do not require unused imports or invented names.
- Deliberate wrong scope/signature/owner/catalogue/resolved-name mutations fail
  before certification or publication; no test is disabled to achieve a pass.
- Rendering is byte-identical across repeat runs and reordered registrations.
- Full C/Java migration gates and a fresh clean independent review pass.
- Mark parent B complete only with this evidence. Compiler-authenticated crate
  graph joins and bundle publication remain E04C/E04D, not implicit completion.

## Evidence

- Focused Linux-container gate `2e1ff3d1-e0b2-4587-9984-fb0445a14a77`:
  all five targets pass, including Java unit/native tests, typed compile-fail,
  Rust formatting/Clippy and Bazel formatting. Java reports 290 passed,
  zero failed and zero ignored.
- Native owners compile separately at their certified paths with Java 21
  `--release 21 -Xlint:all -Werror`; an independent consumer executes 8,204
  boundary/seeded inputs, covering both Boolean values, zero/four-argument calls,
  same-spelled distinct owners and a private record implementation. Invalid
  private helper/record consumers fail specifically for private access.
- Exact catalogue/qualified-name mutations reject, and independently built
  consumer scopes with reordered registrations render identical bytes.
- Full migration/release gate `08de0334-1c70-4fd0-8dae-fe7116b77d23`:
  453 targets, all 363 tests pass. No tests disabled. Cached results are retained.
- Fresh Sol Extra High read-only reviewer `java_dependency_restart_review`
  completed the broad audit with no core correctness, authority, resource-safety,
  API-boundary, native-proof or material coverage findings. No optional changes
  were required. Independently checked the cited authority and test evidence;
  there are no findings to reject or defer. The earlier usage-interrupted review
  is not counted as completed or clean.
