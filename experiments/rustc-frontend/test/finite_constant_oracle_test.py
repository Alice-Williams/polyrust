"""Non-vacuous corpus coverage, exact fault columns and pinned native constants."""
from pathlib import Path
import subprocess
import sys
from arithmetic_oracle import SIGN, INFINITY, encode
from fractions import Fraction
from finite_constant_oracle import CASES, COMPUTED, FRACTIONS, NONFINITE, expected_values, finite
from finite_constant_faults import faulty, round_via_f32


def check():
    assert len(CASES) == len(set(CASES)) == 24566
    inventory = set(CASES)
    for exponent in range(2047):
        for sign in [0, SIGN]:
            for fraction in FRACTIONS:
                assert sign | (exponent << 52) | fraction in inventory
    assert {0, SIGN, 1, SIGN | 1, (1 << 52) - 1, 1 << 52,
            0x7fefffffffffffff, 0xffefffffffffffff} <= inventory
    assert COMPUTED == [SIGN, 0x3fb999999999999a, 0x4340000000000000,
                        0x3ff0000000000000, 0x3fd3333333333334, 0x3fd5555555555555,
                        0x0008000000000000, 2, 0x7fdfffffffffffff, SIGN]
    for bits in expected_values():
        assert finite(bits) == bits
    for bits in NONFINITE:
        try:
            finite(bits)
        except ValueError:
            pass
        else:
            raise AssertionError("nonfinite admitted by oracle")
    for fault in ["zero_sign", "f32", "wrong_value"]:
        assert any(faulty(v, fault) != v for v in expected_values()), fault
    assert round_via_f32(encode(Fraction(1) + Fraction(1, 1 << 24))) == 0x3ff0000000000000
    assert round_via_f32(1) == 0 and round_via_f32(SIGN | 1) == SIGN
    assert round_via_f32(0x7fefffffffffffff) == INFINITY
    assert faulty(SIGN, "zero_sign") == 0


def main():
    check()
    wanted = "".join(" ".join(f"{x:016x}" for x in [bits, *[faulty(bits, f)
                       for f in ["zero_sign", "f32", "wrong_value"]]]) + "\n"
                     for bits in expected_values())
    wanted += "+infinity\n-infinity\nnan\nnan\n"
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], text=True, capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == wanted, reference
    print(f"{len(CASES)} distinct finite patterns plus {len(COMPUTED)} independently specified constant expressions "
          "match native Rust at O0/O2; three native fault columns and four nonfinite controls checked")


if __name__ == "__main__":
    main()
