# M35-01E-04C-02 — Compiler-authenticated Java graph assembly

- Status: complete
- Parent: [M35-01E-04C](M35-01E-04C-java-compiler-graph.md)
- Depends on: M35-01E-04C-01

## Implementation contract

- Add a Java graph entry point using the existing graph configuration,
  metadata CLI and source_check driver. Do not duplicate compiler configuration,
  metadata comparison, source hashing or dependency scheduling.
- Join invocation-scoped compiler crate numbers to exact previously checked
  Java APIs, then check stable crate and declaration identities. No name-only
  lookups or deserialized manifests confer authority.
- Retain owned Java certificates and descriptive inventories only after every
  member succeeds. Check mode reports success only for the complete graph and
  publishes no Java files, metadata or successful prefix on failure.
- Keep graph control, foreign joins and descriptive inventories in focused files;
  give the graph adapter its own Bazel target separate from the single-crate CLI.

## Definition of done and tests

- Two-crate, four-crate diamond, repeated aliases, unused dependencies and
  same-spelled distinct owners pass actual compiler analysis and certification.
- Private or unjoined foreign calls and wrong compiler owner substitutions fail.
- Source/metadata disagreement rejects and compiler scratch files are cleaned.
- Original single-crate Java and C graph targets continue to pass.
- C03 supplies independent execution, determinism and full regression evidence;
  filesystem publication is explicitly reserved for E04D.

## Inventory boundary

The graph inventory is a private owned root/defining-key/certified-owner map in
`java_graph/inventory.rs`. Each immutable API retains the full owning certificate
and its source export/doc/function/used-owner inventories. No descriptive field
is accepted as authority. JSON manifest projection and its byte/path budget are
part of E04D; the C03 test probe is not that production serialization format.

## Evidence

- Gate `ab2b5dbb-2a2e-47c4-aa72-82f528ddc4e4`: 9/9 targets pass, covering
  Java graph/native proof, existing Java source/AST/negative tests, shared source
  agreement, C graph regression, Rust/Bazel formatting and documentation checks.
- Exact metadata file swaps between same-named owners and a changed defining
  key reject. Changed code/docs reject even when the consumer never calls its
  declared dependency. Failed graphs report no successful prefix, publish no
  Java, preserve metadata bytes and remove their owned compiler scratch files.
- Earlier native gate `0d461063-e5cb-4eb3-8a1c-8bd991170168` also passed 5/5.
  The complete four-owner graph has compiler-checked export/doc identities and
  exact certificate-backed used-owner edges. C03 closes full regression/review.
