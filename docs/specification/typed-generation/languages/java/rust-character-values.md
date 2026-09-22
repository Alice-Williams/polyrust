# Rust Unicode scalar values in Java21

- Status: target foundation complete; source admission disabled
- Contract: [shared](../../rust-character-values.md)

Use JavaPrimitive::Int with exact typed integer literals for scalar numbers.
Java char is a UTF-16 code unit, whereas int can hold supplementary code points;
see [Java21 Character representation](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Character.html).
Do not box into Character, truncate to char, store surrogate pairs or construct
Strings. Numeric scalar ordering is not UTF-16 lexicographic ordering.

Source TypePlan::Char remains distinct from I32. Source argument/result joins
must compare original types, not just their shared Java Int representation.
Immutable locals, direct calls, conditionals and six Int comparison nodes
remain structurally generated and certified. There is no runtime helper.

An arbitrary Java int is not a checked source char. Public metadata documents
the scalar input domain; foreign callers must provide values representable by
the original Rust type. Future dynamic conversion/validation is separate.
Do not use Character.isValidCodePoint as proof of a Unicode scalar: surrogate
code points also belong to the code-point interval.

Native proof uses actual separately compiled certified packages, Java21 strict
warnings, normal and -Xint runs, every scalar and all comparison pairs.
Detect actual narrowing and wrong-order faults. Prove typed source identity,
resource accounting and exact dependency/owner preservation before source
admission. This is partial character value parity, not Unicode text support.

The target foundation reuses existing primitive Int admission and structural
rendering. It does not add a validation/conversion function. Surrogate rejection
is proved by the independent Rust char oracle and later checked source input,
not by claiming a Java int has fewer bit patterns. Native fault copies may use
Character/String solely to demonstrate incorrect UTF-16 semantics; correctly
generated packages may not acquire those dependencies or representations.
