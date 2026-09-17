"""Writable private copies of read-only Bazel artifacts; originals stay untouched."""
from pathlib import Path
import shutil
import stat


def writable_copy(original: Path, destination: Path) -> None:
    """Copy into a new scratch directory, then permit owner-only mutations."""
    shutil.copytree(original, destination)
    for path in [destination, *destination.rglob("*")]:
        mode = path.stat().st_mode | stat.S_IWUSR
        if path.is_dir():
            mode |= stat.S_IXUSR
        path.chmod(mode)
