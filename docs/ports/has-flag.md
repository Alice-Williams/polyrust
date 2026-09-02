# has-flag 5.0.1 compatibility port

## Provenance and scope

This port reproduces the complete pure typed behavior of
[sindresorhus/has-flag](https://github.com/sindresorhus/has-flag) 5.0.1 at
revision `63fde682532a6e0bb155125d03a66989e0b0ce24`. The implementation,
TypeScript declaration, runtime and declaration tests, package metadata,
README, and MIT license are retained under `third_party/has-flag/`. An offline
Bazel test verifies all retained files against their upstream Git blob IDs.

The portable function is
`has_flag(flag: String, argv: List<String>) -> Bool`. Upstream accepts the same
explicit `argv` argument but defaults an omitted argument to Node's
`process.argv`. Reading process state is an effectful host adapter and is not
part of the pure function admitted by this port. A caller can supply an
equivalent process-argument adapter outside generated code.

PolyRust `String` values contain Unicode scalar values. All well-formed
JavaScript strings are admitted, including supplementary scalars represented
by two UTF-16 code units; JavaScript strings containing lone surrogate code
units are outside this portable domain. These are explicit API boundaries, not
target-specific approximations.

## Portable implementation

The checked model expresses the implementation without a has-flag-specific
operation. It composes prefix testing, string concatenation, Boolean logic,
conditionals, option inspection, comparisons, and two new general operations:

- `StringUtf16Length: String -> I64` counts the code units in a value's
  well-formed UTF-16 encoding, preserving the upstream distinction between a
  one-unit short flag and an astral scalar.
- `ListIndexOf: List<T> x T -> Option<I64>` returns the first structurally equal
  element position or `None`, avoiding sentinel integers and target-native
  coercion.

The port exposed reusable representation gaps in C and Go. C now admits
`List<String>` parameters and `Option<I64>` results through its ownership-safe
ABI and lowers the required list/option operations through dependency-bearing
fragments. Go now normalizes typed `PolyList`, `PolyOption`,
`PolyValueResult`, and record arguments recursively at the IR runtime boundary;
it also implements the complete option/result predicate core used by checked
programs. Neither fix contains has-flag-specific logic.

Every new target mapping retains the M30 compositional architecture: syntax,
structured imports/includes, and helper roots travel in the same target
fragment; runtime helpers are selected by dependency closure; renderers only
spell resolved directives. JavaScript remains mechanically derived from the
TypeScript runtime.

## Equivalence evidence

The permanent M31 suite proves:

- all 11 official runtime assertions plus 14 empty, duplicate, terminator,
  prefix, equals-sign, NUL, BMP, combining, and astral boundary cases through
  25 portable vectors;
- all 25 vectors pass in the evaluator and generated Rust, TypeScript,
  JavaScript, Python, Go, Java, C++, and C packages;
- 42,273 deterministic admitted comparisons agree with the exact retained
  JavaScript implementation across 2,013 flags, three candidate spellings,
  and seven argument-vector arrangements;
- every fresh generated package passes its formatter/style check, static
  analysis, compiler, tests, and applicable C/C++ sanitizer gates; and
- three complete eight-target generations are byte-identical.

Reproduce the port-specific proof in the Linux development container:

```sh
bazelisk test //examples/real-world/has-flag:all --test_output=errors
```

The same suite is mandatory through `//:release_gate` together with every
earlier compatibility port. Completion gate counts and hosted-CI evidence are
recorded in the M31 milestone.
