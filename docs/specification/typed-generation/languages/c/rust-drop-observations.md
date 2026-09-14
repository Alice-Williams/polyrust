# Pinned Rust ownership/drop observations

- Scope: [M35-02A](../../../../plan/tasks/M35-02A-compiler-drop-evidence.md), compiler-only probe
- Contract: [owned values](rust-owned-values.md)
- Target heap support: **not enabled** in C or Java

## Query and identity

The Rust 1.98.0 rustc-dev probe runs in `after_analysis`, after compiler errors
have been rejected, and checks declared source dependencies. For every fixture
function it borrows `mir_drops_elaborated_and_const_checked` without first
requesting optimized MIR. The returned phase is exactly
`MirPhase::Runtime(RuntimePhase::PostCleanup)`; a typed assertion pins this.
The HIR body owner's LocalDefId and MIR source DefId must agree with the queried
owner. The probe never parses printed MIR or overrides a compiler pass.

Standard Box locals and projected places are recognized by the compiler's
`owned_box` language-item DefId. Calls retain FnDef identities and typed
destinations. Actual allocation-call destinations are Box-typed and their
callees are external, while the counterfeit wrapper calls a local function.
This observation does not yet authenticate a constructor capability or claim
that every external Box-returning call is Box::new.

## Fixture evidence

Numbers are syntactic MIR observations, **not dynamic allocation/drop counts**.
Locals include compiler temporaries; a runtime cleanup may occupy multiple
mutually exclusive blocks. The closed inventory rejects missing/extra functions.

| Fixture | Box locals | Box move operands | Box drops | Projected drops | Switches | Calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| straight | 2 | 1 | 1 | 0 | 0 | 1 |
| conditional | 4 | 3 | 2 | 0 | 2 | 2 |
| partial | 4 | 3 | 2 | 1 | 0 | 2 |
| early | 1 | 0 | 2 | 0 | 1 | 1 |
| shadow | 2 | 0 | 2 | 0 | 0 | 2 |
| imitation::fake | 0 | 0 | 0 | 0 | 0 | 0 |
| counterfeit | 0 | 0 | 0 | 0 | 0 | 1 |

Each body has one return terminator. Abort-mode fixtures have no unwind cleanup
blocks, and call/drop unwind actions are Unreachable. In addition to counts:

- Straight move connects the allocation destination to the dropped owner.
- Conditional cleanup uses a boolean flag: false skips the original owner's
  drop and true reaches it; the move block clears that flag. Typed constant
  assignments pin initialization and updates.
- Partial move extracts field zero, drops its new owner, then drops field one
  of the original record. The moved-from first field is not dropped there.
- Both early-return branches reach distinct drop blocks for the same owner.
- Shadowed allocations have distinct places and drop inner-before-outer.
- The local Box has the same item name but a different DefId to standard Box,
  and has no standard-Box drops. The oracle pins the counterfeit fixture identity.

## Reproducible tests

Run in the Linux dev container:

```sh
bazelisk test //experiments/rustc-frontend:owned_probe_test \
  //experiments/rustc-frontend:owned_probe_format_test
```

The probe compiles through the pinned Clippy adapter action with warnings denied.
Rustfmt has its own target for the nested Rust modules and input fixture. The test also requires
E0382/E0502 for invalid moves/borrows before observation succeeds, rejects an
empty inventory, and checks valid-Rust mutations for conditional, partial,
early-return and counterfeit-name cases. Existing C/Java adapters reject Box construction with
the expected unsupported-direct-call diagnostic, preserving absent/existing
outputs. No output package or renderer is involved in the probe.

## Remaining obligations

Body identity is not variable/operation correspondence. M35-02B must establish
compiler-owned, unambiguous HIR-operation to MIR-place/exit relationships; debug
names, spans and local numbers alone are not authority. M35-02C owns typed C
allocation/move/drop mappings and their allocator/failure policy. M35-02D owns
native runtime cleanup evidence, failure injection and sanitizer/mutation gates.
The MIR graph is not a request for a goto renderer. This probe cannot certify
target cleanup, arbitrary Rust, custom Drop, unwinding or all ownership shapes.
