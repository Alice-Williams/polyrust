"""Golden, invariant, native-Rust and wrong-family proof for exact remainder."""
import subprocess
import sys
from arithmetic_oracle import SIGN, INFINITY, NAN, category, rational, observed
from remainder_oracle import result
from remainder_cases import PAIRS
from remainder_faults import changed


def goldens():
    # Constants below are independently known binary64 values, not encode() output.
    positive = [
        (0x4016000000000000, 0x4000000000000000, 0x3ff8000000000000),  # 5.5 % 2 = 1.5
        (0x4018000000000000, 0x4008000000000000, 0),                 # 6 % 3 = 0
        (0x3ff0000000000001, 0x3ff0000000000000, 0x3cb0000000000000), # (1+2^-52) % 1
        (0x7fefffffffffffff, 0x4008000000000000, 0x4000000000000000), # maximum % 3 = 2
        (0x0010000000000000, 3, 1),  # 2^52 subnormal units % 3 = 1
        (3, 2, 1),
        (1, 2, 1),
        (0x7fefffffffffffff, 1, 0),
        (0, 0x3ff0000000000000, 0),
        (0x3ff0000000000000, INFINITY, 0x3ff0000000000000),
        (0, INFINITY, 0),
        (INFINITY, 0x3ff0000000000000, NAN),
        (0x3ff0000000000000, 0, NAN),
        (0, 0, NAN),
        (INFINITY, INFINITY, NAN),
        (NAN, 0x3ff0000000000000, NAN),
        (0x3ff0000000000000, NAN, NAN),
    ]
    for a, b, expected in positive:
        for sa in [0, SIGN]:
            for sb in [0, SIGN]:
                wanted = NAN if expected == NAN else expected | sa
                assert result(a | sa, b | sb) == wanted, (hex(a | sa), hex(b | sb), hex(wanted))
    return len(positive) * 4


def invariants():
    count = 0
    for left, right in PAIRS:
        bits = result(left, right)
        if category(left) in ("finite", "zero") and category(right) == "finite":
            a, b, remainder = rational(left), rational(right), rational(bits)
            assert bits & SIGN == left & SIGN
            assert abs(remainder) < abs(b)
            q = (a - remainder) / b
            assert q.denominator == 1
            assert abs(q) <= abs(a / b) < abs(q) + 1
            assert q == 0 or (q < 0) == ((a < 0) != (b < 0))
            count += 1
    return count


def main():
    golden_count = goldens()
    invariant_count = invariants()
    wanted = [observed(result(a, b)) for a, b in PAIRS]
    inputs = "".join(f"{a:016x} {b:016x}\n" for a, b in PAIRS)
    native = subprocess.run([sys.argv[1]], input=inputs, text=True, capture_output=True, timeout=60)
    assert native.returncode == 0 and native.stderr == "", native.stderr
    assert native.stdout.splitlines() == wanted, "native Rust % differs from exact truncating remainder"
    counts = {}
    for variant in ["euclidean", "nearest", "rounded_division", "zero_sign", "swapped"]:
        faulty = [observed(changed(a, b, variant)) for a, b in PAIRS]
        counts[variant] = sum(a != b for a, b in zip(faulty, wanted, strict=True))
        assert counts[variant] > 0, variant
    print(f"{golden_count} signed/category goldens, {invariant_count} exact remainder invariants, "
          f"{len(PAIRS)} native Rust results; five semantic faults detected: {counts}")


if __name__ == "__main__":
    main()
