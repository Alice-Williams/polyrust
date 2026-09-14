"""Native nested publication boundaries, cleanup, permissions and races."""
import os
from pathlib import Path
import resource
import shlex
import signal
import subprocess
import sys

driver, publisher = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "directory-tree"
work.mkdir()
names = {"a/b/one", "a/two", "c/three"}


def environment(helper):
    return dict(os.environ, LC_ALL="C", POLYRUST_DIRECTORY_PUBLISHER=str(helper))


def clean():
    assert not list(work.glob(".polyrust-stage-*"))


def invoke(mode, destination, helper=publisher, success=False, preexec=None):
    result = subprocess.run([driver, mode, destination], capture_output=True,
        text=True, timeout=30, env=environment(helper), preexec_fn=preexec)
    assert (result.returncode == 0) == success, result.stderr
    assert not result.stdout
    clean()
    return result


def inventory(directory):
    return {str(path.relative_to(directory)): path.read_text()
        for path in directory.rglob("*") if path.is_file()}


output = work / "exact"
invoke("tree", output, success=True)
assert inventory(output) == dict.fromkeys(names, "one")
for path in [output, *output.rglob("*")]:
    assert path.stat().st_mode & 0o777 == (0o700 if path.is_dir() else 0o600)
before = output.stat().st_ino
invoke("tree", output)
assert output.stat().st_ino == before and inventory(output) == dict.fromkeys(names, "one")

for mode, diagnostic in [
    ("files", "file count"), ("directories", "directory count"),
    ("depth", "directory depth"), ("path-bytes", "path exceeds"),
    ("bytes", "payload exceeds"), ("duplicate", "unique payload"),
    ("prefix", "prefix conflict"), ("parent", "canonical relative"),
    ("absolute", "canonical relative"), ("dot", "canonical relative"),
    ("empty-component", "canonical relative"),
]:
    output = work / mode
    # An invalid helper demonstrates rejection precedes staging/helper resolution.
    result = invoke(mode, output, work / "nonexistent-helper")
    assert diagnostic in result.stderr, result.stderr
    assert not output.exists()

for mode in ["directory-failure", "file-failure"]:
    output = work / mode
    result = invoke(mode, output)
    assert "File name too long" in result.stderr, result.stderr
    assert not output.exists()


def restrict_file_size():
    signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
    resource.setrlimit(resource.RLIMIT_FSIZE, (2, 2))


# Force write_all to fail after writing two of the first file's three bytes.
output = work / "short-write"
result = invoke("tree", output, preexec=restrict_file_size)
assert "File too large" in result.stderr, result.stderr
assert not output.exists()

for timing in ["existing", "late"]:
    for kind in ["empty", "directory", "file", "symlink", "failure"]:
        output = work / (timing + "-" + kind)
        mutation = {
            "empty": 'mkdir "$2"',
            "directory": 'mkdir "$2"; printf preserved > "$2/sentinel"',
            "file": 'printf preserved > "$2"',
            "symlink": 'ln -s missing-target "$2"',
            "failure": 'exit 1',
        }[kind]
        helper = work / (timing + "-" + kind + ".sh")
        helper.write_text('#!/bin/sh\nset -eu\ntest -f "$1/a/b/one"\n'
            + mutation + '\nexec ' + shlex.quote(str(publisher)) + ' "$@"\n')
        helper.chmod(0o700)
        if timing == "existing" and kind != "failure":
            subprocess.run(["/bin/sh", "-c", mutation, "mutation", "unused", str(output)], check=True)
            helper = publisher
        invoke("tree", output, helper)
        if kind == "empty":
            assert output.is_dir() and not list(output.iterdir())
        elif kind == "directory":
            assert inventory(output) == {"sentinel": "preserved"}
        elif kind == "file":
            assert output.read_text() == "preserved"
        elif kind == "symlink":
            assert output.is_symlink() and output.readlink() == Path("missing-target")
        else:
            assert not output.exists()

not_executable = work / "not-executable"
not_executable.write_text("not executable")
invoke("tree", work / "exec-failed", not_executable)
assert not (work / "exec-failed").exists()

# Both processes finish staging before racing the real rename_noreplace syscall.
barrier = work / "barrier"
barrier.mkdir()
helper = work / "race.sh"
helper.write_text('#!/bin/sh\nset -eu\ntest -f "$1/a/b/one"\n'
    + 'barrier=' + shlex.quote(str(barrier)) + '\n'
    + 'touch "$barrier/$(basename "$1")"\n'
    + 'attempt=0\nwhile [ "$(ls -1A "$barrier" | wc -l)" -lt 2 ]; do\n'
    + '  attempt=$((attempt + 1)); [ "$attempt" -lt 500 ]; sleep 0.01\ndone\n'
    + 'exec ' + shlex.quote(str(publisher)) + ' "$@"\n')
helper.chmod(0o700)
output = work / "race"
processes = [subprocess.Popen([driver, "tree", output, token],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    env=environment(helper)) for token in ["aaa", "bbb"]]
results = [process.communicate(timeout=30) for process in processes]
assert sorted(process.returncode for process in processes) == [0, 1], results
winner = "aaa" if processes[0].returncode == 0 else "bbb"
assert inventory(output) == dict.fromkeys(names, winner)
assert all(not stdout for stdout, _ in results)
assert len(list(barrier.iterdir())) == 2
clean()
print("Nested publication: exact/one-over limits, private staging, partial-write cleanup, late destinations and a complete concurrent winner pass")
