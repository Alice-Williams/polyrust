# v0 reference evaluator and canonical result protocol

## Boundary

The evaluator is the executable oracle for Core v0. Its public entry points
accept only portable_check::v0::CheckedProgram; unchecked Document values must
pass check_program first. The evaluator does not repair invalid programs or
select target-specific approximations.

portable_eval::Value is the IR value algebra itself: unit, boolean, signed 32-
and 64-bit integers, exact binary64 bits, Unicode scalar, string, bytes,
immutable list, option, result, nominal record, and nominal enum. Aggregates
are copied values. List append and concatenation produce new lists and cannot
alias or mutate their operands.

## Execution

Expressions and function arguments are evaluated left to right. bool_and
evaluates its right operand only when the left operand is true; bool_or
evaluates it only when the left operand is false. If evaluates one branch and
match evaluates the first matching checked arm. A return inside a nested block,
match arm, or bounded loop returns from the current function or method.

Function and concrete-method calls use their resolved declaration IDs.
Contract calls inspect the receiver's nominal record ID and select the unique
checked implementation for the requested contract and method. Iteration is
available only over a fully materialized immutable list.

Checked integer operations use explicit checked primitives and return a
structured error for overflow, zero division/remainder, invalid shifts, or
narrowing. Wrapping operations use explicit two's-complement modulo 2^32 or
2^64 semantics. They never depend on Rust debug/release overflow behavior.

Binary64 operations preserve the result bits, including negative zero,
infinities, and NaNs. Equality follows IEEE numeric equality: NaN is unequal
to every value and positive zero equals negative zero. Ordering with NaN is
false. String length counts Unicode scalar values, not UTF-8 bytes or grapheme
clusters; an astral scalar and a combining scalar each count as one.

## Deterministic limits and errors

Each public invocation receives a fresh EvaluationLimits budget:

- fuel is charged for calls, blocks, statements, expressions, constant
  evaluation, and each bounded-loop iteration;
- call_depth is charged for every function or method frame; and
- collection_size bounds every string byte length, byte string, list, and
  aggregate field collection supplied to or created by evaluation.

Exhaustion and semantic faults return EvaluationError; no limit path panics.
Stable error codes are:

    checked_overflow
    division_by_zero
    remainder_by_zero
    invalid_shift
    narrowing_out_of_range
    index_out_of_bounds
    invalid_utf8
    fuel_exhausted
    call_depth_exceeded
    collection_limit_exceeded
    invariant_violation

For a portable test whose ExpectedOutcome is Error, the expected typed value is
a string containing the stable error code. Target runners use the structured
canonical error object described below.

## Canonical JSON

Canonical outcomes use protocol polyrust.canonical.v0:

    {
      "outcome": "value",
      "protocol": "polyrust.canonical.v0",
      "value": {"type": "i64", "value": "9223372036854775807"}
    }

Errors replace value with an error object:

    {
      "error": {"code": "checked_overflow", "operation": "add"},
      "outcome": "error",
      "protocol": "polyrust.canonical.v0"
    }

The encoding rules are independent of a target's JSON number model:

- i32, i64, node IDs, indices, lengths, and limits are decimal strings;
- f64 is exactly 16 lowercase hexadecimal IEEE-754 bits;
- bytes are lowercase, even-length hexadecimal;
- char is a JSON string containing exactly one Unicode scalar;
- lists recursively contain canonical values;
- option and result use explicit variant tags;
- records and enums include decimal declaration/variant IDs and ordered field
  ID/value pairs; and
- error-specific data is represented by decimal strings or stable text.

The encode_canonical and decode_canonical families round-trip every value and
error. Malformed, unsupported, or non-canonical input returns
CanonicalDecodeError.

The target-neutral initial corpus is
conformance/v0/evaluator-vectors.json. It contains 20 named inputs and
canonical outcomes and may be consumed by generated target runners without
linking the evaluator crate.

## Verification

The M05 gate runs both profiles:

    cargo test -p polyrust-eval
    cargo test -p polyrust-eval --release

The suite covers 26 operation/fault vectors, mathematical wrapping properties,
all runtime value/error encodings, special floats, left-to-right and
short-circuit evaluation, every sum family, enum payload binding, list
non-aliasing, concrete and contract dispatch, eleven declared portable tests,
and public fuel/depth/size exhaustion.
