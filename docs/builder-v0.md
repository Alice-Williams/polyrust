# v0 typed Rust builder

## Purpose

portable_build is the verbose, stable-Rust authoring frontend for Core v0.
Generator authors construct a complete target-independent document without
directly assembling IR expression or declaration variants. No procedural macro
or target-specific naming rule is involved.

The builder is intentionally a frontend. It does not replace the checker and it
does not contain backend lowering choices.

## Typed handles

Every declaration and member family has a distinct copyable handle:

- ConstantId and AliasId;
- RecordId and RecordFieldId;
- EnumId, EnumVariantId, and EnumFieldId;
- ContractId and ContractMethodId;
- ImplementationId and ImplementationMethodId;
- FunctionId; and
- TestId.

APIs accept only the appropriate handle. For example, Type::contract accepts a
ContractId, Value::record accepts a RecordId, and a concrete call requires an
ImplementationId plus an ImplementationMethodId. Rustdoc compile-fail tests
lock these category errors at compile time.

Record, enum, contract, and implementation configuration closures may return
member handles alongside their parent handle. This keeps later expressions
resolved by stable node identity rather than by target spelling.

## Builder families

ModuleBuilder covers constants, aliases, records, enums, contracts,
implementations, functions, and portable tests. Type and Value constructors
cover every Core v0 type/value. BodyBuilder covers every constant expression,
runtime expression, statement, block, pattern, and match arm.

CallableBuilder accumulates parameters, a return type, and a body. Omitting a
return type or body is an incomplete builder state and returns a structured
P0002 diagnostic at finalization. Duplicate names and other semantic mistakes
are accumulated by the normal checker when finish is used.

All nodes receive monotonically allocated nonzero IDs. Their source is a
logical path whose first segment is module(name), followed by declaration and
member roles such as record(Label), field(text), function(call), and block.

## Finalization

finish_unchecked returns Result<Document, Vec<Diagnostic>>. It rejects
incomplete builder states but deliberately leaves semantic validation to users
that need an unchecked document for serialization or negative testing.

finish first performs builder finalization and then invokes check_program. Its
success value is CheckedProgram, the same proof object consumed by the
evaluator and backends.

Neither path panics for user mistakes. Diagnostics retain logical builder
locations.

## Verification artifacts

The crate-level Rustdoc contains a complete checked and evaluated module plus
two compile-fail handle-misuse examples. The runtime suite:

- compiles every declaration, expression, statement, pattern, type, value, and
  typed portable-test builder family;
- verifies incomplete functions, incomplete contract methods, and duplicate
  names return diagnostics;
- asserts every generated source is logical and module-qualified;
- serializes builder output to canonical JSON and reads it back equally;
- compares the registration demonstration with
  crates/build/testdata/registration.poly.json; and
- checks the demonstration and executes its portable test through the reference
  evaluator.

The milestone commands are:

    cargo test -p polyrust-build
    cargo test --doc -p polyrust-build
