"""Record-order/physical-path independence and preexisting output protection."""
import os
from pathlib import Path
import shutil
import subprocess


def check(adapter, work, scratch, arguments, count, expected_imports):
    import c_bundle_assertions
    output_root = work / "determinism"
    output_root.mkdir()

    def run(output, records):
        result = subprocess.run([str(value) for value in [adapter, "--bundle", output, *records]],
            capture_output=True, text=True, timeout=60, env=dict(os.environ, TMPDIR=str(scratch)))
        assert not list(scratch.iterdir()) and not list(output_root.glob(".polyrust-stage-*"))
        return result

    original = output_root / "original"
    result = run(original, arguments)
    assert result.returncode == 0, result.stderr
    expected = c_bundle_assertions.check(original, count, expected_imports)
    groups = []
    for value in arguments[2:]:
        if value == "--crate":
            groups.append([])
        groups[-1].append(value)
    reordered = arguments[:2] + [value for group in reversed(groups) for value in group]
    result = run(output_root / "reordered", reordered)
    assert result.returncode == 0, result.stderr
    assert c_bundle_assertions.check(output_root / "reordered", count, expected_imports) == expected
    relocation = output_root / "relocated-inputs"
    relocation.mkdir()
    moved = []
    for value in arguments:
        if isinstance(value, Path):
            destination = relocation / value.name
            if value.exists():
                shutil.copyfile(value, destination)
            moved.append(destination)
        else:
            moved.append(value)
    result = run(output_root / "relocated", moved)
    assert result.returncode == 0, result.stderr
    assert c_bundle_assertions.check(output_root / "relocated", count, expected_imports) == expected
    for kind in ["empty", "directory", "file", "symlink", "dangling"]:
        output = output_root / kind
        if kind in {"empty", "directory"}:
            output.mkdir()
            if kind == "directory":
                (output / "sentinel").write_bytes(b"untouched")
        elif kind == "file":
            output.write_bytes(b"untouched")
        else:
            output.symlink_to(original if kind == "symlink" else output_root / "absent")
        before = output.lstat()
        result = run(output, arguments)
        assert result.returncode != 0 and not result.stdout
        assert output.lstat().st_ino == before.st_ino and output.lstat().st_mode == before.st_mode
        if kind == "directory":
            assert {item.name: item.read_bytes() for item in output.iterdir()} == {"sentinel": b"untouched"}
        elif kind == "empty":
            assert not list(output.iterdir())
        elif kind == "file":
            assert output.read_bytes() == b"untouched"
        else:
            assert output.is_symlink()
    assert c_bundle_assertions.check(original, count, expected_imports) == expected
