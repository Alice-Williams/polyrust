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

CFG_TEST_ATTRIBUTE = "#[cfg(test)]"
RAW_STRING_START = re.compile(r'(?:br|rb|r)(?P<hashes>#{0,255})"')
CHAR_LITERAL = re.compile(r"(?:b)?'(?:\\.|[^'\\\n])+'")


def _skip_rust_quoted(source: str, index: int) -> int | None:
    raw = RAW_STRING_START.match(source, index)
    if raw:
        terminator = '"' + raw.group("hashes")
        end = source.find(terminator, raw.end())
        return len(source) if end < 0 else end + len(terminator)
    quote = index + 1 if source.startswith(('b"', 'c"'), index) else index
    if quote < len(source) and source[quote] == '"':
        cursor = quote + 1
        while cursor < len(source):
            if source[cursor] == "\\":
                cursor += 2
            elif source[cursor] == '"':
                return cursor + 1
            else:
                cursor += 1
        return len(source)
    character = CHAR_LITERAL.match(source, index)
    return character.end() if character else None


def _cfg_test_item_end(source: str, start: int) -> int:
    """Return the end of one cfg(test)-annotated Rust item.

    This is a deliberately small lexer, not a Rust parser. It only needs to
    distinguish item braces/semicolons from delimiters inside comments and
    string/character literals. That keeps production text after a test-only
    method visible to the policy without making deliberate test fixtures fail.
    """
    cursor = start
    block_comment_depth = 0
    while cursor < len(source):
        if block_comment_depth:
            if source.startswith("/*", cursor):
                block_comment_depth += 1
                cursor += 2
            elif source.startswith("*/", cursor):
                block_comment_depth -= 1
                cursor += 2
            else:
                cursor += 1
            continue
        if source.startswith("//", cursor):
            newline = source.find("\n", cursor + 2)
            cursor = len(source) if newline < 0 else newline + 1
            continue
        if source.startswith("/*", cursor):
            block_comment_depth = 1
            cursor += 2
            continue
        quoted_end = (
            _skip_rust_quoted(source, cursor)
            if source[cursor] in "'\"rbc"
            else None
        )
        if quoted_end is not None:
            cursor = quoted_end
            continue
        if source[cursor] == ";":
            return cursor + 1
        if source[cursor] == "{":
            depth = 1
            cursor += 1
            while cursor < len(source) and depth:
                if source.startswith("//", cursor):
                    newline = source.find("\n", cursor + 2)
                    cursor = len(source) if newline < 0 else newline + 1
                    continue
                if source.startswith("/*", cursor):
                    comment_depth = 1
                    cursor += 2
                    while cursor < len(source) and comment_depth:
                        if source.startswith("/*", cursor):
                            comment_depth += 1
                            cursor += 2
                        elif source.startswith("*/", cursor):
                            comment_depth -= 1
                            cursor += 2
                        else:
                            cursor += 1
                    continue
                quoted_end = (
                    _skip_rust_quoted(source, cursor)
                    if source[cursor] in "'\"rbc"
                    else None
                )
                if quoted_end is not None:
                    cursor = quoted_end
                    continue
                if source[cursor] == "{":
                    depth += 1
                elif source[cursor] == "}":
                    depth -= 1
                cursor += 1
            return cursor
        cursor += 1
    return len(source)


def _next_cfg_test_attribute(source: str, start: int) -> int | None:
    """Find an actual cfg(test) attribute outside Rust comments and literals."""
    cursor = start
    while cursor < len(source):
        if source.startswith("//", cursor):
            newline = source.find("\n", cursor + 2)
            cursor = len(source) if newline < 0 else newline + 1
            continue
        if source.startswith("/*", cursor):
            depth = 1
            cursor += 2
            while cursor < len(source) and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            continue
        quoted_end = (
            _skip_rust_quoted(source, cursor)
            if source[cursor] in "'\"rbc"
            else None
        )
        if quoted_end is not None:
            cursor = quoted_end
            continue
        if source.startswith(CFG_TEST_ATTRIBUTE, cursor):
            return cursor
        cursor += 1
    return None


def _without_cfg_test_items(source: str) -> str:
    output = list(source)
    cursor = 0
    while True:
        start = _next_cfg_test_attribute(source, cursor)
        if start is None:
            break
        end = _cfg_test_item_end(source, start + len(CFG_TEST_ATTRIBUTE))
        for index in range(start, end):
            if output[index] != "\n":
                output[index] = " "
        cursor = end
    return "".join(output)


def offenders(path: str, source: str) -> list[str]:
    source = _without_cfg_test_items(source)
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
    after_test_item = """
#[cfg(test)]
fn deliberate_fixture() {
    let source = r###"{ add_import(symbol); }"###;
}
add_import(production_symbol);
"""
    findings = offenders("after_test_item.rs", after_test_item)
    if len(findings) != 1 or "manual dependency attachment API" not in findings[0]:
        raise AssertionError("production violation after cfg(test) item was hidden")
    for name, decoy in [
        ("string", 'const MARKER: &str = "#[cfg(test)]";'),
        ("raw_string", 'const MARKER: &str = r#"#[cfg(test)]"#;'),
        ("line_comment", "// #[cfg(test)]"),
        ("block_comment", "/* outer /* #[cfg(test)] */ marker */"),
    ]:
        findings = offenders(
            f"cfg_decoy_{name}.rs", f"{decoy}\nadd_import(production_symbol);"
        )
        if len(findings) != 1 or "manual dependency attachment API" not in findings[0]:
            raise AssertionError(f"cfg(test) text in {name} hid production code")


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
