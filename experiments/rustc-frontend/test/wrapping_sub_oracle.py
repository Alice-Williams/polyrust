"""Independent unbounded differences; share inputs, never addition truth."""
from wrapping_add_oracle import CASES, WIDTHS, cases, inputs, limits


def signed(value, width):
    assert width in WIDTHS
    modulus = 1 << width
    residue = value % modulus
    return residue - modulus if residue >= modulus // 2 else residue


def result(left, right, width):
    low, high = limits(width)
    assert low <= left <= high and low <= right <= high
    return signed(left - right, width)


def expected():
    return "".join(f"{result(left, right, width)}\n" for width, left, right in CASES)


def faulty(left, right, width, fault):
    low, high = limits(width)
    if fault == "saturating":
        return min(high, max(low, left - right))
    if fault == "add":
        return signed(left + right, width)
    if fault == "reverse":
        return signed(right - left, width)
    if fault == "borrowless":
        return signed(left ^ right, width)
    assert fault == "narrow"
    narrow = width // 2
    residue = (left - right) % (1 << narrow)
    return residue - (1 << narrow) if residue >= (1 << (narrow - 1)) else residue
