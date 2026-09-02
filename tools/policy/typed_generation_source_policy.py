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
FORBIDDEN_MANUAL_DEPENDENCY_API = re.compile(
    r"\b(?:require_(?:java|rust|go|python|typescript|cpp|c)|"
    r"add_(?:import|include|dependency)|manual_(?:import|include))\s*\("
)
FORBIDDEN_DEPENDENCY_TEXT_SCAN = re.compile(
    r"\.(?:contains|find|starts_with)\s*\(\s*"
    r"(?:r[#]*\"|\")\s*(?:import\b|#\s*include\b|use\s+)"
)


def offenders(path: str, source: str) -> list[str]:
    # Repository Rust source keeps its cfg(test) module last. Test fixtures must
    # be able to spell deliberate violations without granting production code
    # an escape hatch.
    source = source.split("#[cfg(test)]", maxsplit=1)[0]
    findings: list[str] = []
    for label, pattern in [
        ("opaque executable enum variant", FORBIDDEN_VARIANT),
        ("opaque executable string/byte field", FORBIDDEN_FIELD),
        ("source/document conversion into executable AST", FORBIDDEN_CONVERSION),
        ("document field in executable AST", FORBIDDEN_DOCUMENT_FIELD),
        ("manual dependency attachment API", FORBIDDEN_MANUAL_DEPENDENCY_API),
        ("dependency discovery by text scan", FORBIDDEN_DEPENDENCY_TEXT_SCAN),
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
        'require_java(&mut body, "java.math.BigInteger");',
        'body.contains("import java.util.List");',
    ]
    for injected in rejected:
        if not offenders("injected.rs", injected):
            raise AssertionError(f"injection was not rejected: {injected}")
    before_tests = 'add_import(symbol);\n#[cfg(test)] mod tests {}'
    if not offenders("before_tests.rs", before_tests):
        raise AssertionError("production violation before cfg(test) was hidden")
    test_only = '#[cfg(test)] mod tests { add_import(symbol); }'
    if offenders("test_only.rs", test_only):
        raise AssertionError("deliberate cfg(test) injection was rejected")


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
