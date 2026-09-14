# M35-01D-04B-03B — Certified external C symbols

- Status: complete
- Depends on: [03A](M35-01D-04B-03A-package-catalogues.md)
- Parent: [04B-03](M35-01D-04B-03-dependency-linking.md)

## Definition of done

- Represent certified generated-dependency callables distinctly from static
  known-library callables and owned generated declarations. Retain exact
  consumer/original identity, certificate, signature, owner and public header.
- Extend shared typed reference/catalogue handling without forcing C's existing
  compilation-unit AST into a duplicate shared expression AST.
- Derive C catalogue entries from authenticated imports and each file's required
  includes from actual direct calls. Deduplicate physical headers; native names
  cannot be aliased. Reject conflicting symbols, headers and owning packages.
- Reuse certified scalar effects in existing local C analyses. Never skip
  contextual, numeric, sequencing or storage checks to reach the linker.
- Preserve a final certification rejection until 04B-04 supplies complete
  transitive stack evidence. No copied dependency bodies or raw prototypes.

## Tests and proof

- Multiple public functions sharing one header and independent dependencies.
- Exact fixed symbols, original/consumer owners and reference-derived includes.
- Private/missing/extra/retargeted imports, catalogue changes, aliasing and
  symbol/path collisions reject, including coordinated post-link mutations.
- Imported callback/indirect/owned-body bypasses remain rejected.
- Shared/C/Java/compile-negative/policy gates and fresh review pass.

## Implementation boundaries

- Shared dependency callables are linker symbols with a concrete signature;
  C keeps its existing compilation-unit expression tree. No duplicate arena AST.
- Dialects reconstruct an exact dependency specification from their opaque
  callable witness. C retains consumer reference, original witness and package
  certificate separately from owned generated declaration IDs.
- Fixed-import resolution rejects collisions rather than assigning aliases;
  post-link verification enforces the same exact native spelling.
- Reconstruct all binding allocation from original AST/helper authority to
  reject coordinated binding/reference renames. Reserve unused dependency names
  against owned package bindings and each other. Compare actual rendered header
  include names, not just full output paths; reject overlapping owning crate IDs.
- Reserve each dependency owner's full certified public export inventory,
  including unselected functions, against other owners and actual resolved
  consumer callable names without adding false imported reference roots.

## Review decisions

- Accepted: full-header/object imports expose unselected public functions.
  Added owned/unselected, selected/unselected and unselected/unselected
  collision regressions; all three demonstrated the missing rejection.
- Rejected as a current defect: a synthesized struct could leak through a scalar
  dependency header. The fixture already fails C projection with
  `C public header admits only primary external scalar prototypes`, before
  certification or dependency API construction. Retained explicit early-rejection
  coverage; the reviewer confirmed this existing protection.
- Clarified phase responsibility: selected witness conflicts remain atomic
  registration failures. Full-owner export collisions fail linking, where actual
  allocated names exist. Repeating prefix/alias allocation in the registry would
  create competing naming authorities; no conflicting linked package may escape.

## Verification so far

- Focused gate `ca70b803-7aca-404b-940c-cbccfb1c1f04`: all 11 targets pass,
  including 698 ordinary C cases and 97 shared codegen cases, Java, Rust/Bazel
  linters, compile-negative, documentation and source-policy checks.
- Red run `c068cb15-a3f1-483e-b678-b96c001fb8ab` demonstrated all three
  full-export collision failures. The fourth failing test was incorrectly
  expecting the header-layout fixture to reach certification; it now correctly
  asserts the existing earlier projection rejection.
- Full invocation `322bc63c-b20c-42fe-a045-ab4e32668989` was deliberately
  interrupted when the full-export review finding superseded its inputs.
  It is not completion evidence. Historical capacity tests remain enabled.
- Full gate `c9da358d-2579-463a-bb65-25930383e883`: all 311 tests pass
  across 351 targets (456 seconds), including every historical C capacity test,
  release examples, Rust compiler frontend, shared/Java and lint gates.
- Both the Sol Extra High repair review and a fresh independent Sol Extra High
  review found no remaining core defect. Applied the fresh review's documentation
  precision correction separating exact registration identity checks from actual
  linker name allocation. An optional unprefixed-name test is deferred to the
  next fixture extension; the actual resolved-spelling check is already present.
- Authenticated imports seed existing scalar-effect and prototype availability
  checks. Actual dependency declarations remain in their owning public headers.
- All imports, even unused registrations, remain barred at final resource
  certification until 04B-04 supplies composed transitive stack evidence.
