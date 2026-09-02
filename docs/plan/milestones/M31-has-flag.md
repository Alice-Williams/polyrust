# M31 — has-flag 5.0.1 equivalence port

- Status: in-progress
- Phase: 6
- Depends on: M30

## Outcome

Port the pure typed behavior of MIT-licensed `has-flag` 5.0.1 into one checked
PolyRust model and generate equivalent Rust, TypeScript, JavaScript, Python,
Go, Java, C++, and C packages.

The portable API makes `argv` explicit. Upstream's omitted-argument default to
Node `process.argv` is a host adapter, not pure function behavior, and remains a
documented admission boundary. All well-formed Unicode-scalar `flag` and
`argv` strings are admitted, including astral scalars whose JavaScript length
occupies two UTF-16 code units. Lone surrogate code units remain outside the
PolyRust `String` domain.

## Implementation checklist

- Retain implementation, declaration, runtime tests, declaration tests,
  package metadata, README, and MIT license from tag `v5.0.1`, commit
  `63fde682532a6e0bb155125d03a66989e0b0ce24`.
- Verify every retained byte against its upstream Git blob ID without network
  access at test time.
- Add general `StringUtf16Length` semantics rather than approximating
  JavaScript `String#length` with scalar or UTF-8 byte length.
- Add general `ListIndexOf` semantics as
  `List<T> × T -> Option<I64>` for equality-eligible values.
- Express prefix selection, candidate construction, first-match lookup,
  terminator lookup, and ordering entirely in checked PolyIR.
- Retain all 11 official runtime assertions and add empty, duplicate,
  terminator-first, dash-prefixed, equals-sign, and Unicode boundary vectors.
- Differentially compare the model with the pinned JavaScript implementation
  across a deterministic corpus of flags and argument lists.
- Generate, lint, compile, and natively test all eight outputs, including C/C++
  public consumers and sanitizers.

## Required exit evidence

- Provenance tests pass for all retained upstream blobs.
- Checker and evaluator tests cover UTF-16 length and list first-index typing,
  semantics, and invalid signatures.
- Every backend has focused positive and negative lowering coverage for both
  new operations.
- Every official and expanded portable vector passes in the evaluator and all
  eight generated targets.
- The differential oracle reports every admitted comparison passing.
- Three complete generations are byte-identical.
- Uncached `//...` and `//:release_gate` pass in the Linux development
  container, including Buildifier, Rustfmt, and Clippy.
- The completed milestone is committed, pushed, and green in hosted CI before
  another repository is selected.

## Local completion evidence

- `//examples/real-world/has-flag:all` passes 16/16 port targets, including
  exact provenance, 25 evaluator/eight-target vectors, 42,273 differential
  comparisons, three-generation determinism, native consumers, style checks,
  and C/C++ sanitizers.
- `bazel test //... --nocache_test_results --test_output=errors` passes
  216/216 tests in the Linux development container.
- `bazel test //:release_gate --nocache_test_results --test_output=errors`
  passes 193/193 tests, including Buildifier, Rustfmt, Clippy, documentation,
  dependency-boundary, source-policy, fault-injection, every earlier port, and
  all generated-language native gates.
- Hosted-CI evidence is added in the final M31 documentation checkpoint after
  the implementation commit is pushed and its workflow succeeds.

## Scope boundary

M31 proves the dependency-free decision function when `argv` is supplied.
Reading a process argument vector is an effectful host concern and is not
implicitly mapped across targets. PolyRust strings contain Unicode scalar
values, so JavaScript strings containing unpaired UTF-16 surrogates are not
admitted inputs. These are explicit domain boundaries, not approximate output.
