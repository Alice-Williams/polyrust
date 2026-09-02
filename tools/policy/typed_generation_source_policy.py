#!/usr/bin/env python3
"""Reject opaque executable source escape hatches in production typed ASTs."""

from __future__ import annotations

import re
import sys
from pathlib import Path


FORBIDDEN_VARIANT = re.compile(
    r"(?m)(?:^|[,{])\s*"
    r"(?:Raw|Verbatim|Snippet|TokenStream|SourceText|ExecutableCode)\s*(?:[({,])"
)
FORBIDDEN_FIELD = re.compile(
    r"(?m)(?:^|[,{])\s*(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:code|text|tokens|token_stream|snippet|verbatim|source_code|executable_code)\s*:\s*"
    r"(?:String|&\s*(?:'\w+\s+)?str|Vec\s*<\s*u8\s*>)"
)
FORBIDDEN_CONVERSION = re.compile(
    r"(?s)impl(?:\s*<[^>]*>)?\s+(?:From|TryFrom)\s*<\s*"
    r"(?:String|&\s*(?:'\w+\s+)?str|Document)\s*>\s+for\s+"
    r"\w*(?:Expr|Expression|Stmt|Statement|Item|Package)\w*"
)
FORBIDDEN_DOCUMENT_FIELD = re.compile(
    r"(?m)(?:^|[,{])\s*(?:pub(?:\([^)]*\))?\s+)?"
    r"\w+\s*:\s*(?:Code)?Document\b"
)


def offenders(path: str, source: str) -> list[str]:
    findings: list[str] = []
    for label, pattern in [
        ("opaque executable enum variant", FORBIDDEN_VARIANT),
        ("opaque executable string/byte field", FORBIDDEN_FIELD),
        ("source/document conversion into executable AST", FORBIDDEN_CONVERSION),
        ("document field in executable AST", FORBIDDEN_DOCUMENT_FIELD),
    ]:
        for match in pattern.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            findings.append(f"{path}:{line}: {label}")
    return findings


def self_test() -> None:
    allowed = """
enum Expr { Literal(i64), Call { target: KnownCallable } }
struct GeneratedType { name: String, source: SourceRef }
struct AstViolation { message: String }
"""
    if offenders("allowed.rs", allowed):
        raise AssertionError("typed metadata was rejected")

    rejected = [
        "enum Expr { Raw(String) }",
        "struct Expr { code: String }",
        "struct Expr { body: CodeDocument }",
        "impl From<String> for JavaExpression {}",
    ]
    for injected in rejected:
        if not offenders("injected.rs", injected):
            raise AssertionError(f"injection was not rejected: {injected}")


def main() -> int:
    self_test()
    if not sys.argv[1:] or sys.argv[1] != "verify" or len(sys.argv) < 3:
        raise SystemExit("usage: typed_generation_source_policy.py verify SOURCE...")
    findings: list[str] = []
    for source_path in sorted(sys.argv[2:]):
        path = Path(source_path)
        findings.extend(offenders(path.as_posix(), path.read_text(encoding="utf-8")))
    if findings:
        print("\n".join(findings), file=sys.stderr)
        return 1
    print("typed-generation opaque-source policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
