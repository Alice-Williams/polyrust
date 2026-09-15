"""Observation only; no whole-call ownership certificate or target output."""
from pathlib import Path
import subprocess
import sys


def main():
    probe, fixture = [Path(p).resolve() for p in sys.argv[1:]]
    result = subprocess.run([str(probe), str(fixture)], capture_output=True,
                            text=True, check=False, timeout=60)
    assert result.returncode == 0, result.stderr
    assert "owned-call representation observation passed" in result.stdout
    print(result.stdout)


if __name__ == "__main__":
    main()
