# M22 — Add C++ and C outputs

- Status: in-progress
- Phase: 7
- Depends on: M21

## Outcome

Expand every checked PolyRust program from six to eight outputs by adding
independent C++ and C backends. C++ uses modern value types and RAII. C uses a
documented generated ABI, explicit ownership, concrete generated container
types, and fallible operations without hidden exceptions.

Zig is used as the pinned hermetic C/C++ compiler driver. It is not an
intermediate source language: Zig's supported translation direction is C to
Zig, not general Zig to C or C++.

## Implementation checklist

- Define C++20 mappings for records, interfaces, tagged variants, `option`,
  result values, immutable lists, strings, bytes, integer behavior, and
  structured failures.
- Define a C17 ABI for public types and functions, including allocation,
  ownership transfer, borrowing, destruction, error values, and name mangling.
- Monomorphize every required C list, option, result, record, and enum shape
  deterministically from checked IR.
- Add independent checked-IR backend descriptors and capability maps; neither
  backend may accept unchecked input.
- Pin hermetic C/C++ compiler rules and enable warning-as-error plus formatter
  or style gates.
- Generate portable and conformance tests in both languages, including
  allocation-failure and sanitizer coverage for C ownership boundaries.
- Extend CLI registration, determinism, models-and-validation, benchmark, and
  every completed real-world port to eight outputs.
- Document unsupported ABI combinations with diagnostics rather than unsafe
  approximations.

## Required exit evidence

- Generated C++20 and C17 compile with every warning enabled and treated as an
  error under the pinned Linux toolchains.
- AddressSanitizer and UndefinedBehaviorSanitizer report no issue in generated
  native and conformance tests.
- C ownership tests prove success, failure, empty, nested, clone, and destruction
  paths without leaks or double frees.
- All eight manifests regenerate byte-identically.
- Every completed real-world port passes its pinned upstream oracle and native
  tests in all eight outputs.
- Repository Rust and Bazel linters and the complete release gate pass.

## Scope boundary

C++ and C are separate semantic outputs. C++ templates, standard-library
containers, exceptions, or destructors must not leak into the C ABI. The C
backend does not expose untyped `void *` containers or undocumented allocator
assumptions. Zig interop is not evidence of C/C++ source equivalence.
