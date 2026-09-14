# M35-03 — Production integration decision

- Status: planned
- Depends on: M35-02

## Goal

Specify production integration using evidence from the compiler experiments.

The initial no-heap C bridge is specified and scheduled under M35-01B/C/D.
This task extends the evidence-based decision to all languages and legacy
ownership retirement; it does not delay or rename that initial C work.

## Definition of done

- Fix a private compiler-adapter/internal-IR boundary, capability contracts,
  crate identity, generics, library mappings and target configuration.
- Update every language and frontend contract. Inputs bypassing rustc cannot
  inherit its type/borrow-checking guarantee.
- Separate input legality, target capability admission, lowering correctness
  and resource checks. Do not duplicate Rust's borrow checker.
- Give existing C proof modules explicit keep/adapt/retire dispositions; remove
  checks only after their obligations have permanent replacement evidence.
- Preserve interfaces, composition, executable capability registration, typed
  target ASTs, derived imports and structural renderers.
- Write bounded migration tasks for Rust, C, Java, TypeScript, derived
  JavaScript, Python, Go and C++, with native and historical-port gates.

## Tests and proof

Replay the existing corpus and compiler experiment; test admission failures,
proof privacy, declaration identity spoofing, determinism and dependency
direction. Fresh review and full Linux/Bazel gates precede legacy cutover.

## Commit gate

Record the integration decision and evidence, commit/push, update M34A order.
