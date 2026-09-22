"""Pin const-context truth, actual faulty columns and strict native framing."""
from pathlib import Path
import subprocess
import sys

from character_constant_oracle import (BOUNDARIES, CONTEXTS, FAULTS, INTERIOR,
                                       constant, expected_output, expected_values, faulty, verify)
from character_oracle import scalar


def rejects(operation):
    try:
        operation()
    except ValueError:
        return
    raise AssertionError("invalid oracle input or observation accepted")


def check():
    values = expected_values()
    assert len(BOUNDARIES) == 19 and len(CONTEXTS) == 12 and len(values) == 4127
    assert len(INTERIOR) == len(set(INTERIOR)) == 4096
    assert INTERIOR[:3] == (17, 67602, 133139) and INTERIOR[-1] == 368656
    assert all(scalar(value) and constant(value) == value for value in values)
    assert {0, 0x378, 0xd7ff, 0xe000, 0xfdd0, 0xfffe, 0xffff, 0x10000,
            0x1f980, 0xf0000, 0x10fffe, 0x10ffff} <= set(values)
    for value in (True, False, None, "a", 1.0, -1, 0xd800, 0xdfff, 0x110000, 0xffffffff):
        rejects(lambda: constant(value))
    for fault in FAULTS:
        assert any(faulty(value, fault) != value for value in values), fault
    rejects(lambda: faulty(0xd800, "byte"))
    rejects(lambda: faulty(0, "unknown"))
    assert faulty(0x10000, "code_unit") == faulty(0x10000, "byte") == 0
    assert faulty(0x1f980, "replacement") == 0xfffd
    assert faulty(0, "changed") == 1
    original = expected_output()
    verify(original)
    for malformed in ("", original[:-1], original + "extra\n",
                      original.replace("4127", "4126", 1),
                      original.replace("00000000", "0000d800", 1),
                      original.replace("invalid 0", "invalid 1", 1)):
        rejects(lambda: verify(malformed))


def main():
    check()
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], text=True, capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        verify(native.stdout)
    print("4127 native compile-time character observations agree at O0/O2: "
          "19 boundaries, 4096 distinct interior samples, 12 named/computed contexts; "
          "four real faulty columns, four invalid const conversions and strict protocol controls")


if __name__ == "__main__":
    main()
