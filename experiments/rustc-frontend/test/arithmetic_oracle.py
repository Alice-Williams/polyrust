"""Exact binary64 arithmetic expectations; no host floating-point operations."""
from enum import Enum
from fractions import Fraction

SIGN = 1 << 63
MAGNITUDE = SIGN - 1
FRACTION = (1 << 52) - 1
INFINITY = 0x7ff0000000000000
NAN = INFINITY | (1 << 51)


class Operation(Enum):
    ADD = "add"
    SUBTRACT = "subtract"
    MULTIPLY = "multiply"
    DIVIDE = "divide"


def category(bits):
    magnitude = bits & MAGNITUDE
    if magnitude > INFINITY:
        return "nan"
    if magnitude == INFINITY:
        return "infinity"
    return "zero" if magnitude == 0 else "finite"


def rational(bits):
    """Decode one finite signed bit pattern exactly; reject nonfinite values."""
    assert 0 <= bits < 1 << 64 and category(bits) not in ("nan", "infinity")
    exponent = (bits >> 52) & 0x7ff
    significand = bits & FRACTION
    power = -1074
    if exponent:
        significand |= 1 << 52
        power = exponent - 1075
    value = Fraction(significand << power) if power >= 0 else Fraction(significand, 1 << -power)
    return -value if bits & SIGN else value


def rounded_quotient(numerator, denominator):
    quotient, remainder = divmod(numerator, denominator)
    comparison = 2 * remainder - denominator
    return quotient + int(comparison > 0 or (comparison == 0 and quotient & 1))


def encode(value, zero_sign=0):
    """Round an exact rational to binary64, nearest/even and gradual underflow."""
    assert isinstance(value, Fraction) and zero_sign in (0, SIGN)
    if value == 0:
        return zero_sign
    sign = SIGN if value < 0 else 0
    value = abs(value)
    numerator, denominator = value.numerator, value.denominator
    exponent = numerator.bit_length() - denominator.bit_length()
    below = numerator < (denominator << exponent) if exponent >= 0 else (numerator << -exponent) < denominator
    if below:
        exponent -= 1
    if exponent < -1022:
        # A carry to bit 52 naturally becomes the least normal number.
        return sign | rounded_quotient(numerator << 1074, denominator)
    if exponent > 1023:
        return sign | INFINITY
    shift = 52 - exponent
    significand = rounded_quotient(numerator << shift, denominator) if shift >= 0 else rounded_quotient(numerator, denominator << -shift)
    if significand == 1 << 53:
        significand >>= 1
        exponent += 1
    if exponent > 1023:
        return sign | INFINITY
    assert 1 << 52 <= significand < 1 << 53
    return sign | ((exponent + 1023) << 52) | (significand & FRACTION)


def result(left, right, operation):
    assert 0 <= left < 1 << 64 and 0 <= right < 1 << 64
    assert isinstance(operation, Operation)
    if operation is Operation.SUBTRACT:
        # Negating a NaN changes only information outside the category contract.
        return result(left, right ^ SIGN, Operation.ADD)
    lc, rc = category(left), category(right)
    if "nan" in (lc, rc):
        return NAN
    sign = (left ^ right) & SIGN
    if operation is Operation.ADD:
        if lc == rc == "infinity":
            return NAN if sign else left
        if lc == "infinity":
            return left
        if rc == "infinity":
            return right
        zero_sign = SIGN if lc == rc == "zero" and left & right & SIGN else 0
        return encode(rational(left) + rational(right), zero_sign)
    if operation is Operation.MULTIPLY:
        if ("infinity" in (lc, rc)) and ("zero" in (lc, rc)):
            return NAN
        if "infinity" in (lc, rc):
            return sign | INFINITY
        return encode(rational(left) * rational(right), sign)
    if operation is Operation.DIVIDE:
        if lc == rc and lc in ("zero", "infinity"):
            return NAN
        if lc == "infinity" or rc == "zero":
            return sign | INFINITY
        if rc == "infinity" or lc == "zero":
            return sign
        return encode(rational(left) / rational(right), sign)
    raise AssertionError("closed arithmetic operation")


def observed(bits):
    return "nan" if category(bits) == "nan" else f"{bits:016x}"
