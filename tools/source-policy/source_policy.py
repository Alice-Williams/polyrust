#!/usr/bin/env python3
"""Enforce renderer-only dependency directives in backend source templates."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


POLICY_ROOT = re.compile(r"^(?:crates/backend-[^/]+|examples/external-backend)/")
TARGET_SUFFIXES = {".c", ".cc", ".cpp", ".h", ".hpp", ".java", ".go", ".py", ".ts", ".js"}
DIRECTIVE = re.compile(
    r"(?m)^\s*(?:"
    r"#\s*include\b|"
    r"(?:pub\s+)?use\s+[A-Za-z_:*{]|"
    r"mod\s+[A-Za-z_][A-Za-z0-9_]*\s*;|"
    r"import\s+(?:type\s+)?[^\s]|"
    r"from\s+[^\s]+\s+import\b|"
    r"export\s+(?:type\s+)?(?:\*|\{).*\s+from\b"
    r")"
)

# Native consumer fixtures are handwritten inputs to target compilers, not
# generated body templates. Every exception is path-exact and reviewed here.
FIXTURE_ALLOWLIST = {
    "crates/backend-c/test/abi_shapes_test.c",
    "crates/backend-c/test/c_consumer_test.c",
    "crates/backend-c/test/runtime_ownership_test.c",
    "crates/backend-c/test/runtime_template_preamble.h",
    "crates/backend-cpp/test/cpp_consumer_test.cc",
    "crates/backend-java/test/JavaConsumerTest.java",
}


class PolicyError(RuntimeError):
    pass


@dataclass(frozen=True)
class RustString:
    start: int
    line: int
    value: str


def _blank(mask: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if mask[index] != "\n":
            mask[index] = " "


def _decode_rust_string(value: str) -> str:
    # Dependency directives use ASCII spelling. Decoding the layout escapes is
    # sufficient and avoids interpreting arbitrary target-language backslashes.
    sentinel = "\0POLYRUST_BACKSLASH\0"
    return (
        value.replace("\\\\", sentinel)
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace('\\"', '"')
        .replace(sentinel, "\\")
    )


def rust_strings_and_mask(source: str) -> tuple[list[RustString], str]:
    strings: list[RustString] = []
    mask = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end < 0 else end
            _blank(mask, index, end)
            index = end
            continue
        if source.startswith("/*", index):
            start = index
            depth = 1
            index += 2
            while index < len(source) and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            _blank(mask, start, index)
            continue

        if source[index] == "'":
            start = index
            end = index + 1
            if end < len(source) and source[end] == "\\":
                end += 1
                if end < len(source) and source.startswith("u{", end):
                    closing = source.find("}", end + 2)
                    end = len(source) if closing < 0 else closing + 1
                else:
                    end += 1
            else:
                end += 1
            if end < len(source) and source[end] == "'":
                end += 1
                _blank(mask, start, end)
                index = end
                continue

        raw = re.match(r"(?:b|c)?r(#{0,255})\"", source[index:])
        if raw:
            start = index
            hashes = raw.group(1)
            content_start = index + raw.end()
            terminator = '"' + hashes
            content_end = source.find(terminator, content_start)
            if content_end < 0:
                raise PolicyError(f"unterminated Rust raw string at line {source.count(chr(10), 0, start) + 1}")
            end = content_end + len(terminator)
            strings.append(
                RustString(start, source.count("\n", 0, start) + 1, source[content_start:content_end])
            )
            _blank(mask, start, end)
            index = end
            continue

        if source[index] == '"':
            start = index
            index += 1
            content = []
            while index < len(source):
                if source[index] == "\\" and index + 1 < len(source):
                    content.extend(source[index : index + 2])
                    index += 2
                elif source[index] == '"':
                    index += 1
                    break
                else:
                    content.append(source[index])
                    index += 1
            else:
                raise PolicyError(f"unterminated Rust string at line {source.count(chr(10), 0, start) + 1}")
            strings.append(
                RustString(
                    start,
                    source.count("\n", 0, start) + 1,
                    _decode_rust_string("".join(content)),
                )
            )
            _blank(mask, start, index)
            continue
        index += 1
    return strings, "".join(mask)


def _brace_span(mask: str, opening: int) -> tuple[int, int]:
    depth = 0
    for index in range(opening, len(mask)):
        if mask[index] == "{":
            depth += 1
        elif mask[index] == "}":
            depth -= 1
            if depth == 0:
                return opening, index + 1
    raise PolicyError("unclosed Rust block while enforcing source policy")


def _function_spans(mask: str, name: str) -> list[tuple[int, int]]:
    spans = []
    for match in re.finditer(rf"\bfn\s+{re.escape(name)}\s*\(", mask):
        opening = mask.find("{", match.end())
        if opening < 0:
            raise PolicyError(f"function {name} has no body")
        spans.append(_brace_span(mask, opening))
    return spans


def _language_renderer_spans(mask: str) -> list[tuple[int, int]]:
    spans = []
    pattern = re.compile(
        r"\bimpl\b[^{};]*\bLanguageRenderer\s*<[^{}]*>\s+for\s+[^{};]+\{"
    )
    for match in pattern.finditer(mask):
        spans.append(_brace_span(mask, mask.rfind("{", match.start(), match.end())))
    return spans


def _test_module_spans(mask: str) -> list[tuple[int, int]]:
    spans = []
    pattern = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{")
    for match in pattern.finditer(mask):
        spans.append(_brace_span(mask, mask.rfind("{", match.start(), match.end())))
    return spans


def _inside(index: int, spans: list[tuple[int, int]]) -> bool:
    return any(start <= index < end for start, end in spans)


def rust_template_offenders(relative: str, source: str) -> list[str]:
    strings, mask = rust_strings_and_mask(source)
    renderer_impls = _language_renderer_spans(mask)
    allowed = [
        span
        for span in _function_spans(mask, "render_imports")
        if _inside(span[0], renderer_impls)
    ] + _test_module_spans(mask)
    return [
        f"{relative}:{string.line}: dependency directive outside render_imports"
        for string in strings
        if not _inside(string.start, allowed) and DIRECTIVE.search(string.value)
    ]


def target_template_offenders(relative: str, source: str) -> list[str]:
    if not DIRECTIVE.search(source):
        return []
    if relative in FIXTURE_ALLOWLIST:
        return []
    line = source[: DIRECTIVE.search(source).start()].count("\n") + 1
    return [f"{relative}:{line}: dependency directive in generated body template"]


def verify(root: Path) -> None:
    offenders: list[str] = []
    seen_fixtures: set[str] = set()
    for path in sorted(root.glob("crates/backend-*/**/*")):
        if not path.is_file():
            continue
        relative = path.relative_to(root).as_posix()
        if not POLICY_ROOT.match(relative):
            continue
        if relative in FIXTURE_ALLOWLIST:
            seen_fixtures.add(relative)
        if path.suffix == ".rs" and "/src/" in relative:
            offenders.extend(rust_template_offenders(relative, path.read_text(encoding="utf-8")))
        elif path.suffix in TARGET_SUFFIXES:
            offenders.extend(target_template_offenders(relative, path.read_text(encoding="utf-8")))
    missing = sorted(FIXTURE_ALLOWLIST - seen_fixtures)
    if missing:
        offenders.append(f"fixture allowlist entries are missing from policy inputs: {missing}")
    if offenders:
        raise PolicyError("\n".join(offenders))


def self_test() -> None:
    allowed = '''
impl LanguageRenderer<String> for Renderer {
    fn render_imports(&self) -> String { "import allowed".into() }
}
#[cfg(test)] mod tests { const EXPECTED: &str = "#include <allowed>"; }
const BODY: &str = "plain body";
'''
    if rust_template_offenders("allowed.rs", allowed):
        raise AssertionError("renderer or unit-test spelling was rejected")
    injected = 'const BODY: &str = "body\\nimport forbidden";'
    if not rust_template_offenders("injected.rs", injected):
        raise AssertionError("Rust body directive injection was not detected")
    counterfeit = 'fn render_imports() -> &\'static str { "import forbidden" }'
    if not rust_template_offenders("counterfeit.rs", counterfeit):
        raise AssertionError("non-renderer function inherited renderer permission")
    if not target_template_offenders("crates/backend-c/src/injected.c", "#include <bad.h>\n"):
        raise AssertionError("target template directive injection was not detected")
    fixture = "crates/backend-c/test/c_consumer_test.c"
    if target_template_offenders(fixture, '#include "generated.h"\n'):
        raise AssertionError("path-exact native fixture exception was rejected")
    if not target_template_offenders(fixture + ".copy", '#include "generated.h"\n'):
        raise AssertionError("fixture exception was not path-exact")


def main() -> int:
    try:
        if sys.argv[1:] == ["self-test"]:
            self_test()
            print("source policy deliberate-injection checks passed")
        elif len(sys.argv) == 3 and sys.argv[1] == "verify":
            verify(Path(sys.argv[2]).resolve())
            print("renderer-only dependency directive policy passed")
        else:
            raise SystemExit("usage: source_policy.py verify ROOT | self-test")
    except PolicyError as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
