# M09 — Implement CLI and safe atomic output

- Status: complete
- Phase: 2
- Depends on: M06, M08

## Outcome

Expose checking, inspection, generation, and diagnostics through a stable CLI and
write validated manifests without partial or out-of-scope filesystem changes.

## Implementation checklist

- `polyrust check`, `emit`, `targets`, and `explain` commands.
- Human and JSON diagnostic modes with meaningful exit codes.
- Backend registration in the CLI layer and target/option validation.
- Staged output writer with path containment checks and documented recovery.
- Dry-run/manifest display mode reporting files, helpers, and dependencies.
- Integration-test harness using isolated temporary directories.

## Required exit evidence

- `check --target` performs full validation and capability preflight.
- No output directory changes occur on parse, check, preflight, generation, or
  manifest-validation failure.
- v0 never deletes unknown files in an existing directory.
- Every resolved staging/output path is verified beneath its explicit root.
- CLI output and exit codes are documented and stable within v0.
- `targets` is registry-driven and includes backend/IR versions and support.

### Verification

- End-to-end CLI success and every failure phase in fresh temp directories.
- Before/after directory hashes prove failures and dry runs make no changes.
- Symlink/reparse-point and path traversal tests on supported hosts.
- Simulated interruption tests validate documented recovery behavior.
- JSON diagnostics parse and contain no ANSI bytes.
- Snapshot tests for help, targets, and explain output.

```text
cargo test -p polyrust-cli
cargo run -p polyrust-cli -- --help
```

### Completion gate

All filesystem safety tests pass on Windows and Linux CI, failure atomicity is
demonstrated at each phase, command documentation matches snapshots, and the CLI
can run the mock backend without concrete-target conditionals in core crates.

## Completion evidence

- `check`, `emit`, `targets`, and `explain` have stable help snapshots, JSON
  modes, and documented exit codes. `check --target` uses the registry's
  generation-free compatibility/options/capability preflight.
- The CLI-owned inspection backend proves registry composition without concrete
  target branches in core. `targets` deterministically reports backend and IR
  versions plus every capability support level.
- Isolated filesystem tests compare output around parse, semantic-check,
  preflight, generation, manifest-validation, output, and dry-run failures.
  Unknown files survive successful replacement and every failure.
- The writer validates each joined/resolved path, rejects symlink/reparse-point
  roots and ancestors, stages and syncs all files before mutation, journals
  backups, rolls simulated interruptions back byte-for-byte, and recovers a
  deliberately abandoned transaction. Windows reparse-point detection is
  compiled behind the platform API and is included in the M16 OS matrix.
- A real container run checked the registration fixture, displayed a dry-run,
  committed an artifact, and verified SHA-256
  `d8d361c0c97c54f633be12d114532c9f04f27a1d1c6e9ede340de6b83948a3b9`.
- Cargo formatting, Clippy with warnings denied, all workspace tests/doctests,
  and all 16 authoritative Bazel tests (including Rustfmt, Clippy, Buildifier,
  and dependency boundaries) pass in the pinned Linux development container.

## Scope boundary

Cleaning owned directories, watching files, daemon mode, and downloading tools or
dependencies.
