# Curated generated Java snapshots

These two directories are deliberate, reviewable exceptions to the normal rule
that generated output is not committed. They exist because an example must show
the actual public source a user receives:

- `v0/` is generated from `crates/build/testdata/registration.poly.json`.
- `interfaces/` is generated from the canonical interface/composition fixture.

Do not edit files below those directories by hand. The
`snapshot_regeneration_test` target regenerates both packages through Bazel and
requires every byte to match.
