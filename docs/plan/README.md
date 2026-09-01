# Engineering plan

This plan turns the portable code-generation feasibility study into reviewable,
dependency-ordered work. Each milestone has a separate file under
`milestones/`; its exit evidence becomes a permanent gate for later work.

## How to use this plan

1. Work on the earliest unblocked milestone, except for explicitly independent
   research or test preparation.
2. Set its status to `in-progress` before implementation.
3. Keep outcome, checklist, required evidence, and scope boundary visible in the
   milestone file.
4. Cite the milestone ID in commits.
5. Commit and push after each completed milestone.
6. Mark a milestone `complete` only after every exit criterion has evidence.

Valid statuses are `planned`, `in-progress`, `blocked`, and `complete`.

## Architecture and sequencing

- [Feasibility and risk analysis](analysis.md)
- [Phases, dependency graph, and release cuts](compatibility.md)
- [Product charter](../charter.md)
- [Portable language map](../portable-language.md)
- [Technical architecture](../architecture.md)
- [Decision record template](../adr/0000-template.md)

The critical path is:

`Linux/Bazel baseline -> unchecked IR -> checker -> evaluator/builder -> backend contract -> Rust/Go proof -> remaining targets -> conformance`

All authoring frontends lower to the same unchecked IR. All safe backends accept
only checked programs. Rust output is required and follows the same backend
contract as every other target.

## Milestones

### Phase 0 — Reproducible foundation

- [M00 — Linux/Bazel development environment](milestones/M00-development-environment.md) — complete

### Phase 1 — Semantic spine

- [M01 — Workspace boundaries](milestones/M01-workspace.md) — complete
- [M02 — Portable IR](milestones/M02-portable-ir.md) — complete
- [M03 — Diagnostics](milestones/M03-diagnostics.md) — complete
- [M04 — Resolver and checker](milestones/M04-checker.md) — complete
- [M05 — Reference evaluator](milestones/M05-evaluator.md) — complete
- [M06 — Rust builder frontend](milestones/M06-rust-builder.md) — complete

### Phase 2 — Backend platform

- [M07 — Structured document writer](milestones/M07-document-writer.md) — complete
- [M08 — Backend API and manifests](milestones/M08-backend-api.md) — complete
- [M09 — CLI and safe output](milestones/M09-cli-output.md) — complete

### Phase 3 — Required target backends

- [M10 — Rust backend](milestones/M10-rust-backend.md) — complete
- [M11 — TypeScript backend](milestones/M11-typescript-backend.md) — complete
- [M12 — Python backend](milestones/M12-python-backend.md) — complete
- [M13 — Go backend](milestones/M13-go-backend.md) — complete

### Phase 4 — Cross-language proof

- [M14 — Differential conformance](milestones/M14-conformance.md) — complete

### Phase 5 — Usability and release

- [M15 — Examples and extension proof](milestones/M15-examples-extensions.md) — complete
- [M16 — CI and release gate](milestones/M16-ci-release.md) — complete

### Phase 6 — Real-world compatibility

- [M17 — escape-string-regexp equivalence port](milestones/M17-escape-string-regexp.md) — complete
- [M18 — trim-newlines equivalence port](milestones/M18-trim-newlines.md) — complete
- [M19 — slash equivalence port](milestones/M19-slash.md) — complete
- [M20 — strip-bom equivalence port](milestones/M20-strip-bom.md) — complete

### Phase 7 — Target expansion

- [M21 — JavaScript derivative and Java backend](milestones/M21-javascript-java.md) — complete
- [M22 — C++ and C backends](milestones/M22-cpp-c.md) — in progress
  - [M22A — C++20 backend checkpoint](milestones/M22A-cpp-backend.md) — complete
  - [M22B — C17 backend](milestones/M22B-c-backend.md) — in progress

### Phase 6 continuation — Real-world compatibility

- [M23 — html-escaper equivalence port](milestones/M23-html-escaper.md) — complete

### Phase 8 — Language translation architecture

- [M24 — Language package IR and dynamic imports](milestones/M24-language-package-ir.md) — complete
- [M26 — Dependency-bearing flat language IR](milestones/M26-flat-language-ir.md) — complete

### Phase 6 continuation — Real-world compatibility

- [M25 — truncate-utf8-bytes equivalence port](milestones/M25-truncate-utf8-bytes.md) — complete
- [M27 — parse-ms equivalence port](milestones/M27-parse-ms.md) — complete
