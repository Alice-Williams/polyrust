"""Actual Rust publisher cleanup, validation and late-destination refusal."""
import os
from pathlib import Path
import shlex
import subprocess
import sys

driver, publisher = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "publication"
work.mkdir()


def run(mode, output, helper=publisher, success=False):
    result = subprocess.run([driver, mode, output], capture_output=True, text=True,
        env=dict(os.environ, POLYRUST_DIRECTORY_PUBLISHER=str(helper)), timeout=30)
    assert (result.returncode == 0) == success, result.stderr
    assert not result.stdout and not list(work.glob(".polyrust-stage-*"))
    return result


for mode, count in [("single", 3), ("bundle", 4)]:
    output = work / mode
    run(mode, output, success=True)
    assert {path.name: path.read_text() for path in output.iterdir()} == {
        f"file{index}": f"contents{index}" for index in range(count)}
    before = output.stat().st_ino
    run(mode, output)
    assert output.stat().st_ino == before

for mode in ["duplicate", "nested", "write-failure", "wrong-single-count", "zero-members", "too-many-members"]:
    output = work / mode
    run(mode, output)
    assert not output.exists()
assert not (work / "escape").exists()

for mode in ["single", "bundle"]:
    for kind in ["empty", "directory", "file", "symlink", "failure"]:
        output = work / ("late-" + mode + "-" + kind)
        helper = work / (mode + "-" + kind + ".sh")
        mutation = {
            "empty": 'mkdir "$2"',
            "directory": 'mkdir "$2"; printf preserved > "$2/sentinel"',
            "file": 'printf preserved > "$2"',
            "symlink": 'ln -s missing-target "$2"',
            "failure": 'exit 1',
        }[kind]
        helper.write_text('#!/bin/sh\nset -eu\ntest -f "$1/file0"\n' + mutation + '\nexec ' + shlex.quote(str(publisher)) + ' "$@"\n')
        helper.chmod(0o700)
        run(mode, output, helper)
        if kind == "empty":
            assert output.is_dir() and not list(output.iterdir())
        elif kind == "directory":
            assert {path.name: path.read_text() for path in output.iterdir()} == {"sentinel": "preserved"}
        elif kind == "file":
            assert output.read_text() == "preserved"
        elif kind == "symlink":
            assert output.is_symlink() and output.readlink() == Path("missing-target")
        else:
            assert not output.exists()

not_executable = work / "not-executable"
not_executable.write_text("not executable")
run("bundle", work / "exec-failed", not_executable)
assert not (work / "exec-failed").exists()
print("Rust publication: late destinations preserved, partial writes/helper failures cleaned, count/path guards retained")
