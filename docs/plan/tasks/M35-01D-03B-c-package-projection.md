# M35-01D-03B — Whole C package projection and resources

- Status: complete
- Depends on: [M35-01D-03A](M35-01D-03A-typed-file-imports.md)
- Parent: [M35-01D-03](M35-01D-03-c-public-packages.md)
- Contract: [public packages](../../specification/typed-generation/languages/c/rust-hir-public-packages.md)

## Goal

Authenticate a header/implementation package once as a complete registry and
derive exact per-file declarations, uses and resource requirements.

## Definition of done

- Shared immutable package authority with exact per-file units; no independent
  verification of incomplete registry fragments.
- Primary public declarations in the header, exact source definitions, private
  prototypes/layouts in the implementation and body-local placement for values.
- Reconstruct all files, dependency imports, guards, declaration/definition
  links, documentation and visibility before resource certification.
- Per-file and aggregate package/source budgets plus whole-definition call
  paths; declarations in headers do not create phantom executable frames.
- Structural header/source rendering only from certified packages.

## Implementation order

1. Separate immutable package authority (frozen registry and complete source
   inventory) from per-file projections. Derive per-file primary declarations
   separately from referenced bindings; parameters/locals belong to the source
   body even when their function's primary declaration belongs to the header.
   Keep single-unit behavior and keep paired admission closed during this refactor.
2. Admit exactly one public header plus one sibling implementation, in addition
   to the existing single-unit profile. Check the complete package before
   recursive contextual passes; header prototypes are external, implementation
   helpers/layouts remain private, and prototypes precede all uses through the
   checked header dependency. Install/check platform assertions through this
   package view. Do not introduce a new C syntax tree or duplicate registrations.
3. Resolve header-owned grammar/guards and per-file bindings from that authority.
   Reconstruct the complete shared package, not each registry fragment. Route
   docs using compiler export-module membership: no private ancestor docs in
   the public header and no duplicated primary docs on definitions.
4. Measure all definition frames and call edges as one graph. Account for header
   syntax conservatively without creating header frames; include/guard bytes
   count toward per-file and aggregate bounds. Only then admit complete paired
   packages to public rendering and replace the temporary rejection test with
   certified positive and forged-package negative controls.

## Tests and proof

- Positive two-file typed fixture and unchanged one-file controls.
- Reject missing/extra/foreign file owners, duplicate primary declarations,
  missing bodies, changed linkage, wrong source placement, private header leaks
  and forged imports/guards.
- Actual-AST aggregate resource boundary/one-over/missing-edge tests; strict
  native compile and frame evidence from exact certified bytes.
- Separate consumer includes the public header twice; exact imports and symbols.
- Relevant C/shared tests, mandatory linters and independent review are green.

## Completion evidence

- One immutable authority now owns the complete frozen registry and source
  inventory. Focused per-file projections distinguish primary declarations from
  used bindings; public-body parameters/locals remain implementation-owned.
- Closed paired grammar, header-owned platform assertions, checked guard
  grammar, exact typed file includes, compiler-export-based documentation
  routing and canonical path/role ordering are connected to shared verification.
  Dependency inventories are joined by CFileRef, not traversal position.
- Resource accounting is split into focused node, aggregate, syntax and call
  modules. Header syntax is charged to real definition frames, with no phantom
  header frame. Typed includes/guards, per-file and summed output bounds are
  checked before certification; actual formatter lengths are also checked.
- New tests cover exact paired declarations/import/guard/body ownership,
  reordered inputs, eleven forged package variants, private/public docs through
  certified rendering, aggregate node/comment boundaries and one-over cases,
  arithmetic overflow and rejection of underestimated output bounds.
- Native proof compiles exact certified files separately from duplicate-header
  consumers and requires the exact public external symbol set. A separate
  address-retained frame probe keeps the existing strict complete-frame oracle.
  Both execute at 1 MiB stack under GCC/Zig O0/O2 and GCC ASan/UBSan O0/O2.
  Missing optimized helper reports were observed and rejected before this
  explicit probe split; no missing-report allowance was added.
- Linux/Bazel `54e18ee3-3979-4ae2-b6b3-ad2f59db6855`: 6/6 targets pass,
  including ordinary C unit/native tests, C compile-fail, Clippy, rustfmt,
  typed-generation policy and renderer policy. No tests disabled; caches enabled.
- The broader pre-review regression gate also passed 21/21 targets
  (`b09bcf55-6482-4d4a-8825-5a59399cb18d`). Review then identified and closed:
  a private-record proof gap; export root/orphan-module coherence; and generated
  header shadowing of standard/transitive headers under external-consumer `-I`.
- Header shadowing was reproduced before repair: Linux/Bazel
  `28553dde-068b-4070-8fa5-aae45950c78e` passed 672 tests and failed the new
  certificate rejection for `stdint.h`. Headers now require the documented
  reserved `polyrust_<nonempty>.h` namespace. Native tests exercise scalar,
  private-record and independently named longest-admitted-header packages.
- Coordinated malformed export graphs shared by all owners are rejected before
  documentation routing. Root/ancestry/crate checks apply to every owner;
  exact reachable graph validation applies to paired routing once per crate.
  Finite cycles and unexpanded foreign edges have positive controls.
- A suggested shared-basename restriction was not adopted: the typed C layer
  has registered file identities, not a required Rust crate input. Independent
  sibling basenames are valid and linked by exact CFileRef. Crate-derived names
  belong to M35-01D-03C; the specification now makes that responsibility explicit.
- Linux/Bazel `031ebca3-b13f-4fe3-8e36-0a22404aab20`: all 675 ordinary tests
  and all capacity/shared/Java/compile-fail/linter gates passed; 20/21 targets
  passed overall. Source policy required the extracted handwritten consumer
  helper to be explicitly `cfg(test)`; this annotation was added without any
  production policy exception.
- Final Linux/Bazel regression `20eabe50-02b8-4e38-be0a-62d3cfdf1b41`:
  21/21 targets pass, including all C capacity partitions, shared/Java
  regressions, compile-fail contracts, mandatory linters, docs and policies.
- Follow-up and fresh blind Sol Extra High reviews both found no unresolved
  core defect. The blind review covered the complete B package, link, render,
  resource and native proof boundary, without treating planned C/D functionality
  as a B defect. Documentation gate `8f64823b-0117-4b8d-99ec-2355d4324086`
  passed after evidence/plan updates. No tests were disabled.
- Complete locally; continue M35-01D-03C. Pushes remain held for C/Java migration.
