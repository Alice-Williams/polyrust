#!/usr/bin/env python3
"""Local Docker build guard. This is not a virtual-disk hard quota."""

import fcntl
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

from cache_budget import CACHE_ROOT, prune

DOCKER_BYTES = 200_000_000_000
HOST_FREE_BYTES = 150_000_000_000
LOCK = CACHE_ROOT.parent / "development-storage.lock"


def local_container():
    return (
        os.environ.get("GITHUB_ACTIONS") != "true"
        and Path("/.dockerenv").exists()
        and Path("/workspace").is_mount()
    )


def check_space(host, docker):
    free = host.f_bavail * host.f_frsize
    used = (docker.f_blocks - docker.f_bfree) * docker.f_frsize
    if free < HOST_FREE_BYTES:
        raise RuntimeError(f"host drive has {free / 1e9:.1f} GB free; need 150 GB")
    if used >= DOCKER_BYTES:
        raise RuntimeError(
            f"Docker filesystem uses {used / 1e9:.1f} GB; budget is 200 GB"
        )


def check_current_space():
    check_space(os.statvfs("/workspace"), os.statvfs(CACHE_ROOT.parent))


def maintain():
    total, removed = prune(CACHE_ROOT)
    if removed:
        print(
            f"Storage budget: evicted {removed / 1e9:.2f} GB of cached artifacts; "
            f"{total / 1e9:.2f} GB remain.",
            file=sys.stderr,
        )


def stop(process):
    if process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        process.wait()
        return
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def run_guarded(command):
    # Batch mode keeps all build processes in the monitored process group.
    # Do not silently change an explicitly requested server invocation.
    options = command[: command.index("--")] if "--" in command else command
    if any(option in ("--nobatch", "--batch=false", "--batch=0") for option in options):
        raise RuntimeError("local budgeted builds require batch mode")
    for argument in options:
        if argument.startswith("--disk_cache"):
            raise RuntimeError("local builds must use the shared .bazelrc disk cache")
    if "--batch" not in command:
        command = [command[0], "--batch", *command[1:]]
    CACHE_ROOT.mkdir(parents=True, exist_ok=True)
    with LOCK.open("a") as lock:
        # Serialize local build/eviction cycles across candidate checkouts.
        fcntl.flock(lock, fcntl.LOCK_EX)
        maintain()
        check_current_space()
        process = subprocess.Popen(command, start_new_session=True)
        try:
            while process.poll() is None:
                check_current_space()
                time.sleep(2)
            check_current_space()
            return process.returncode
        finally:
            stop(process)
            maintain()


def main():
    command = sys.argv[1:]
    if not command:
        raise RuntimeError("missing Bazel command")
    if not local_container():
        os.execv(command[0], command)

    def interrupted(_signal, _frame):
        raise KeyboardInterrupt("build interrupted")

    signal.signal(signal.SIGTERM, interrupted)
    return run_guarded(command)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, KeyboardInterrupt) as error:
        print(f"Storage guard stopped the build: {error}", file=sys.stderr)
        sys.exit(75)
