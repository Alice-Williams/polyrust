#!/usr/bin/env python3
"""Prove that every static capability has one marker module and catalogue row."""

from __future__ import annotations

import re
import sys
from pathlib import Path


MODULE = re.compile(r"(?m)^mod ([a-z][a-z0-9_]*);$")
REEXPORT = re.compile(
    r"(?m)^pub use ([a-z][a-z0-9_]*)::([A-Z][A-Za-z0-9]*);$"
)
MARKER = re.compile(r"(?m)^pub enum ([A-Z][A-Za-z0-9]*) \{\}$")
CATALOGUE = re.compile(r"(?s)capability_catalogue!\((.*?)\);")
INVENTORY = re.compile(
    r"(?s)const CAPABILITY_CONSTRUCTOR_INVENTORY:.*?= \[(.*?)\];"
)
PRESENCE_ASSERTION = re.compile(r"(?s)assert_capabilities!\((.*?)\);")


def fail(message: str) -> None:
    raise SystemExit(f"capability layout: {message}")


def verify_shared(root: Path) -> int:
    source = (root / "mod.rs").read_text(encoding="utf-8")
    modules = MODULE.findall(source)
    exports = REEXPORT.findall(source)
    if not modules:
        fail("mod.rs declares no capability modules")
    if len(modules) != len(set(modules)):
        fail("mod.rs contains duplicate capability module declarations")
    if len(exports) != len(modules):
        fail("every capability module must have exactly one public re-export")
    if {module for module, _ in exports} != set(modules):
        fail("module declarations and public re-exports must name the same modules")

    files = sorted(path.stem for path in root.glob("*.rs") if path.name != "mod.rs")
    if sorted(modules) != files:
        fail(f"module/file mismatch: modules={sorted(modules)!r}, files={files!r}")

    marker_by_module: dict[str, str] = {}
    for module in modules:
        path = root / f"{module}.rs"
        markers = MARKER.findall(path.read_text(encoding="utf-8"))
        if len(markers) != 1:
            fail(f"{path.name} must define exactly one public empty marker enum")
        marker_by_module[module] = markers[0]

    for module, exported in exports:
        if marker_by_module[module] != exported:
            fail(
                f"{module}.rs defines {marker_by_module[module]}, "
                f"but mod.rs exports {exported}"
            )

    catalogues = CATALOGUE.findall(source)
    if len(catalogues) != 1:
        fail("mod.rs must contain exactly one closed capability_catalogue! invocation")
    catalogue = re.findall(r"\b[A-Z][A-Za-z0-9]*\b", catalogues[0])
    exported = [name for _, name in exports]
    if len(catalogue) != len(set(catalogue)):
        fail("the closed catalogue contains duplicate capability markers")
    if set(catalogue) != set(exported):
        fail(f"catalogue/export mismatch: catalogue={catalogue!r}, exports={exported!r}")

    typed_source = (root.parent / "typed_program.rs").read_text(encoding="utf-8")
    inventories = INVENTORY.findall(typed_source)
    if len(inventories) != 1:
        fail("typed_program.rs must contain exactly one constructor inventory")
    inventory_names = re.findall(
        r'\(\s*"([A-Z][A-Za-z0-9]*)"\s*,', inventories[0]
    )
    if len(inventory_names) != len(set(inventory_names)):
        fail("typed constructor inventory contains duplicate capability rows")
    if set(inventory_names) != set(catalogue):
        fail(
            "typed constructor inventory/catalogue mismatch: "
            f"inventory={inventory_names!r}, catalogue={catalogue!r}"
        )

    asserted_names: set[str] = set()
    for assertion in PRESENCE_ASSERTION.findall(typed_source):
        asserted_names.update(re.findall(r"\b[A-Z][A-Za-z0-9]*\b", assertion))
    asserted_names &= set(catalogue)
    if asserted_names != set(catalogue):
        fail(
            "every catalogued capability needs a compiled ContainsCapability assertion: "
            f"asserted={sorted(asserted_names)!r}, catalogue={sorted(catalogue)!r}"
        )

    return len(catalogue)


def verify_java(root: Path) -> tuple[int, set[str]]:
    source = (root / "mod.rs").read_text(encoding="utf-8")
    ignored = {"dispatch", "support"}
    modules = [name for name in MODULE.findall(source) if name not in ignored]
    files = sorted(
        path.stem
        for path in root.glob("*.rs")
        if path.stem not in ignored | {"mod"}
    )
    if sorted(modules) != files:
        fail(f"Java module/file mismatch: modules={sorted(modules)!r}, files={files!r}")

    registrations = re.findall(
        r"(?m)^\s*\.support\(observed\((Java[A-Za-z0-9]+)\)\)$", source
    )
    if len(registrations) != len(set(registrations)):
        fail("Java registration contains duplicate mappings")

    mappings: dict[str, str] = {}
    macro_mapping = re.compile(
        r"java_(?:ast|intrinsic)_mapping!\("
        r"\s*(Java[A-Za-z0-9]+)\s*,\s*([A-Z][A-Za-z0-9]+)"
    )
    direct_name = re.compile(r"(?m)^pub struct (Java[A-Za-z0-9]+);$")
    direct_capability = re.compile(
        r"(?m)^\s*type Capability = ([A-Z][A-Za-z0-9]+);$"
    )
    for module in modules:
        module_source = (root / f"{module}.rs").read_text(encoding="utf-8")
        macro_matches = macro_mapping.findall(module_source)
        if macro_matches:
            if len(macro_matches) != 1:
                fail(f"{module}.rs must invoke exactly one mapping macro")
            mapping, capability = macro_matches[0]
        else:
            names = direct_name.findall(module_source)
            capabilities = direct_capability.findall(module_source)
            if len(names) != 1 or len(capabilities) != 1:
                fail(f"{module}.rs must define exactly one Java capability mapping")
            mapping, capability = names[0], capabilities[0]
        if mapping in mappings:
            fail(f"Java mapping {mapping} is defined more than once")
        mappings[mapping] = capability

    if set(registrations) != set(mappings):
        fail(
            "Java mapping/registration mismatch: "
            f"mappings={sorted(mappings)!r}, registrations={sorted(registrations)!r}"
        )
    unsupported = re.findall(
        r"(?m)^\s*\.unsupported::<([A-Z][A-Za-z0-9]+)>\(\)$", source
    )
    if len(unsupported) != len(set(unsupported)):
        fail("Java registration contains duplicate unsupported decisions")
    supported_capabilities = set(mappings.values())
    if supported_capabilities & set(unsupported):
        fail("Java marks a capability as both supported and unsupported")
    return len(mappings), supported_capabilities | set(unsupported)


def main() -> None:
    shared_count = verify_shared(Path(sys.argv[1]))
    java_count, java_decisions = verify_java(Path(sys.argv[2]))
    shared_source = (Path(sys.argv[1]) / "mod.rs").read_text(encoding="utf-8")
    shared_catalogue = CATALOGUE.findall(shared_source)
    shared_names = set(re.findall(r"\b[A-Z][A-Za-z0-9]*\b", shared_catalogue[0]))
    if java_decisions != shared_names:
        fail(
            "Java registry must decide every shared capability: "
            f"decisions={sorted(java_decisions)!r}, catalogue={sorted(shared_names)!r}"
        )
    print(
        f"capability layout: {shared_count} shared markers and "
        f"{java_count} Java mappings verified one-per-file; "
        f"{len(java_decisions) - java_count} explicitly unsupported"
    )


if __name__ == "__main__":
    main()
