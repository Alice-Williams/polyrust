"""Independent integer-only infinity truth; no target or host-float evaluation."""
from finite_constant_oracle import CASES as FINITE_CASES

SIGN = 1 << 63
FRACTION = (1 << 52) - 1
INFINITY = 0x7ff0000000000000
MAX_FINITE = INFINITY - 1
EXPRESSIONS = [
    ("named_positive", 0), ("named_negative", SIGN),
    ("negate_negative", 0), ("negate_positive", SIGN),
    ("positive_over_positive_zero", 0), ("positive_over_negative_zero", SIGN),
    ("negative_over_negative_zero", 0), ("negative_over_positive_zero", SIGN),
    ("positive_add_overflow", 0), ("negative_add_overflow", SIGN),
    ("positive_mul_overflow", 0), ("negative_mul_overflow", SIGN),
    ("positive_div_overflow", 0), ("negative_div_overflow", SIGN),
    ("bits_positive", 0), ("bits_negative", SIGN),
    ("negative_times_negative", 0), ("positive_times_negative", SIGN),
    ("alias_positive", 0), ("alias_negative", SIGN),
]


def classify(bits):
    assert 0 <= bits < 1 << 64
    if bits & INFINITY != INFINITY:
        return "finite"
    if bits & FRACTION:
        return "nan"
    return "negative" if bits & SIGN else "positive"


def cases():
    fractions = [0, *[1 << bit for bit in range(52)], FRACTION,
                 (1 << 51) | 1, FRACTION ^ (1 << 51)]
    state = 0x91a547d0e23cb681
    for _ in range(128):
        state = (6364136223846793005 * state + 1442695040888963407) % (1 << 64)
        fractions.append(state & FRACTION)
    return list(dict.fromkeys([*FINITE_CASES,
                              *[sign | INFINITY | fraction for sign in [0, SIGN]
                                for fraction in fractions]]))


CASES = cases()


def faulty(bits, fault):
    assert classify(bits) in ["positive", "negative"]
    if fault == "sign_loss":
        return INFINITY
    if fault == "finite_clamp":
        return (bits & SIGN) | MAX_FINITE
    assert fault == "zero"
    return 0


def inputs():
    return "".join(f"{bits:016x}\n" for bits in CASES)


def expected():
    constants = "".join(
        f"constant {name} {sign | INFINITY:016x} "
        + " ".join(f"{faulty(sign | INFINITY, fault):016x}"
                   for fault in ["sign_loss", "finite_clamp", "zero"]) + "\n"
        for name, sign in EXPRESSIONS)
    return constants + "".join(
        f"class {bits:016x} {classify(bits)} "
        f"{str(classify(bits) in ['positive', 'negative']).lower()} "
        f"{str(classify(bits) == 'nan').lower()}\n" for bits in CASES)
