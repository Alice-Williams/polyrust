"""Check the independent oracle before using it to certify target mappings."""
from fractions import Fraction
import subprocess
import sys
from arithmetic_cases import BOUNDARIES, PAIRS
from arithmetic_oracle import (
    SIGN, INFINITY, NAN, MAGNITUDE, Operation as Op,
    category, rational, encode, rounded_quotient, result, observed,
)

ONE = 0x3ff0000000000000
TWO = 0x4000000000000000
HALF = 0x3fe0000000000000
HALF_ULP = 0x3ca0000000000000
MAXIMUM = 0x7fefffffffffffff


def golden_cases():
    cases = [
        (ONE, HALF_ULP, Op.ADD, ONE),
        (ONE + 1, HALF_ULP, Op.ADD, ONE + 2),
        (ONE | SIGN, HALF_ULP | SIGN, Op.ADD, (ONE | SIGN)),
        ((ONE + 1) | SIGN, HALF_ULP | SIGN, Op.ADD, (ONE + 2) | SIGN),
        (ONE, ONE, Op.SUBTRACT, 0),
        (ONE | SIGN, ONE, Op.ADD, 0),
        (SIGN, SIGN, Op.ADD, SIGN),
        (SIGN, 0, Op.ADD, 0),
        (SIGN, 0, Op.SUBTRACT, SIGN),
        (0, SIGN, Op.SUBTRACT, 0),
        (0x000fffffffffffff, 1, Op.ADD, 0x0010000000000000),
        (0x0010000000000000, HALF, Op.MULTIPLY, 0x0008000000000000),
        (1, TWO, Op.DIVIDE, 0),
        (3, TWO, Op.DIVIDE, 2),
        (1 | SIGN, TWO, Op.DIVIDE, SIGN),
        (3 | SIGN, TWO, Op.DIVIDE, SIGN | 2),
        (MAXIMUM, TWO, Op.MULTIPLY, INFINITY),
        (MAXIMUM, 0x7c90000000000000, Op.ADD, INFINITY),
        (ONE, TWO, Op.DIVIDE, HALF),
        (ONE, 0, Op.DIVIDE, INFINITY),
        (ONE, SIGN, Op.DIVIDE, SIGN | INFINITY),
        (0, 0, Op.DIVIDE, NAN),
        (INFINITY, INFINITY, Op.DIVIDE, NAN),
        (INFINITY, INFINITY | SIGN, Op.ADD, NAN),
        (0, INFINITY, Op.MULTIPLY, NAN),
        (ONE | SIGN, INFINITY, Op.DIVIDE, SIGN),
        (INFINITY | SIGN, TWO, Op.MULTIPLY, INFINITY | SIGN),
        (NAN, ONE, Op.ADD, NAN),
    ]
    for left, right, operation, expected in cases:
        assert result(left, right, operation) == expected, (left, right, operation)
    # Primitive rounding controls: tie-even, tie-odd and either side of halfway.
    assert [rounded_quotient(n, 4) for n in [5, 6, 7, 9, 10, 11]] == [1, 2, 2, 2, 2, 3]
    assert encode(Fraction(1, 1 << 1075)) == 0
    assert encode(Fraction(3, 1 << 1075)) == 2
    assert encode(-Fraction(1, 1 << 1075)) == SIGN
    assert encode(rational(MAXIMUM) + Fraction(1 << 970)) == INFINITY
    assert encode(rational(MAXIMUM) + Fraction((1 << 970) - 1)) == MAXIMUM
    return len(cases)


def round_trips():
    values = set(BOUNDARIES)
    values.update(bits for pair in PAIRS for bits in pair)
    finite = [bits for bits in values if category(bits) in ("zero", "finite")]
    for bits in finite:
        assert encode(rational(bits), bits & SIGN) == bits
    return len(finite)


def halfway_intervals():
    # Expected neighbors are consecutive bit patterns, independent of encode().
    # Exercise both tie parities and exponent carries across every normal binade.
    lower_bits = [0, 1, 2, (1 << 51) - 1, (1 << 52) - 2, (1 << 52) - 1]
    lower_bits += [(exponent << 52) | fraction
                   for exponent in range(1, 0x7ff)
                   for fraction in [0, 1, (1 << 52) - 2, (1 << 52) - 1]
                   if ((exponent << 52) | fraction) < MAXIMUM]
    for lower in lower_bits:
        low, high = rational(lower), rational(lower + 1)
        midpoint, quarter = (low + high) / 2, (high - low) / 4
        for sign in [0, SIGN]:
            for value, expected in [(midpoint - quarter, lower),
                                    (midpoint, lower + (lower & 1)),
                                    (midpoint + quarter, lower + 1)]:
                assert encode(-value if sign else value) == expected | sign
    return len(lower_bits) * 6


def main():
    golden = golden_cases()
    finite = round_trips()
    rounding = halfway_intervals()
    expected = [observed(result(left, right, operation))
                for left, right in PAIRS for operation in Op]
    text = "".join(f"{left:016x} {right:016x}\n" for left, right in PAIRS)
    native = subprocess.run([sys.argv[1]], input=text, text=True, capture_output=True, timeout=60)
    assert native.returncode == 0 and native.stderr == "", native.stderr
    assert native.stdout.splitlines() == expected, "pinned Rust differs from exact rational arithmetic"
    faults = [
        [observed(result(left, right, Op.ADD)) for left, right in PAIRS for _ in Op],
        [observed(result(right, left, op)) for left, right in PAIRS for op in Op],
        [observed(0 if (bits := result(left, right, op)) & MAGNITUDE == 0 else bits)
         for left, right in PAIRS for op in Op],
    ]
    assert all(fault != expected for fault in faults), "oracle corpus missed an intended fault"
    print(f"{golden} golden cases, {finite} exact finite round-trips, "
          f"{rounding} rounding-neighbor checks, {len(expected)} independent Rust arithmetic results, "
          "3 non-vacuous fault controls")


if __name__ == "__main__":
    main()
