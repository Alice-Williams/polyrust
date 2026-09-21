"""Independent arithmetic invariants, fault sensitivity and native Rust agreement."""
from pathlib import Path
import subprocess
import sys
from wrapping_add_oracle import CASES, WIDTHS, cases, expected, faulty, inputs, limits, result, signed


def check():
    for width in WIDTHS:
        low, high = limits(width)
        known = [(high, 1, low), (low, -1, high), (low, low, 0), (high, high, -2),
                 (low, high, -1), (1, 1, 2), (-1, -1, -2), (0, 0, 0)]
        for left, right, wanted in known:
            assert result(left, right, width) == wanted
        pairs = cases(width)
        assert len(pairs) == len(set(pairs))
        assert all(any(result(a, b, width) != faulty(a, b, width, fault) for a, b in pairs)
                   for fault in ["saturating", "carryless", "subtract", "narrow"])
        assert any(a > 0 and b > 0 and a + b > high for a, b in pairs)
        assert any(a < 0 and b < 0 and a + b < low for a, b in pairs)
        for left, right in pairs:
            value = result(left, right, width)
            assert low <= value <= high
            assert (value - left - right) % (1 << width) == 0
            assert result(left, 0, width) == left
            assert result(left, signed(-left, width), width) == 0
            # Addition is commutative: reversal needs an independent trace proof.
            assert result(left, right, width) == result(right, left, width)
    assert len(CASES) == len(set(CASES))


def main():
    check()
    truth, data = expected(), inputs()
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], input=data, text=True,
                                capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == truth, reference
    print(f"{len(CASES)} signed 32/64-bit pairs match unbounded modular truth and "
          "native Rust with overflow checks on/off; four fault families detected")


if __name__ == "__main__":
    main()
