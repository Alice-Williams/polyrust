"""Compiler/admission failures cannot create or overwrite Java artifacts."""
import os
from pathlib import Path
import subprocess
import sys


def main():
    adapter, fixture = sys.argv[1:]
    fixtures = Path(fixture).parent
    work = Path(os.environ["TEST_TMPDIR"]) / "java-rejections"
    work.mkdir()
    cases = {
        "use_after_move": "E0382", "escaping_borrow": "E0515",
        "conflicting_borrow": "E0506", "wrong_type": "E0308", "unstable": "E0554",
        "foreign_abi": "safe non-variadic ordinary Rust signatures",
        "unsupported": "generic or mismatched direct callee identity",
        "packed": "custom Rust record representations", "aligned": "custom Rust record representations",
        **{case: "Rust type alias uses require an unimplemented provenance mapping" for case in
           ["alias_signature", "alias_field", "alias_local", "alias_external", "alias_constructor"]},
    }
    for case, diagnostic in cases.items():
        output = work / case / "Generated.java"
        output.parent.mkdir()
        command = [adapter, str(fixtures / (case + ".rs")), str(output)]
        for existing in [False, True]:
            if existing:
                output.write_bytes(b"existing output must survive\x00\xff")
            result = subprocess.run(command, capture_output=True, text=True, check=False)
            assert result.returncode != 0, (case, "unexpected admission")
            assert diagnostic in result.stderr, (case, result.stderr)
            if existing:
                assert output.read_bytes() == b"existing output must survive\x00\xff", case
            else:
                assert not output.exists(), case
    print("14 compiler/admission cases reject; absent and existing outputs preserved")
    valid = work / "valid.rs"
    valid.write_text("pub fn score(value: i32) -> i32 { value }\n")
    for name in ["Wrong.java", "generated.java", "Generated.txt"]:
        output = work / name
        for existing in [False, True]:
            if existing:
                output.write_bytes(b"preserved\x00\xff")
            result = subprocess.run([adapter, str(valid), str(output)],
                                    capture_output=True, text=True, check=False)
            assert result.returncode != 0, name
            assert result.stderr.strip() == "Java output filename must be Generated.java", result.stderr
            assert output.read_bytes() == b"preserved\x00\xff" if existing else not output.exists()
    accepted = work / "Generated.java"
    result = subprocess.run([adapter, str(valid), str(accepted)],
                            capture_output=True, text=True, check=False)
    assert result.returncode == 0 and accepted.is_file(), result.stderr
    print("Java compilation-unit filename enforced before publication; valid control accepted")


if __name__ == "__main__":
    main()
