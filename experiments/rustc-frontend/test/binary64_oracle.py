"""Independent integer-bit oracle: no host floating arithmetic or decimal parser."""
SIGN = 1 << 63
MAGNITUDE = SIGN - 1
INFINITY = 0x7FF0000000000000
LITERALS = {
    "positive_zero": 0, "negative_zero": SIGN,
    "minimum_subnormal": 1, "maximum_subnormal": 0x000FFFFFFFFFFFFF,
    "minimum_normal": 0x0010000000000000, "maximum_finite": 0x7FEFFFFFFFFFFFFF,
    "negative_maximum": 0xFFEFFFFFFFFFFFFF, "next_above_one": 0x3FF0000000000001,
    "rounded_integer": 0x4340000000000000, "decimal_tenth": 0x3FB999999999999A,
    "negative_underflow": SIGN,
}
TRANSPORT = ["identity", "local", "imported", "shared", "record", "shared_record"]
COMPARISONS = ["equal", "not_equal", "less", "less_equal", "greater", "greater_equal"]
OPERATORS = ["==", "!=", "<", "<=", ">", ">="]
VALUES = list(dict.fromkeys([*LITERALS.values(), 0x3FF0000000000000,
                            0x7FF0000000000000, 0x7FF8000000000001, 0x7FF0000000000001]))
state = 0x12345678
for _ in range(24):
    state = (state * 6364136223846793005 + 1442695040888963407) & MAGNITUDE
    if state < INFINITY:
        VALUES.append(state)
VALUES = list(dict.fromkeys(VALUES + [bits ^ SIGN for bits in VALUES]))


def observed(bits):
    return "nan" if bits & MAGNITUDE > INFINITY else f"{bits:016x}"


def comparison(a, b):
    if (a & MAGNITUDE) > INFINITY or (b & MAGNITUDE) > INFINITY:
        return [False, True, False, False, False, False]
    equal = a == b or (a & MAGNITUDE == 0 and b & MAGNITUDE == 0)
    # IEEE encoding is monotone in magnitude, reversed for negative values.
    def order(bits):
        return -(bits & MAGNITUDE) if bits & SIGN else bits
    less, greater = order(a) < order(b), order(a) > order(b)
    return [equal, not equal, less, less or equal, greater, greater or equal]


def expected():
    lines = [observed(bits) for bits in LITERALS.values()]
    lines.append(observed(LITERALS["decimal_tenth"]))
    lines.extend(observed(bits) for bits in VALUES for _ in TRANSPORT)
    lines.extend(str(int(value)) for a in VALUES for b in VALUES for value in comparison(a, b))
    return "\n".join(lines) + "\n"


def traces(pair="LR"):
    count = len(LITERALS) + 1 + len(VALUES) * len(TRANSPORT)
    return "\n" * count + (pair + "\n") * (len(VALUES) ** 2 * len(COMPARISONS))
