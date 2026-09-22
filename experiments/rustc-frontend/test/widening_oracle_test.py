"""Boundary membership, independent fault models and pinned native agreement."""
from pathlib import Path
import subprocess
import sys
from widening_oracle import CASES, HIGH, LOW, expected, faulty, inputs, result


def check():
    inventory = set(CASES)
    # Corpus shrinkage requires a deliberate reviewed update.
    assert len(CASES) == len(inventory) == 73_890
    assert set(range(-(1 << 15), 1 << 15)) <= inventory
    assert {LOW, LOW + 1, HIGH - 1, HIGH, -1, 0, 1, 0x55555555, -0x55555556} <= inventory
    for bit in range(32):
        for sign in [-1, 1]:
            for delta in [-2, -1, 0, 1, 2]:
                value = sign * (1 << bit) + delta
                if LOW <= value <= HIGH:
                    assert value in inventory
    for value in CASES:
        assert result(value) == value and -(1 << 63) <= result(value) < (1 << 63)
    for fault in ["zero_extend", "narrow", "zero"]:
        assert any(faulty(value, fault) != result(value) for value in CASES), fault
    assert faulty(-1, "zero_extend") == 4294967295
    assert faulty(65535, "narrow") == -1
    assert faulty(1, "zero") == 0


def main():
    check()
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], input=inputs(), text=True,
                                capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == expected(), reference
    print(f"{len(CASES)} exact signed values match Rust as/From at O0/checks-on and "
          "O2/checks-off; exhaustive i16 plus full-i32 boundaries; three fault families detected")


if __name__ == "__main__":
    main()
