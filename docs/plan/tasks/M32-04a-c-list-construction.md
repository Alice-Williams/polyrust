# M32-04a — Complete C17 dynamic `List<String>` construction

- Status: complete

## Completion evidence

- Validation admits only `ConstructList<String>` and recursively validates
  every element.
- Generated C evaluates and clones each element once, tracks the initialized
  prefix, deep-clones callable results, and unwinds every success/failure path.
- An external public-API consumer covers empty, two-element ASCII, and Unicode
  results. Its tracking allocator injects failure at every observed allocation
  point and proves zero live allocations and no invalid frees.
- Strict C17, AddressSanitizer, and UndefinedBehaviorSanitizer targets pass.

## Goal

Close the reusable backend gap exposed by the exact split-on-first result shape:
C17 already transports `List<String>` across public APIs, but checked
`ConstructList` expressions must also be able to build empty and populated
owned lists inside a function.

## Definition of done

- C validation admits `ConstructList` only for the already supported
  `List<String>` element family and recursively validates every element.
- Lowering evaluates each element once, clones it into allocator-owned storage,
  and unwinds every initialized element plus the list allocation on any failure.
- Empty construction is valid without allocation; populated construction
  preserves order and exact UTF-8 bytes.
- Result ownership composes with existing function returns, portable test
  expectations, and failure cleanup without leaks, double frees, or borrowed
  lifetime escape.
- Focused unit, native, public-consumer, ASan, and UBSan tests cover empty,
  two-element, Unicode, and injected allocation-failure paths.

## Tests

- Focused C backend tests plus
  `bazel test //examples/real-world/split-on-first:c_generated_test //examples/real-world/split-on-first:c_sanitizer_test --test_output=errors`.
