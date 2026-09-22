"""Unbounded modular products and deterministic multiplication-specific inputs."""
from math import isqrt
from wrapping_add_oracle import WIDTHS, cases as base_cases, limits


def signed(value, width):
    assert width in WIDTHS
    modulus = 1 << width
    residue = value % modulus
    return residue - modulus if residue >= modulus // 2 else residue


def result(left, right, width):
    low, high = limits(width)
    assert low <= left <= high and low <= right <= high
    return signed(left * right, width)


def cases(width):
    low, high = limits(width)
    pairs = list(base_cases(width))
    # Cross every bit with every other bit, including sign-bit factors.
    for a in range(width):
        for b in range(width):
            for sign_a in [-1, 1]:
                for sign_b in [-1, 1]:
                    pairs.append((signed(sign_a * (1 << a), width),
                                  signed(sign_b * (1 << b), width)))
    # Neighbours of products crossing the signed and unsigned width limits.
    for bound in [high, -low, (1 << width) - 1, 1 << width]:
        root = isqrt(bound)
        for left in range(root - 1, root + 2):
            for right in range(root - 1, root + 2):
                for sign_a in [-1, 1]:
                    for sign_b in [-1, 1]:
                        pairs.append((sign_a * left, sign_b * right))
    return list(dict.fromkeys(pairs))


CASES = [(width, left, right) for width in WIDTHS for left, right in cases(width)]


def inputs():
    return "".join(f"{width} {left} {right}\n" for width, left, right in CASES)


def expected():
    return "".join(f"{result(left, right, width)}\n" for width, left, right in CASES)


def narrowed(value, width):
    half_width = width // 2
    residue = value % (1 << half_width)
    return residue - (1 << half_width) if residue >= (1 << (half_width - 1)) else residue


def faulty(left, right, width, fault):
    low, high = limits(width)
    if fault == "saturating":
        return min(high, max(low, left * right))
    if fault == "add":
        return signed(left + right, width)
    if fault == "carryless":
        a, b = left % (1 << width), right % (1 << width)
        product = 0
        while b:
            if b & 1:
                product ^= a
            a <<= 1
            b >>= 1
        return signed(product, width)
    if fault == "narrow_operands":
        return signed(narrowed(left, width) * narrowed(right, width), width)
    assert fault == "narrow_result"
    return narrowed(left * right, width)
