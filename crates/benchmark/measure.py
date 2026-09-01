#!/usr/bin/env python3
"""Measure the release benchmark wall time and child peak RSS on Linux."""

from __future__ import annotations

import json
import platform
import resource
import subprocess
import sys
import time


def main() -> int:
    started = time.monotonic()
    completed = subprocess.run([sys.argv[1]], check=True, capture_output=True, text=True)
    wall_ms = round((time.monotonic() - started) * 1000)
    peak_mib = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024
    measurement = json.loads(completed.stdout.strip().splitlines()[-1])
    measurement.update(
        {
            "host": platform.platform(),
            "wall_ms": wall_ms,
            "peak_mib": round(peak_mib, 2),
        }
    )
    if measurement["declarations"] != 1_000 or measurement["targets"] != 8:
        raise SystemExit("benchmark did not exercise 1,000 declarations and eight targets")
    if measurement["elapsed_ms"] >= 2_000:
        raise SystemExit(f"generation exceeded 2,000 ms: {measurement}")
    if peak_mib >= 512:
        raise SystemExit(f"generation exceeded 512 MiB: {measurement}")
    print(json.dumps(measurement, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
