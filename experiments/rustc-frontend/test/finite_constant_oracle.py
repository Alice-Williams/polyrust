"""Integer/rational truth for finite compiler constants; no host floats."""
from fractions import Fraction
from arithmetic_oracle import SIGN, FRACTION, INFINITY, Operation, category, encode, result

FRACTIONS = [0, 1, 2, 1 << 51, FRACTION]
RANDOM_COUNT = 4096
SEED = 0x51A9D32782E641B5


def cases():
    values = [sign | (exponent << 52) | fraction
              for exponent in range(2047) for sign in [0, SIGN] for fraction in FRACTIONS]
    state, count = SEED, 0
    while count < RANDOM_COUNT:
        state = (6364136223846793005 * state + 1442695040888963407) % (1 << 64)
        if category(state) in ["zero", "finite"]:
            values.append(state)
            count += 1
    return values


CASES = cases()
# Fixed native expressions are independent of this expectation implementation.
TENTH = encode(Fraction(1, 10))
COMPUTED = [
    encode(Fraction(0), SIGN),
    TENTH,
    encode(Fraction(9007199254740993)),
    encode(Fraction(1) + Fraction(1, 1 << 53)),
    result(TENTH, encode(Fraction(1, 5)), Operation.ADD),
    encode(Fraction(1, 3)),
    encode(Fraction(1, 1 << 1023)),
    encode(Fraction(1, 1 << 1073)),
    0x7fdfffffffffffff,
    SIGN,
]
NONFINITE = [INFINITY, SIGN | INFINITY, INFINITY | (1 << 51), SIGN | INFINITY | 1]


def finite(bits):
    assert 0 <= bits < 1 << 64
    if category(bits) not in ["zero", "finite"]:
        raise ValueError("nonfinite constant")
    return bits


def expected_values():
    return [*CASES, *COMPUTED]
