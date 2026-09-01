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
