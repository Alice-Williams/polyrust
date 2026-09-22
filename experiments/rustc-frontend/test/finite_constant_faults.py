"""Exact integer model of premature f32 rounding and two bit faults."""
from fractions import Fraction
from arithmetic_oracle import SIGN, INFINITY, encode, rational, rounded_quotient


def round_via_f32(bits):
    sign = bits & SIGN
    value = abs(rational(bits))
    if value == 0:
        return sign
    n, d = value.numerator, value.denominator
    exponent = n.bit_length() - d.bit_length()
    below = n < (d << exponent) if exponent >= 0 else (n << -exponent) < d
    if below:
        exponent -= 1
    if exponent < -126:
        significand = rounded_quotient(n << 149, d)
        return encode(Fraction(significand, 1 << 149) * (-1 if sign else 1), sign)
    if exponent > 127:
        return sign | INFINITY
    shift = 23 - exponent
    significand = rounded_quotient(n << shift, d) if shift >= 0 else rounded_quotient(n, d << -shift)
    if significand == 1 << 24:
        significand >>= 1
        exponent += 1
    if exponent > 127:
        return sign | INFINITY
    power = exponent - 23
    exact = Fraction(significand << power) if power >= 0 else Fraction(significand, 1 << -power)
    return encode(-exact if sign else exact, sign)


def faulty(bits, fault):
    if fault == "zero_sign":
        return 0 if bits == SIGN else bits
    if fault == "f32":
        return round_via_f32(bits)
    assert fault == "wrong_value"
    return bits ^ 1
