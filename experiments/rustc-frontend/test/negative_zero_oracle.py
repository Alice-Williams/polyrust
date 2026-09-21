"""Raw-bit corpus and expected observations, independent of floating operations."""
SIGN = 1 << 63
MASK = SIGN - 1
INFINITY = 0x7FF0000000000000
MANTISSA = (1 << 52) - 1
official = [SIGN, 0, 0x4014000000000000, 0xBFF0000000000000,
            0x7FF8000000000000, 0x4000000000000000]
values = list(official)
for exponent in range(2048):
    for sign in [0, SIGN]:
        for mantissa in [0, 1, 1 << 51, MANTISSA - 1, MANTISSA]:
            values.append(sign | exponent << 52 | mantissa)
state = 0x4D595DF4D0F33173
for _ in range(65536):
    state = (state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
    values.append(state)
VALUES = list(dict.fromkeys(values))
VARIANTS = ["plain", "traced", "zero_sign", "unguarded", "flipped", "eager", "duplicate"]


def inputs():
    return "".join(f"{bits:016x}\n" for bits in VALUES)


def expected(variant="plain"):
    def result(bits):
        if variant == "zero_sign":
            return bits & MASK == 0
        if variant == "unguarded":
            return bool(bits & SIGN) and bits & MASK < INFINITY
        if variant == "flipped":
            return bits == 0
        return bits == SIGN
    return "".join(str(int(result(bits))) + "\n" for bits in VALUES)


def traces(variant="plain"):
    result = []
    for bits in VALUES:
        zero = bits & MASK == 0
        count = int(zero)
        if variant == "plain" or variant == "zero_sign":
            count = 0
        elif variant == "unguarded":
            count = 1
        elif variant == "eager":
            count = 1
        elif variant == "duplicate":
            count *= 2
        result.append("R" * count + "|")
    return "".join(result)
