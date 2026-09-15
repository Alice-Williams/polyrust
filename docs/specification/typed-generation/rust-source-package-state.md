# Rust-source package and function state

- Status: normative; implemented in M35-03A-02F-02B-04A
- Parent: [public scalar constants](rust-public-constants.md)

## Boundary

Package registration precedes body lowering. Package state must be constructible
with zero function declarations and has no TypeckResults, current function,
current source body, local bindings or control scope. An empty body list returns
that state unchanged. This internal operation is not public export admission and
does not certify an incomplete package.

A function Reader is created only from a registered LocalDefId. It obtains
TypeckResults for that exact function and creates fresh parameter bindings,
temporary names, lexical scopes and expression depth. Missing registrations
produce diagnostics. Completing a body returns persistent package state; no
function-local binding or scope can be reused by another function.

Registry/builder authority must be moved, not reconstructed from names, cloned
into a new registry, or replaced with default state. Record declarations and
foreign witnesses remain attached to the same package through every body.
Ordering continues to follow the existing stable declaration inventory.

## C state

Persistent state owns CRegistry, the generated source CFileRef, registered local
and foreign functions, discovered CStructRef records, their CFileItem declarations,
source-origin cache and selected executable capability bindings.

The Reader additionally owns the current CFunctionRef and LocalDefId, checked
compiler types, parameters, local places, control scopes, active scope, prelude
and local/temporary/scope counters. Body lowering emits the ordinary typed
prototype and definition. Public prototypes remain in the registered header;
private prototypes and definitions remain in the source. C ownership, layout,
sequencing, numeric, indexing and storage checks still run on assembled files.

## Java state

Persistent state owns TargetAstBuilder<JavaDialect>, registered callables,
authenticated foreign callables, discovered record declarations, source-origin
cache, executable capability bindings and the remaining package expression budget.

The Reader additionally owns checked compiler types, local places, prelude,
local-name counter, active lexical scope, scope ancestry and expression depth.
Probe observations are function-local. Completing a function returns its typed
JavaMethod and the persistent state. The facade constructor and source file are
assembled outside any function Reader.

The existing 100,000-expression package budget is initialized once per package,
decremented during body lowering, and never reset between functions. Existing
per-expression depth limits remain function-local. Creating a new Reader must
not enlarge either budget.

## Proof and migration boundary

Compiler probes exercise the production empty-body path and registered-function
lifecycle. They check authority preservation, fresh local state and Java budget
carry-over. Existing native compiler comparisons and actual AST/production-byte
comparisons remain required for selected-entry and public-package generation.

This state separation does not broaden supported syntax. Public constants are
admitted only when their subsequent declaration/read mappings, complete export
inventory, typed certificates and native proof exist. No dummy source function,
optional checked-types fallback, runtime file or unchecked renderer is introduced.
