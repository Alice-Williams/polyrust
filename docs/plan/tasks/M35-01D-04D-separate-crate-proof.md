# M35-01D-04D — Separate-crate integration proof

- Status: complete
- Depends on: [M35-01D-04C](M35-01D-04C-rust-crate-dependency-driver.md)
- Parent: [M35-01D-04](M35-01D-04-c-crate-linking.md)

## Goal

Prove the complete two-crate compiler-to-C pipeline and close the C boundary work.

## Definition of done

- Two actual Rust crates become two independent checked C implementations and
  public APIs; only exact admitted foreign symbols cross their boundary.
- Public aliases, same-spelled items, private helpers/layouts, docs and typed
  dependency imports agree with compiler/manifest/native inventories.
- Refresh inspectable ignored examples from tested declared artifacts.
- Close parent C obligations only after all child evidence and reviews pass;
  continue Java retrofit without pushing a failing migration checkpoint.

## Implementation sequence

1. Add a focused real Rust source diamond: shared leaf, two consumers, and root.
   Exercise same-spelled functions, public aliases, private concrete records and
   helpers, module/function docs, bool/i32, zero/multiple arguments and nested
   foreign calls. Reuse existing capabilities only; unsupported syntax is not
   silently approximated to make a fixture pass.
2. Build the same sources as independent pinned Rust libraries and a native
   oracle. Generate the C bundle through rust_c_bundle, retaining declared
   source/doc/metadata inputs and separately owned implementations.
3. Construct handwritten native test consumers from the descriptive public
   binding inventory. Compile every generated .c file separately, then link;
   do not concatenate source or alter generated imports. Run both bool values,
   i32 boundary seeds and the existing -4096..4096 differential range.
4. Check exact public definitions and generated foreign undefined symbols per
   object with inlining disabled. Verify private helper/header and private
   record access rejection, duplicate/header-order safety and doc placement.
5. Run GCC/Zig O0/O2 and GCC ASan/UBSan at O0/O2, three include orders, with the
   existing 1 MiB controlled-stack policy. Reuse existing certificate/resource
   mutation gates; add actual-source differential coverage rather than copying
   a second semantic evaluator into the test.
6. Refresh a byte-verified ignored workspace bundle and document commands,
   admitted subset and evidence. Run all integration gates and fresh broad
   review; only then close C parents and begin Java retrofit.

## Required proof matrix

- Native Rust oracle versus separately compiled/linked C packages over boundary
  and differential vectors, GCC/Zig O0/O2 plus GCC ASan/UBSan, both include orders
  and the existing controlled-stack/resource policy.
- Exact defined/undefined production symbols, no copied foreign bodies, illegal
  private consumers, metadata/registry/resource mutations and cache invalidation.
- Complete release/frontend/C/shared/Java gate with all tests enabled and cache
  reuse allowed, plus Clippy/rustfmt/buildifier/docs/source-policy checks.
- Fresh broad Sol Extra High review loop has no unresolved core findings.

## Implementation evidence

The same four Rust source files build as pinned independent native Rust libraries
and as a declared checked C bundle. The shared leaf has public aliases behind a
private ancestor, a private concrete record/helper, a private bool helper and
zero/multiple-parameter scalar APIs. Left/right consumers share that leaf; the
root exercises nested foreign calls and local/public aliases. No new backend
capability was added for the fixture.

Initial gate `a342027d-f076-48d5-924c-ac7445181061` correctly rejected
an unsupported conditional expression inside a call argument and flagged a
needlessly verbose boolean expression. The fixture now uses supported terminal
branches with helper calls, including private bool calls in both branches.
Clippy remains strict: no lint was disabled. Conditional call arguments remain
an explicit diagnostic regression, not an accidental feature extension.

Focused gate `1fd0a33c-0e1b-4100-af6b-240d198e566a` passed all six
targets. The actual declared bundle is compiled into four separate objects in
eight GCC/Zig/ASan/UBSan O0/O2 profiles, each linked with three independently
compiled header-order consumers and run on a 1 MiB stack. Native Rust supplies
8,204 oracle rows with sixteen results each, including both bool values and i32
boundary/differential inputs. Exact public definitions and generated foreign
undefined symbols, alias sharing, same-spelled cross-crate separation, private
function/record rejection and public/private documentation placement passed.
Full integration gate b40ff3e6-0280-43cb-b7df-98e95c86fead passed 342 tests.
The first independent review found proof gaps: linkage expectations were
self-derived, one private helper and several docs were untested, and the nested
call pair commuted. All were accepted. The strengthened oracle reconciles every
external function with public bindings, rejects both private helpers through
headers, checks every fixture doc exactly once on the correct side, and includes
a deliberately noncommutative nested pair with a native Rust witness.

The first strengthened run exposed a test-only schema mistake (owned manifest
functions do not include parameter lists); both known single-scalar private
fixture calls now use their explicit fixture protocol. No product schema or
warning policy was changed to accommodate the test.

Post-repair full gate 70fd38a2-4532-4d5b-9a93-6bb4e9a30080 passed
342/342 tests across 411 targets, including historical Java, in 46.627 seconds.
The ignored thirteen-file output/crate-bundle/m35-04d example was refreshed
from generated_crate_proof.bundle and diff -rq verified byte equality.
README now documents the native target, artifact and reproduction procedure.
At that checkpoint independent re-review remained in progress. This is evidence for the
admitted subset, not a universal theorem about arbitrary Rust programs.

## Final review and completion

Subsequent reviews accepted and closed further proof-harness gaps: the branch
choices are observably distinct and the correct nested composition is nonconstant;
reversing nested calls differs at zero; root export rows include exact module,
namespace, kind and uniqueness checks; expected import signatures/owners are
specified independently from emitted imports. All owned public/private function
symbols are checked at O0. O2 permits legitimate compiler removal/cloning of
private helpers while retaining exact public definitions and foreign imports.
This scoped O0 rule avoids claiming that optimization preserves private symbols.

The stack test now reads back the effective 1,024 KiB limit. Every fixture doc
must occur exactly once across all eight C/header files, on its expected side,
so copying another crate's private documentation also fails. No product feature
was added and no warning/test was disabled to satisfy these reviews.

Post-repair native/full gate cde26f77-b05f-4054-8b55-a90b23a9d189
passed 342 tests. Final gate 5630aeea-7116-4b63-bfcf-5a07969cb6a2
passed 343/343 tests across 413 targets, including the strengthened stack/docs
checks and the independent compiler-capability probe. It took 31.626 seconds.
The thirteen-file ignored m35-04d bundle and ten-file m35-04c bundle were each
byte-verified against their final declared artifacts after capability extraction.

Fresh broad Sol Extra High closure review found no concrete scoped core defect.
This closes M35-01D-04D, M35-01D-04, M35-01D and the pending M35-01B public
boundary obligation. Java retrofit continues; pushes remain held until its
full migration and proposed-commit-tree gates pass.
