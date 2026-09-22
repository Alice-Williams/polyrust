"""Native original crates at both profiles; no generated implementation as truth."""
from pathlib import Path
import struct
import subprocess
import sys

from character_source_truth import cases, corpus, trace


def run(binary, data):
    return subprocess.run([str(binary)], input=data, capture_output=True, timeout=180)


def main():
    binaries = [Path(value).resolve() for value in sys.argv[1:]]
    assert len(binaries) == 3 and len(set(binaries)) == 3
    data, expected, count = corpus()
    for binary in binaries[:2]:
        result = run(binary, data)
        assert result.returncode == 0 and result.stdout == expected and not result.stderr
        for bad in [0xd800, 0xdfff, 0x110000, 0xffffffff]:
            for packet in [struct.pack("<IIi", bad, 0, 0), struct.pack("<IIi", 0, bad, 0)]:
                rejected = run(binary, packet)
                assert rejected.returncode != 0 and rejected.stderr
        for size in [1, 4, 8, 11, 13]:
            rejected = run(binary, bytes(size))
            assert rejected.returncode != 0 and b"partial character packet" in rejected.stderr
    small, wanted, _ = corpus(False)
    observed = run(binaries[2], small)
    assert observed.returncode == 0 and observed.stdout == wanted
    assert observed.stderr == b"".join(trace(*case) for case in cases(False))
    print(f"{count} original multi-crate rows, 19 literals, mixed char/i32 records; "
          "both Rust profiles match independent integer truth; 13 malformed packets rejected/profile; "
          "actual original operand/field-order traces agree")


if __name__ == "__main__":
    main()
