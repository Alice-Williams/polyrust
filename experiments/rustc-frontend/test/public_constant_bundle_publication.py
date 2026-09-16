"""Whole-owner reconciliation and atomic publication of constant bundles."""
import os
from pathlib import Path
import subprocess
import sys


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in root.rglob("*") if p.is_file()}


def main():
    c, java, collision, inventory, data, data_metadata, mixed, mixed_metadata = [
        Path(p).resolve() for p in sys.argv[1:]]
    work = Path(os.environ["TEST_TMPDIR"]) / "constant-bundle-publication"
    work.mkdir()
    scratch = work / "scratch"
    scratch.mkdir()
    root = ["--root", "proof.public.constant.values"]
    leaf = ["--crate", "constant_data", "proof.public.constant.data",
            data, data.name, data_metadata]
    consumer = ["--crate", "constant_values", "proof.public.constant.values",
                mixed, mixed.name, mixed_metadata, "--input", data, data.name,
                "--dependency", "unused_constants", "proof.public.constant.data"]
    records = root + leaf + consumer
    originals = {p: p.read_bytes() for p in [data, data_metadata, mixed, mixed_metadata]}

    def run(adapter, tail, success):
        result = subprocess.run([str(v) for v in [adapter, *tail]], capture_output=True,
                                text=True, timeout=90, env=dict(os.environ, TMPDIR=str(scratch)))
        assert (result.returncode == 0) == success, result.stderr
        assert not list(scratch.iterdir()) and not list(work.glob(".polyrust-stage-*"))
        assert originals == {p: p.read_bytes() for p in originals}
        return result

    run(inventory, ["--check-crates", *records], True)
    for name, adapter in [("c", c), ("java", java)]:
        original = work / name
        run(adapter, ["--bundle", original, *records], True)
        expected = files(original)
        assert len(expected) == (7 if name == "c" else 5)
        reordered = work / (name + "-reordered")
        run(adapter, ["--bundle", reordered, *root, *consumer, *leaf], True)
        assert files(reordered) == expected

        for kind in ["empty", "directory", "file", "symlink"]:
            output = work / (name + "-" + kind)
            if kind in ["empty", "directory"]:
                output.mkdir()
                if kind == "directory":
                    (output / "sentinel").write_bytes(b"unchanged\x00\xff")
            elif kind == "file":
                output.write_bytes(b"unchanged\x00\xff")
            else:
                output.symlink_to(original, target_is_directory=True)
            before = output.lstat()
            run(adapter, ["--bundle", output, *records], False)
            assert (output.lstat().st_ino, output.lstat().st_mode) == (before.st_ino, before.st_mode)
            if kind == "directory":
                assert files(output) == {"sentinel": b"unchanged\x00\xff"}
            elif kind == "empty":
                assert not list(output.iterdir())
            elif kind == "file":
                assert output.read_bytes() == b"unchanged\x00\xff"
            else:
                assert output.is_symlink()
            assert files(original) == expected

        for existing in [False, True]:
            output = work / (name + "-missing-" + str(existing))
            if existing:
                output.mkdir()
                (output / "sentinel").write_bytes(b"preserved")
            result = run(adapter, ["--bundle", output, *root, *consumer], False)
            assert "dependency" in result.stderr
            assert output.exists() == existing
            if existing:
                assert files(output) == {"sentinel": b"preserved"}

    for existing in [False, True]:
        output = work / ("collision-" + str(existing))
        if existing:
            output.mkdir()
            (output / "sentinel").write_bytes(b"preserved")
        result = run(collision, ["--bundle", output, *records], False)
        assert "bundle has duplicate external definitions" in result.stderr
        assert output.exists() == existing
        if existing:
            assert files(output) == {"sentinel": b"preserved"}
    print("Constant bundle inventory, deterministic order, owner failures, collisions and atomic existing-output guards")


if __name__ == "__main__":
    main()
