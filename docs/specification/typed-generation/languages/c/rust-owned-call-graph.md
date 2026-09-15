# Closed owned caller/callee graph

- Status: complete for this bounded compiler proof; no target heap output
- Plan: [M35-02B-03I-02](../../../../plan/tasks/M35-02B-03I-02-owned-call-correspondence.md)
- Identity prerequisite: [owned function calls](rust-owned-functions.md)

## Graph boundary

The first supported graph contains exactly an i32-to-i32 entry function and one
directly called local leaf. Both are safe nongeneric Rust-ABI free functions in
the same compiler session. The entry has exactly one local owned call; the leaf
has none. Thus recursive or longer call chains cannot acquire this certificate.
Unrelated functions in the source crate do not become admitted through this graph.

Retain the exact entry and leaf LocalDefIds, I-01 call input and independently
checked leaf body. A same-signature function cannot substitute for the certified
callee. Role classification alone never supplies a leaf effect or cleanup fact.

## Closed source grammar

Each body has one unlabeled ordinary root block, a single immutable parameter,
at most 128 immutable let bindings and one tail or explicit return. Bindings use
simple patterns. The existing MIR bound remains 512 blocks and 1,024 locals.
Labels, nested scopes, scalar expressions, borrows, mutations, branches, extra
calls and other parameter/result types diagnose in this initial graph stage.

| Leaf | Source operations | Ownership leaving its frame |
| --- | --- | --- |
| Producer | Standard Box<i32> construction from its scalar parameter, optional whole-local moves, return | Newly constructed owner |
| Consumer | Optional whole-local moves from its Box parameter, scalar dereference, cleanup, return | Scalar value only |
| Relay | Optional whole-local moves from its Box parameter, return | Same incoming owner |

A Producer may directly return the constructor expression, without a source
binding. The other Producer form binds that result before optional moves and a
final owner return. Consumer and Relay bodies may have no let bindings at all.
Every form retains a canonical SourceExit accessible to the lowering consumer.

| Entry mode | Source operations | Entry cleanup |
| --- | --- | --- |
| Producer call | Pass scalar parameter to leaf, bind returned owner, optional moves, dereference | Drop final returned owner |
| Consumer call | Construct from scalar parameter, optional moves, pass current owner to leaf as scalar exit | None after transfer |
| Relay call | Construct from scalar parameter, optional moves, pass current owner to leaf, bind returned owner, optional moves, dereference | Drop final returned owner |

No mode composes multiple local calls yet. A function that matches a Relay
signature but allocates a replacement is not a Relay body and must reject.

## Typed correspondence

Authenticate actual PostCleanup MIR owner/phase and every normal block, call,
assignment, Box local, move, scalar producer and final return. Retain typed MIR
places/locations together with canonical HIR parameter/binding/call identities.
Compiler argument staging and RETURN_PLACE are not source-local bindings.

- A direct Producer constructor can write the Box RETURN_PLACE directly. A
  bound Producer instead has a final move into RETURN_PLACE. Neither has a Drop.
- Consumer leaf reads come from the final incoming owner and precede its one
  exact Drop, which precedes the scalar return. Relay leaf moves reach the Box
  RETURN_PLACE with no constructor or Drop.
- Call arguments use distinct staging locals of the actual parameter type.
  Scalar staging is exactly one direct Copy from the exact scalar parameter;
  the call may use Copy or Move of that i32 stage. Direct parameter arguments,
  Move staging and transitive copy chains reject. Owned staging moves the
  current owner before the exact resolved call. The destination's type and
  identity belong to that call, not another same-typed operation.
- The Consumer entry's call produces the scalar RETURN_PLACE directly. Producer
  and Relay entries read/drop the current owner returned by their exact call.
- All claimed effects must agree at the edge: producing transfers one new
  obligation, consuming discharges the incoming obligation in the leaf, and
  relaying passes that same obligation back. Missing/duplicate edges fail closed.

Only a private query-backed constructor may assemble graph evidence. No safe
consumer supplies arbitrary MIR, roles with assumed effects or fabricated
callee certificates. Existing executable mappings consume their corresponding
checked inputs. MIR branches remain evidence and never authorize a goto renderer.

## Evidence before implementation

Observation gate `7108ee7d-ede5-490a-a088-0ee94460eb87` passed runtime and
format targets in 13.760 seconds. Sixteen canonical HIR/PostCleanup bodies cover
direct/bound/moved Producer returns, direct/moved/explicit Consumer and Relay
exits, entry call/argument staging and explicit-return entry variants. The
observations establish the representations above, not a graph certificate.

## Implemented proof boundary

The private graph reader queries both complete bodies, then checks the exact
callee identity/signature and allocator-bearing Box type at assembly. Neither
arbitrary MIR nor an externally supplied body token is a safe constructor input.
The enum source grammar records construction, local move and local call operations;
the relation matches them in order against a bounded complete normal trace.

Read-only body projections expose canonical tail/return expressions, source and
MIR parameter/binding pairs, call argument/staging/destination locations, owner
move locations, read/drop endings and the final return. RETURN_PLACE and argument
staging are represented only as compiler places, never invented source bindings.
Consuming the evidence yields separate authenticated construction/local-call
inputs for the existing capability mappings; it does not enable a target emitter.

Focused proof covers ten graphs, 28 rejected entry bodies, 475 identity/transfer/
cleanup/graph corruptions, five exact compile-negative contracts, four invalid
Rust controls, six valid-source mutations and the 128/129 source-binding boundary.
The repaired isolated tree passed all 469 historical/native/lint tests and fresh
independent review. Exact tree and gate evidence lives in the linked task and
checkpoint commit. Native C cleanup/failure proof and Java heap mapping remain separate.
