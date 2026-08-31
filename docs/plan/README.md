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
- [M09 — CLI and safe output](milestones/M09-cli-output.md) — planned

### Phase 3 — Required target backends

- [M10 — Rust backend](milestones/M10-rust-backend.md) — planned
- [M11 — TypeScript backend](milestones/M11-typescript-backend.md) — planned
- [M12 — Python backend](milestones/M12-python-backend.md) — planned
- [M13 — Go backend](milestones/M13-go-backend.md) — planned

### Phase 4 — Cross-language proof

- [M14 — Differential conformance](milestones/M14-conformance.md) — planned

### Phase 5 — Usability and release

- [M15 — Examples and extension proof](milestones/M15-examples-extensions.md) — planned
- [M16 — CI and release gate](milestones/M16-ci-release.md) — planned
