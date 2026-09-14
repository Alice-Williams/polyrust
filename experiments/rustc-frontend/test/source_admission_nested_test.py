"""Nested HIR reference lists must stop inside the list, not at a later item."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    c_adapter, java_adapter = sys.argv[1:]
    work = Path(os.environ["TEST_TMPDIR"]) / "nested-admission"
    work.mkdir()
    prefix = "#![allow(dead_code)]\npub fn score(value: i32) -> i32 { value }\n"
    cases = [
        ("module", "mod huge {", "const V{index}: () = ();", "}"),
        ("trait", "trait Huge {", "fn item{index}();", "}"),
        ("impl", "struct Huge; impl Huge {", "fn item{index}() {}", "}"),
        ("foreign", 'unsafe extern "C" {', "fn item{index}();", "}"),
    ]
    for case, opening, item, closing in cases:
        for count in [16, 100001]:
            source = work / f"{case}_{count}.rs"
            source.write_text(prefix + opening + "\n" + "\n".join(
                item.replace("{index}", str(index)) for index in range(count)
            ) + "\n" + closing + "\n")
            for language, adapter, filename in [("c", c_adapter, "generated.c"), ("java", java_adapter, "Generated.java")]:
                output = work / f"{case}-{count}-{language}" / filename
                output.parent.mkdir()
                result = subprocess.run([adapter, str(source), str(output)],
                                        capture_output=True, text=True, check=False, timeout=180)
                if case == "foreign":
                    # Production pins -Funsafe-code: even an unused unsafe extern
                    # block is rejected by rustc, before admission can run. Keep
                    # that stronger boundary instead of weakening test config.
                    assert result.returncode != 0 and not output.exists(), language
                    assert "unsafe extern" in result.stderr and "unsafe-code" in result.stderr, result.stderr
                    assert "unsupported Rust:" not in result.stderr, result.stderr
                elif count == 16:
                    assert result.returncode == 0 and output.is_file(), (case, language, result.stderr)
                else:
                    assert result.returncode != 0 and not output.exists(), (case, language)
                    assert "unsupported Rust:" in result.stderr, result.stderr
                    # A later outer-item budget failure is NOT sufficient evidence.
                    assert "Rust source admission nested-reference budget exceeded" in result.stderr, result.stderr
    print("C/Java module/trait/impl lists stop at their budget; small controls pass; foreign lists reject earlier in rustc")


if __name__ == "__main__":
    main()
