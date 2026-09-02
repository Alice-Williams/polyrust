#!/usr/bin/env python3
"""Reject semantic decisions and source inventories in certified templates."""

from __future__ import annotations

import re
import sys
from pathlib import Path


SEMANTIC_NAME = re.compile(
    r"(?i)\b(?:core_?ir|capabilit(?:y|ies)|runtime_?helper|checked_?[iu]\d+|"
    r"float_?abs|decode_?utf8|ownership|cleanup|file_?placement|"
    r"known_?(?:type|callable|method|field|constructor))\b"
)
SEMANTIC_BRANCH = re.compile(
    r"(?i){{[#^](?:if|unless|each|with)\s+"
    r"(?:feature|helper|operator|operation|capability|ownership|placement)(?:_|\b)"
)
HARDCODED_DEPENDENCY = re.compile(
    r"(?im)^\s*(?:import\s+[A-Za-z_$][\w.$]*\s*;|"
    r"\#\s*include\s*[<\"][^{}>\"]+[>\"]|"
    r"use\s+[A-Za-z_][\w:]*\s*;)"
)
INLINE_HELPER = re.compile(
    r"{{\s*(?![#/!>])([A-Za-z_][\w.]*)\s+[^}]+}}"
)
FORBIDDEN_RENDERER_LAYER = re.compile(
    r"\b(?:portable_core_ir|portable_check|CoreProgram|CapabilityRegistry|"
    r"TargetAstPackage|TargetSymbolRef|UnresolvedPackage)\b|\.unresolved\s*\("
)


def offenders(path: str, source: str) -> list[str]:
    if path.endswith(".rs"):
        production = "\n".join(
            line for line in source.splitlines() if not line.lstrip().startswith("///")
        )
        return [
            f"{path}:{production.count(chr(10), 0, match.start()) + 1}: "
            "renderer imports or inspects a pre-resolution layer"
            for match in FORBIDDEN_RENDERER_LAYER.finditer(production)
        ]
    findings: list[str] = []
    for label, pattern in [
        ("semantic identifier in template", SEMANTIC_NAME),
        ("semantic branch in template", SEMANTIC_BRANCH),
        ("hard-coded import/include/use inventory", HARDCODED_DEPENDENCY),
        ("custom inline helper invocation", INLINE_HELPER),
    ]:
        for match in pattern.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            findings.append(f"{path}:{line}: {label}")
    if "\r" in source or "\0" in source:
        findings.append(f"{path}:1: non-canonical template encoding")
    return findings


def self_test() -> None:
    allowed = """package {{package_name}};
{{#each imports}}import {{this}};
{{/each}}{{#each declarations}}{{this}}
{{/each}}"""
    if offenders("allowed.hbs", allowed):
        raise AssertionError("generic grammar skeleton was rejected")
    rejected = [
        "{{#if feature_float_abs}}return abs(value);{{/if}}",
        "{{RuntimeHelper.DecodeUtf8}}",
        "import java.util.List;",
        "#include <string.h>",
        "use std::collections::HashMap;",
        "{{semantic_helper value}}",
        "bad\r\n",
        "bad\0",
    ]
    for injected in rejected:
        if not offenders("injected.hbs", injected):
            raise AssertionError(f"template injection was not rejected: {injected!r}")
    if offenders("allowed.rs", "use crate::{LinkedFile, LinkedTargetPackage};"):
        raise AssertionError("resolved renderer input was rejected")
    if not offenders("injected.rs", "use portable_core_ir::CoreProgram;"):
        raise AssertionError("pre-resolution renderer input was not rejected")


def main() -> int:
    self_test()
    if len(sys.argv) < 2 or sys.argv[1] != "verify":
        raise SystemExit("usage: template_policy.py verify [TEMPLATE...]")
    findings: list[str] = []
    for source_path in sorted(sys.argv[2:]):
        path = Path(source_path)
        findings.extend(offenders(path.as_posix(), path.read_text(encoding="utf-8")))
    if findings:
        print("\n".join(findings), file=sys.stderr)
        return 1
    print("certified template policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
