"""Shared deterministic cases, plus quotient/representability-specific coverage."""
from arithmetic_cases import PAIRS as ARITHMETIC_PAIRS
from arithmetic_oracle import SIGN

# Include ordinary nonintegral division and quotients exceeding binary64 range.
FOCUSED = [
    (0x4016000000000000, 0x4000000000000000),  # 5.5 / 2
    (0x7fefffffffffffff, 0x4008000000000000),  # maximum / 3
    (0x0010000000000000, 3),                 # minimum normal / 3 subnormal units
    (0x3ff0000000000001, 0x3ff0000000000000), # adjacent to 1
]
FOCUSED = [(a | sa, b | sb) for a, b in FOCUSED for sa in [0, SIGN] for sb in [0, SIGN]]


def systematic():
    # Every finite normal exponent, both signs and both magnitude orderings.
    for exponent in range(1, 0x7ff):
        larger = (exponent << 52) | ((1 << 52) - 1)
        smaller = ((exponent - 1) << 52) | 3
        for sign in [0, SIGN]:
            yield larger | sign, smaller
            yield smaller, larger | sign


PAIRS = ARITHMETIC_PAIRS + FOCUSED + list(systematic())
