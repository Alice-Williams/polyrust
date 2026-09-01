# M22B — C17 backend

- Status: in-progress
- Phase: 7
- Depends on: M22A

## Outcome

Add C17 as the eighth independent output with explicit allocation, ownership,
monomorphized container/result shapes, contract vtables, and no C++ ABI leakage.

## Required exit evidence

The C-specific checklist and exit evidence are normative in
[M22](M22-cpp-c.md). Completion requires all eight outputs, allocation-failure
coverage, ASan/UBSan, deterministic manifests, and all real-world native gates.

## Foundation checkpoint evidence

- The checked-only `org.polyrust.c` backend generates module-prefixed C17 APIs,
  borrowed inputs, allocator-owned string/byte results, recursive record
  clone/drop functions, and borrowed contract vtables.
- The registration package has strict Zig C17 compile, public-consumer,
  conformance, style, allocation-failure, and pinned GCC ASan/UBSan tests.
- Models-and-validation and all four completed real-world ports generate C and
  pass 20 C-specific native/conformance/style/sanitizer targets.
- CLI registration, eight-manifest determinism, benchmark, Rustfmt, Clippy, and
  release policy include C.
- Deterministic dependency-ordered ABI generation now emits concrete typed
  lists, options, value results, records, enums, and owned callable-result
  wrappers. Public aggregate headers contain no untyped container pointers;
  irreducible by-value cycles produce diagnostics.
- Native plus GCC 14 ASan/UBSan ownership tests cover empty and nested lists,
  every option/result/enum tag, independent clones, recursive and idempotent
  destruction, allocation failure at the container and nested-element points,
  and zeroed droppable failure outputs. These tests are release-gate members;
  the expanded gates pass 123/123 repository tests and 101/101 release tests.
- Container/enum construction, matching, intrinsics, bounded iteration, and
  arithmetic lowering remain diagnostic-only until the next M22B slice.
- The foundation checkpoint passes `bazelisk test //...` with 121/121 tests and
  `bazelisk test //:release_gate` with 99/99 tests in the Linux container.
- Runtime string operations return exact `POLY_INVALID_UTF8` and
  `POLY_ALLOCATION_FAILED` codes, reject non-empty null views, and leave failed
  outputs zero-initialized and droppable. The ownership and sanitizer tests
  cover both failure classes, and the complete 121/121 plus 99/99 gates pass.
- The eight-target 1,000-declaration benchmark emits 47 files and 2,143,600
  bytes in 423 ms with 11.80 MiB peak RSS, below both release limits.
