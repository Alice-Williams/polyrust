"""Exact truncating binary64 remainder, without host floating arithmetic."""
from arithmetic_oracle import SIGN, NAN, category, rational, encode


def quotient(value):
    """Truncate an exact rational to an unbounded integer."""
    magnitude = abs(value.numerator) // value.denominator
    return -magnitude if value < 0 else magnitude


def result(left, right):
    assert 0 <= left < 1 << 64 and 0 <= right < 1 << 64
    lc, rc = category(left), category(right)
    if "nan" in (lc, rc) or lc == "infinity" or rc == "zero":
        return NAN
    if rc == "infinity" or lc == "zero":
        return left
    a, b = rational(left), rational(right)
    exact = a - quotient(a / b) * b
    bits = encode(exact, left & SIGN)
    assert rational(bits) == exact, "finite binary64 remainder must be exact"
    return bits
