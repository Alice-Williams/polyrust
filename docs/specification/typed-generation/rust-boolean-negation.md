# Rust-source Boolean negation

- Status: implemented; bounded native and compile-contract evidence recorded
- Task: [M35-03A-02A](../../plan/tasks/M35-03A-02A-boolean-negation.md)

## Source contract

BooleanNegation is a target-independent capability with a session-bound HIR
input. Admission checks the built-in unary Not form, bool result and operand,
absence of overload resolution, and absence of implicit compiler adjustments.
Integer bitwise negation and trait-based Not are separate capabilities, not
alternative interpretations of this one. Explicit dereference of an already
supported shared bool place can form the operand.

The target mapping obtains the operand through the reader's existing expression
mapping. It evaluates it once, preserves its prelude, and produces a typed unary
expression. This adds no statements except those already needed to evaluate
the operand. Parentheses and token separation remain renderer responsibilities.

## C mapping

Use CUnaryOperator::LogicalNot through the checked expression builder. C's
integer result is explicitly converted to CScalarType::Bool before returning
the value. Reuse registry provenance, scopes and target certification. Do not
render a source fragment or request a helper/runtime file.

## Java mapping

Use JavaUnaryOperator::Not, unary precedence and Boolean expression/result type.
Use the existing Value representation check and package certification. Do not
request JavaRuntimeCallable or any runtime helper/catalogue entry.

## Registration and scope

Both source capability builders require the actual executable mapping, with
typed context, input, output and associated capability. Missing or duplicate
registration and incompatible signatures fail to compile. The generic input
module has no target backend dependency.

This contract does not enable lazy and/or, integer bit-not, trait dispatch or
new ownership forms. Unsupported valid Rust produces a diagnostic, not an
approximate mapping. Tests prove supported behavior and the closed boundary;
they are not a formal proof of every arbitrary Rust program.
