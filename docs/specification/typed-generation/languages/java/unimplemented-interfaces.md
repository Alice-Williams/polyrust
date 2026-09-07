# Java interfaces with no portable implementations

- Status: implemented locally in M34A-10X; integration and review pending
- Decision: zero implementations is valid in the generic `Interfaces` capability

## Representation

The public generated type remains a sealed Java interface with exactly its
portable method signatures. When no portable record implements it, the Java
`Interfaces` mapping emits one private, zero-constant enum implementing it and
names that enum in the interface's permits list. Both declarations live inside
the generated enclosing class. The enum is implicitly final and has no values;
it is an uninhabited target-only permitted subtype, not a fake portable record.

Each required enum method has the same resolved signature as its interface
method and a body that throws an internal assertion failure. Ordinary Java
cannot construct an enum instance, and there are no constants, so these bodies
are unreachable. No public constructor, factory, enum value, portable symbol,
or conformance witness is issued for this subtype. Reflection, unsafe runtime
manipulation, and bytecode injection remain outside the ordinary typed boundary.

When at least one real portable implementation exists, only the real generated
implementations are permitted; no synthetic subtype is needed. This does not
add inheritance to the generic frontend or introduce an inheritance chain.

## Layer ownership

- CoreIR preserves the interface and its empty implementation set unchanged.
- The Java mapping chooses this representation and requests a fresh typed
  generated symbol with an explicit synthesis reason.
- The symbol allocator reserves collision-free names in the same namespace
  as user nominal types before unresolved AST verification, not only during
  linking. The synthetic name is not referenced through raw source text.
- The Java AST models the empty enum, exact conformance, methods, and permits
  edge structurally. Verification checks their matching identities and types.
- The linker resolves all references; rendering only prints certified nodes.

The AST verifier MUST NOT globally relax sealed-interface or enum checks to
admit this representation. Any new internal privilege must be restricted to
this exact synthesized shape and independently tested against mutations.

The concrete AST variant is `UninhabitedEnum(interface_id)`, distinct from
ordinary `Enum`. Its registration uses `SynthesisReason::UninhabitedInterface`.
Each method carries `UninhabitedImplementation(interface_method_id)`, never a
forged portable implementation ID. Verification checks the exact registered
owner and generic signature, private visibility, exclusive permits edge,
zero values, no constructors or exposed factories, and the fixed assertion
body, and agreement with the registered declaration name. A generated type
identity has exactly one AST declaration in the package, even if a duplicate
node claims a different spelling. The normal nonempty portable-enum grammar
is unchanged.

## Required evidence

Typed programs declare zero-method and nonempty interfaces without adding
implementations, including interfaces used in parameter/result and container
types. Generation must not panic and all outputs compile with Java 21
`-Xlint:all -Werror`. A separate Java consumer can name the interface but cannot
instantiate the enum or add an implementation. Compile-negative tests prove
both restrictions, and mutation tests reject a constant, exposed factory,
mismatched permit, foreign method, or incorrect method signature. Existing
implemented-interface dispatch and immutability tests continue to pass.
