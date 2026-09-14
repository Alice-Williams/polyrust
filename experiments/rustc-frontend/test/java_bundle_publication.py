"""Real compiler-to-Java publication: no partial tree on rejection or races."""
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys

adapter, transaction_probe, publisher, *artifacts = [Path(value).resolve() for value in sys.argv[1:]]
work = Path(os.environ["TEST_TMPDIR"]) / "java-bundle-publication"
work.mkdir()
scratch = work / "scratch"
scratch.mkdir()
owners = ["leaf", "left", "right", "root"]
dependencies = {"leaf": [], "left": ["leaf"], "right": ["leaf"], "root": ["left", "right"]}
records = ["--root", "native.diamond.root.v1"]
for index, name in enumerate(owners):
    source, metadata = artifacts[index * 2:index * 2 + 2]
    records += ["--crate", name, f"native.diamond.{name}.v1", source, source.name, metadata]
    for dependency in dependencies[name]:
        records += ["--dependency", dependency, f"native.diamond.{dependency}.v1"]
before = {path: path.read_bytes() for path in artifacts}


def environment(helper=publisher):
    return dict(os.environ, TMPDIR=str(scratch), LC_ALL="C", POLYRUST_DIRECTORY_PUBLISHER=str(helper))


def clean():
    assert not list(work.glob(".polyrust-stage-*")), "partial staging leaked"
    assert not list(scratch.iterdir()), "compiler scratch leaked"
    assert before == {path: path.read_bytes() for path in artifacts}


def invoke(destination, args=records, helper=None, success=False):
    binary = adapter if helper is None else transaction_probe
    result = subprocess.run([str(value) for value in [binary, "--bundle", destination, *args]],
        capture_output=True, text=True, timeout=90, env=environment(helper or publisher))
    assert (result.returncode == 0) == success, result.stderr
    assert result.stdout == ("published 4 Java crates\n" if success else ""), result.stdout
    clean()
    return result


def inventory(directory):
    return {str(path.relative_to(directory)): path.read_bytes()
            for path in directory.rglob("*") if path.is_file()}


original = work / "original"
invoke(original, success=True)
expected = inventory(original)
assert len(expected) == 9
assert len(json.loads(expected["bundle.json"])["members"]) == 4
for path in [original, *original.rglob("*")]:
    assert path.stat().st_mode & 0o777 == (0o700 if path.is_dir() else 0o600)

for kind in ["empty", "directory", "file", "symlink", "dangling"]:
    output = work / ("existing-" + kind)
    if kind in {"empty", "directory"}:
        output.mkdir()
        if kind == "directory":
            (output / "sentinel").write_bytes(b"untouched")
    elif kind == "file":
        output.write_bytes(b"untouched")
    else:
        output.symlink_to(original if kind == "symlink" else work / "absent")
    previous = output.lstat()
    invoke(output)
    assert (output.lstat().st_ino, output.lstat().st_mode) == (previous.st_ino, previous.st_mode)
    if kind == "directory":
        assert inventory(output) == {"sentinel": b"untouched"}
    elif kind == "empty":
        assert not list(output.iterdir())
    elif kind == "file":
        assert output.read_bytes() == b"untouched"
    else:
        assert output.is_symlink()

# A compiler-rejected member must never call the publisher or create output.
bad_source = work / "bad.rs"
bad_source.write_text("pub fn score(value: i32) -> i32 { value + false }")
bad_records = [bad_source if value == artifacts[6] else value for value in records]
missing_helper = work / "helper-does-not-exist"
output = work / "failed-member"
result = invoke(output, bad_records, helper=missing_helper)
assert "error[E0277]" in result.stderr and "compiler rejected source crate" in result.stderr and not output.exists(), result.stderr
output = work / "missing-owner"
result = invoke(output, records[:-3], helper=missing_helper)
assert "node unreachable from its root" in result.stderr and not output.exists(), result.stderr

# Declared but unused dependency owners remain in the complete bundle exactly
# once, while the root manifest exposes no invented used-owner edge.
unused_source = work / "unused.rs"
unused_source.write_text("pub fn score(value: i32) -> i32 { value }")
unused_records = [unused_source if value == artifacts[6] else value for value in records]
unused_output = work / "unused-dependencies"
invoke(unused_output, unused_records, success=True)
unused_files = inventory(unused_output)
unused_index = json.loads(unused_files["bundle.json"])
assert len(unused_files) == 9 and len(unused_index["members"]) == 4
root_manifest = next(member["manifest"] for member in unused_index["members"]
                     if member["root"] == unused_index["root"])
assert json.loads(unused_files[root_manifest])["dependencies"] == []

# The test-only wrapper leaves the helper injectable; compiler and production
# publication functions are identical, with no mutation branches in their code.
for kind, mutation in [
    ("empty", 'mkdir "$2"'),
    ("directory", 'mkdir "$2"; printf untouched > "$2/sentinel"'),
    ("file", 'printf untouched > "$2"'),
    ("dangling", 'ln -s absent "$2"'),
    ("failure", 'exit 1'),
]:
    helper = work / ("late-" + kind + ".sh")
    helper.write_text('#!/bin/sh\nset -eu\ntest -f "$1/bundle.json"\n'
        + '[ "$(find "$1" -type f | wc -l)" -eq 9 ]\n'
        + mutation + '\nexec ' + shlex.quote(str(publisher)) + ' "$@"\n')
    helper.chmod(0o700)
    output = work / ("late-" + kind)
    invoke(output, helper=helper)
    if kind == "empty":
        assert output.is_dir() and not list(output.iterdir())
    elif kind == "directory":
        assert inventory(output) == {"sentinel": b"untouched"}
    elif kind == "file":
        assert output.read_bytes() == b"untouched"
    elif kind == "dangling":
        assert output.is_symlink() and output.readlink() == Path("absent")
    else:
        assert not output.exists()

for helper in [missing_helper, work / "not-executable"]:
    if helper.name == "not-executable":
        helper.write_text("not executable")
    output = work / (helper.name + "-output")
    invoke(output, helper=helper)
    assert not output.exists()

# Force both fully staged production bundles to race the real no-replace rename.
barrier = work / "barrier"
barrier.mkdir()
helper = work / "race.sh"
helper.write_text('#!/bin/sh\nset -eu\ntest -f "$1/bundle.json"\n'
    + 'barrier=' + shlex.quote(str(barrier)) + '\n'
    + 'touch "$barrier/$(basename "$1")"\n'
    + 'attempt=0\nwhile [ "$(ls -1A "$barrier" | wc -l)" -lt 2 ]; do\n'
    + 'attempt=$((attempt+1)); [ "$attempt" -lt 1000 ]; sleep 0.01\ndone\n'
    + 'exec ' + shlex.quote(str(publisher)) + ' "$@"\n')
helper.chmod(0o700)
output = work / "race"
processes = [subprocess.Popen([str(value) for value in [transaction_probe, "--bundle", output, *records]],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment(helper)) for _ in range(2)]
results = [process.communicate(timeout=90) for process in processes]
assert sorted(process.returncode for process in processes) == [0, 1], results
assert sorted(stdout for stdout, _ in results) == ["", "published 4 Java crates\n"]
assert len(list(barrier.iterdir())) == 2 and inventory(output) == expected
assert inventory(original) == expected
clean()
print("Java production bundles: compiler rejection, exact permissions, existing/late destinations, helper failures and concurrent atomic winner pass")
