# C17 interfaces and composition

- Status: normative for the complete Interfaces mapping

One portable interface produces a concrete opaque owning handle and a private
flat function-table type. Each table method has an exact prototype, typed
receiver context, immutable arguments, result/ABI transport contract and
lifecycle operations from the [exact ABI](callable-abi.md). A table identity includes the interface, implementation
witness and concrete record type. The context pointer may be erased only by a
closed, verified adapter which restores that exact record; a matching C cast
alone is not conformance evidence.

The Interfaces mapping owns declaration, method prototypes, each complete
implementation bundle, table construction, clone/drop callbacks, handle move, conversion through
the exact witness, concrete dispatch and interface dispatch. Multiple
conformance uses distinct typed tables; method names and positions are not
global string keys. Composition forwards through named concrete fields, with
no prefix-layout inheritance or container-of casts.

An interface with zero implementations remains valid: emit its opaque public
declaration and method ABI, but no public constructor or fabricated conformance.
Internal dispatch code cannot fabricate an inhabitant. A moved/null lifecycle
slot is not a valid interface value. Options/results containing the interface
can still represent their other variants. No foreign vtable registration API
is exposed by default.

All admitted type positions support interface values, including nested lists,
records, options, results and legacy payload-enum fields. Copies preserve independent value ownership.
Interface values are never directly or recursively equality-comparable:
records, lists, options and results containing them cannot acquire equality
support. Equality of other concrete values follows portable semantics, never
context addresses. Table function prototypes are authenticated after monomorphization
and linking; a wrong receiver/result/callback or missing lifecycle slot fails
certification before rendering.

Proof includes independently compiled consumers; two interfaces implemented
by one record with overlapping names; distinct implementation witnesses;
empty interfaces; concrete/dynamic dispatch; nesting; explicit delegation;
native alias tests; move/clone/drop; all allocation-failure points; and
ASan/UBSan execution. A sizeof or vtable-layout text comparison is insufficient.
