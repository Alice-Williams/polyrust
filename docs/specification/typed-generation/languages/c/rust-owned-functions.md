# Rust owned function-boundary correspondence

- Status: in-progress; I-01 identity complete, whole-body call support not admitted
- Plan: [M35-02B-03I](../../../../plan/tasks/M35-02B-03I-owned-function-boundaries.md)
- Parent contract: [owned values](rust-owned-values.md)

## Separate call identity from callee effects

A resolved function call is not an ownership certificate. Canonical HIR and
TypeckResults establish the direct local function identity, concrete signature,
argument identities and result type. A separate complete body proof establishes
what the function does with incoming and newly created ownership obligations.
Caller admission requires both, connected by the exact compiler function DefId.

Initially select only safe, nongeneric, ordinary Rust-ABI local free functions
with these closed signatures, represented by an enum rather than name tests:

| Role | Parameter | Result | Required body proof |
| --- | --- | --- | --- |
| Producer | i32 | standard Box<i32> | Construct and transfer one owner |
| Consumer | standard Box<i32> | i32 | Read its scalar, then clean up once |
| Relay | standard Box<i32> | standard Box<i32> | Transfer the incoming owner |

Role classification authenticates only a signature. It does not grant the
listed effect until whole-body correspondence succeeds. Full Box type identity
includes allocator arguments. Aliases preserve identity; same-spelled or
same-signature distinct declarations cannot substitute for one another.

## Implemented operation boundary (I-01)

OwnedLocalCall has a private LocalCallInput containing caller LocalDefId,
canonical call/argument, local callee identity, GenericArgsRef, instantiated
compiler FnSig, full standard Box<i32> Ty and CallRole. Canonical-node/owner
checks precede TypeckResults queries. A direct FnDef path must resolve to that
same local free function; empty generic arguments, safe Rust ABI, one exact
unadjusted argument and an exact unadjusted result are required. Method calls,
function-item locals, function pointers, computed callees and closures do not
enter this boundary. The pinned frontend rejects unsafe source before admission.

The fourth consuming builder slot, local_call, stores an executable Mapping.
Its input/context/output/capability relations are fixed by Rust traits; a missing
or duplicate slot cannot satisfy Supports<OwnedLocalCall>. The base scalar-Box
constructor slot remains required, and prior inferred consumer types are kept.
Role signatures use the actual compiler
[FnSig type](https://doc.rust-lang.org/stable/nightly-rustc/rustc_middle/ty/type.FnSig.html),
compiled and checked against pinned Rust 1.98.0 rather than assumed from docs.

The replacement fixture intentionally allocates a new Box despite having the
Relay signature. Its direct call passes I-01 identity checks, but receives no
callee-body or transfer evidence. I-02 must reject that body as a relay until
an explicit richer effect contract exists. No role marker is an effect proof.

## Call-graph and cleanup boundary

An admitted caller references checked callee evidence from the same compiler
session. Direct call arguments and destinations retain their actual compiler
types and places. Match HIR binding order and source argument identity to MIR
staging/move paths; no matching by spelling, debug data or equal types alone.

The first graph is finite, local and acyclic. Every reachable owned callee must
have one of the specified body proofs. Missing evidence or a cycle diagnoses
before any target lowering/publication. The precise graph and body budgets,
parameter/return staging and supported body grammar must be fixed from pinned
compiler observations before this planned contract is marked implemented.

Returning an owner transfers its cleanup obligation to the caller; passing an
owner to a consumer transfers that obligation to the callee. A relay must not
create or discharge an obligation. A body-wide call/move/drop census must rule
out missing or duplicated transfers. Source returns remain structured returns,
not generated MIR goto edges.

No target heap support follows from this compiler-only proof. Native allocation,
failure, clone and cleanup behavior still require the separate C mapping and
runtime gates. Existing Java heap restrictions remain unchanged.
