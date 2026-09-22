"""Exact invariants, full-width product coverage and pinned native agreement."""
from math import isqrt
from pathlib import Path
import subprocess
import sys
from wrapping_mul_oracle import CASES, WIDTHS, cases, expected, faulty, inputs, limits, result, signed


def check():
    for width in WIDTHS:
        low, high = limits(width)
        known = [(low, -1, low), (high, 2, -2), (low, 2, 0), (low, low, 0),
                 (high, high, 1), (low, high, low), (-1, -1, 1),
                 (3, 3, 9), (-3, 3, -9), (0, low, 0)]
        for left, right, wanted in known:
            assert result(left, right, width) == wanted
        pairs = cases(width)
        inventory = set(pairs)
        assert len(pairs) == len(inventory)
        assert any(a * b > high for a, b in pairs)
        assert any(a * b < low for a, b in pairs)
        assert any(abs(a) > (1 << (width - 2)) and abs(b) > (1 << (width - 2))
                   for a, b in pairs)
        for a in range(width):
            for b in range(width):
                left, right = signed(1 << a, width), signed(1 << b, width)
                assert (left, right) in inventory
                assert result(left, right, width) == signed(1 << (a + b), width)
        for bound in [high, -low, (1 << width) - 1, 1 << width]:
            root = isqrt(bound)
            assert (root, root) in inventory and (root + 1, root + 1) in inventory
        for fault in ["saturating", "add", "carryless", "narrow_operands", "narrow_result"]:
            assert any(result(a, b, width) != faulty(a, b, width, fault)
                       for a, b in pairs), (width, fault)
        for left, right in pairs:
            value = result(left, right, width)
            assert low <= value <= high
            assert (value - left * right) % (1 << width) == 0
            assert result(left, 0, width) == 0
            assert result(left, 1, width) == left
            assert result(left, -1, width) == signed(-left, width)
            assert value == result(right, left, width)
            assert result(left, signed(right + 1, width), width) == signed(value + left, width)
    # A reviewed inventory; corpus shrinkage requires an explicit update.
    assert len(CASES) == len(set(CASES)) == 34_546


def main():
    check()
    truth, data = expected(), inputs()
    assert len(sys.argv) == 3
    for reference in map(Path, sys.argv[1:]):
        native = subprocess.run([str(reference.resolve())], input=data, text=True,
                                capture_output=True, timeout=90)
        assert native.returncode == 0 and native.stderr == "", native.stderr[:2000]
        assert native.stdout == truth, reference
    print(f"{len(CASES)} signed 32/64-bit products match unbounded modular truth and "
          "native Rust with overflow checks on/off; five fault families detected")


if __name__ == "__main__":
    main()
