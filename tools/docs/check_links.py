#!/usr/bin/env python3
"""Reject broken relative Markdown links in the checked-in documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path


LINK = re.compile(r"(?<!!)\[[^]]+\]\(([^)]+)\)")


def main() -> int:
    root = Path(sys.argv[1]).resolve()
    documents = sorted((root / "docs").rglob("*.md"))
    documents.extend([root / "README.md", root / "CONTRIBUTING.md"])
    failures: list[str] = []
    for document in documents:
        for match in LINK.finditer(document.read_text(encoding="utf-8")):
            raw = match.group(1).strip().strip("<>")
            destination = raw.split("#", 1)[0]
            if not destination or "://" in destination or destination.startswith("mailto:"):
                continue
            target = (document.parent / destination).resolve()
            if not target.exists():
                failures.append(f"{document.relative_to(root)} -> {raw}")
    if failures:
        print("broken documentation links:", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    required = {
        "docs/author-guide.md": [
            "bazelisk run //examples/models-and-validation:generate",
            "bazelisk test //examples/models-and-validation:all",
            "Rust host code",
            "PolyRust portable code",
            "Generated target code",
        ],
        "docs/backend-author-guide.md": ["check_backend_contract"],
    }
    for relative, snippets in required.items():
        contents = (root / relative).read_text(encoding="utf-8")
        for snippet in snippets:
            if snippet not in contents:
                print(f"missing documented/tested snippet in {relative}: {snippet}", file=sys.stderr)
                return 1
    print(f"checked {len(documents)} Markdown files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
