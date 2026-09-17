# Java21 binary64 truncation mapping

## Typed lowering

Materialize the original receiver once as Double. Use a typed Conditional
expression with Boolean comparison receiver < +0.0:
MathCeil(receiver) when true, MathFloor(receiver) otherwise.
Each known call has exact (Double)->Double signature, no receiver and Primary
precedence. Use the existing JavaKnownCallable identities and linker-resolved
names; never embed a qualified invocation in raw text. No runtime helper.

This maps negative fractions toward negative zero, positive fractions toward
positive zero, and retains either input zero and infinity. NaNs follow the
category-only contract. Do not use a long/int cast or positive-zero replacement.

## Certification and dependencies

Closed dependency-body admission permits only these two new known callables,
retains argument traversal and checks exact catalogue signatures. Actual
resolved owner-type names, the separator, and catalogue-owned method names
contribute to source-byte reservation at every call, matching the existing
structural renderer. For this implicit java.lang type the owner renders as Math;
normal qualifier-shadowing checks remain mandatory. Bounds and name
collision resolution remain enforced. Other known calls remain unadmitted;
a generic pure-signature flag grants no permission.

## Required proof

Java21 -Xlint:all -Werror compilation and native integer-bit oracle checks for
floor/ceil/composed truncation. Separate original/imported-owner evaluation
traces detect dropping/duplication. Wrong rounding, branch selection and zero
sign controls must fail. Invalid types, signatures, receiver, arity, precedence,
unadmitted calls and forged authority reject. Rendered bytes remain within the
computed bound.

Reference: [Java21 Math](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Math.html),
floor(double) and ceil(double) contracts.
