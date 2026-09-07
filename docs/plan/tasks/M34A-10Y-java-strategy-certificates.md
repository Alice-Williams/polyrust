# M34A-10Y — Bind strategy certificates to executable Java mappings

- Status: planned
- Depends on: M34A-08V and M34A-10U
- Blocks: completion of M34A-10W, M34A-10R, and M34A-11

## Goal

Make each exact dynamic support decision derive from the registered mapping
which performs that operation, not an independent central strategy switch.

## Implementation order

1. Move operation strategy selection into the owning mapping contract. Use
   closed typed operation/shape inputs; registration supplies the same mapping
   instance to certification and lowering. No blanket or default strategy.
2. Preserve shape facts needed to distinguish native operators, runtime
   helpers, and structured expression plans. In particular, boolean RHS plans
   containing statements require structured short-circuit control flow.
3. Have lowered plans carry their selected strategy or an equivalent typed
   witness and compare it with the preflight decision before certification.
   Do not duplicate target lowering logic in the generic feature collector.
4. Remove the central blanket unary/binary `Emulated(RuntimeHelper)` switch
   and audit non-intrinsic decisions using the same criterion.

## Definition of done

- Native wrapping arithmetic, bitwise/float operators, boolean not, and string
  concatenation no longer receive runtime-helper certificates.
- Payload-free enum equality selects `Enums`; local reads select the capability
  owning their checked binding origin. Both expression and constant paths agree.
- A strategy mismatch between preflight and actual mapping output is rejected;
  recomputing the same independent switch is not accepted as proof.
- Removing a mapping prevents both typed admission and dynamic certification.
- New implementations and tests remain in small capability-owned modules.

## Tests

- Table-driven coverage of every closed intrinsic operation and relevant shape
  against the actual typed AST/plan produced by its owning mapping.
- Short-circuit RHS fixtures with and without prerequisite statements; native
  execution proves RHS evaluation remains conditional.
- Enum/general equality and all four local binding-origin certificates.
- Zero-variant enums on the legacy dynamic path: the shared shape collector
  and Java currently disagree on whether they count as payload-free. Resolve
  representation or reject the shape explicitly; never certify a different
  owner from the one invoked by lowering. This is not a typed enum constructor.
- Deliberately mismatched strategy/output witness is rejected.
- Full Java tests, Rustfmt, strict Clippy, Buildifier, tracked Bazel graph,
  release gate, deterministic conformance, hosted CI, and fresh uncapped review.

## Commit gate

Commit and push after local proof, citing this task. M34A-10W remains open until
this task and M34A-10X both meet their exit criteria.
