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


def fail(message: str) -> None:
    raise SystemExit(f"capability layout: {message}")


def main() -> None:
    root = Path(sys.argv[1])
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

    print(f"capability layout: {len(catalogue)} one-file capability markers verified")


if __name__ == "__main__":
    main()
