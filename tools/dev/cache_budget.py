#!/usr/bin/env python3
"""Size-bounded eviction of disposable Bazel AC/CAS files; never source trees."""

import os
import re
import stat
import time
from pathlib import Path

CACHE_ROOT = Path("/var/tmp/polyrust-cache/bazel-disk")
CACHE_BYTES = 50_000_000_000
MAX_AGE_SECONDS = 14 * 24 * 60 * 60


def inventory(root):
    """Reject links and unexpected layouts before deleting even one file."""
    if not root.is_absolute() or root.resolve() != root or root == Path("/"):
        raise RuntimeError(f"unsafe cache root: {root}")
    entries = []
    for directory, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            path = Path(directory) / name
            metadata = path.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                raise RuntimeError(f"refusing symlink in cache: {path}")
            if stat.S_ISDIR(metadata.st_mode):
                continue
            if not stat.S_ISREG(metadata.st_mode):
                raise RuntimeError(f"refusing special cache entry: {path}")
            relative = path.relative_to(root)
            parts = relative.parts
            disposable = (
                len(parts) == 3
                and parts[0] in ("ac", "cas")
                and re.fullmatch(r"[0-9a-f]{2}", parts[1])
                and re.fullmatch(r"[0-9a-f]{64}", parts[2])
                and parts[2].startswith(parts[1])
            )
            if not disposable and parts[0] != "tmp":
                raise RuntimeError(f"unexpected cache entry: {path}")
            # Include filesystem allocation and temporary files in the budget.
            size = max(metadata.st_size, metadata.st_blocks * 512)
            entries.append((metadata.st_mtime_ns, path, size, bool(disposable)))
    return entries


def prune(root, budget=CACHE_BYTES, max_age=MAX_AGE_SECONDS, now=None):
    if budget < 0 or max_age < 0:
        raise ValueError("cache limits must be nonnegative")
    entries = inventory(root)
    total = sum(entry[2] for entry in entries)
    removed = 0
    cutoff = ((time.time() if now is None else now) - max_age) * 1_000_000_000
    for modified, path, size, disposable in sorted(entries):
        if disposable and (total > budget or modified < cutoff):
            path.unlink(missing_ok=True)
            total -= size
            removed += size
    if total > budget:
        raise RuntimeError("cache temporary files exceed budget; inspect them manually")
    return total, removed
