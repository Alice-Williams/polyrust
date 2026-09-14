"""Measure atomic no-replace directory publication independently of lowering."""
import os
from pathlib import Path
import subprocess
import sys

rename = Path(sys.argv[1]).resolve()
work = Path(os.environ["TEST_TMPDIR"]) / "rename-noreplace"
work.mkdir()


def stage(name):
    path = work / name
    path.mkdir()
    (path / "complete").write_text(name)
    return path


def invoke(source, destination):
    return subprocess.run([rename, source, destination], capture_output=True, timeout=10)


source = stage("success-source")
destination = work / "success"
assert invoke(source, destination).returncode == 0
assert not source.exists()
assert (destination / "complete").read_text() == "success-source"

# Model a destination arriving after the caller staged and checked absence.
for kind in ["empty-directory", "nonempty-directory", "file", "symlink", "dangling"]:
    source = stage("source-" + kind)
    destination = work / ("destination-" + kind)
    assert not destination.exists()
    if kind in ["empty-directory", "nonempty-directory"]:
        destination.mkdir()
        if kind == "nonempty-directory":
            (destination / "keep").write_text("preserve")
    elif kind == "file":
        destination.write_text("preserve")
    elif kind == "symlink":
        destination.symlink_to(work / "success", target_is_directory=True)
    else:
        destination.symlink_to(work / "missing", target_is_directory=True)
    identity = destination.lstat()
    result = invoke(source, destination)
    assert result.returncode != 0, kind
    after = destination.lstat()
    assert (after.st_dev, after.st_ino, after.st_mode) == (
        identity.st_dev, identity.st_ino, identity.st_mode), kind
    assert (source / "complete").read_text() == "source-" + kind
    if kind == "file":
        assert destination.read_text() == "preserve"
    elif kind == "nonempty-directory":
        assert (destination / "keep").read_text() == "preserve"
    elif kind == "empty-directory":
        assert not list(destination.iterdir())
    else:
        assert destination.is_symlink()

source = stage("failed-parent")
assert invoke(source, work / "absent-parent" / "destination").returncode != 0
assert (source / "complete").read_text() == "failed-parent"
assert invoke(work / "absent-source", work / "absent-destination").returncode != 0
assert not (work / "absent-destination").exists()

# Concurrent publishers must have exactly one winner, never a replaced winner.
left, right = stage("left"), stage("right")
destination = work / "race"
processes = [subprocess.Popen([rename, candidate, destination],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE) for candidate in [left, right]]
for process in processes:
    process.communicate(timeout=10)
assert sorted(process.returncode for process in processes) == [0, 1]
winner = (destination / "complete").read_text()
assert winner in ["left", "right"]
assert not (work / winner).exists()
loser = "right" if winner == "left" else "left"
assert (work / loser / "complete").read_text() == loser
print("Linux no-replace probe: complete publication, late existing paths, failure preservation and concurrent single winner passed")
