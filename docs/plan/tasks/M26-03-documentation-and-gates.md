# M26-03 — Architecture documentation and release proof

- Status: complete

## Goal

Document the strengthened translation boundary and prove it across the complete
repository and release surface.

## Definition of done

- Architecture and backend-author documentation show checked IR → target units
  → grouped language package → renderer → manifest.
- M26 evidence records focused, full, release, determinism, and native results.
- The milestone is committed with its ID, pushed, and its GitHub CI run passes.

## Tests

- `bazelisk test //... --test_output=errors` in the Linux container.
- `bazelisk test //:release_gate --test_output=errors` in the Linux container.
- Buildifier, Rustfmt, Clippy, target linters/formatters, and three-generation
  byte determinism remain part of those gates.
