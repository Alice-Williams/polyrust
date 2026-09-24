# Rust scalar results in C17

- Status: private transport and owned public ABI complete; imports planned
- Contract: [shared](../../rust-scalar-results.md)

Use a source-derived complete struct with a typed Boolean success tag and I32
payload. Initialize both fields on every construction; canonical error
construction sets the unused payload to zero. Any error-tagged payload is
ignored semantically. Avoid unions, uninitialized storage and error sentinels
that collide with valid integers. By-value transport needs no custom allocator.

The result's nominal identity, member identities and complete defining header
must survive independent package certification/import. Extend the currently
scalar-only public signature profile narrowly; arbitrary aggregate, recursive,
pointer-bearing or heap types stay rejected. Update call/frame/metadata/output
budgets before admitting the shape. No unchecked C struct/name reconstruction.

Source match lowering tests the tag before exposing success payload or the
opaque error witness; it evaluates only the selected arm. Foreign C callers
must use the published nominal type. Fully initialized representation does not
authorize observing an inactive payload in the source model. Prove standalone
headers, native ABI/copies/returns and original dependency ownership.

## Private transport boundary

Before public ABI support, the certified profile may pass/return this exact
unqualified two-field layout only through internal functions in its defining
implementation file. The complete declaration must precede signatures. Layout
recognition uses registered nominal/member types, never spelling. The closed
call-effect analysis can include these pointer-free values, but still derives
effects from actual bodies and their acyclic callees. This does not certify Rust
variant identity or make all same-layout source types interchangeable.

## Owned public ABI boundary

The certified profile admits only the same exact complete Bool/I32 layout
as an owning public-header declaration before prototypes. Its type and fields
are exported graph symbols because C exposes fields in a complete struct.
Source variant access remains independently checked during future HIR lowering.
External signatures use header-owned types; internal functions may reuse them.
No standalone source-only aggregate ABI or private type leakage is admitted.

Exported fields retain aggregate scope, not the ordinary package-wide namespace.
The linker derives `BindingScope::Type` from each exact registered member owner
in the original checked projection and reconstructs it during certification.
Different structs may reuse field names, including a name also used by an
ordinary function. No textual owner prefix or renderer-side name repair is
permitted. This extension leaves existing source-private spelling allocation
unchanged.

Until nominal import certification is implemented, any package whose header
contains an aggregate is rejected by CDependencyApi publication. This applies
even to imports of scalar functions from that package: including the header
would also introduce its tag namespace. Native C consumers can still use the
certified header/source directly under the documented C17 platform contract.
