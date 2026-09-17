"""Wrong semantic families used only as independent oracle sensitivity controls."""
from arithmetic_oracle import SIGN, MAGNITUDE, NAN, category, rational, encode, rounded_quotient
from arithmetic_oracle import Operation, result as arithmetic
from remainder_oracle import result, quotient


def changed(left, right, variant):
    actual = result(left, right)
    if variant == "zero_sign":
        return 0 if actual & MAGNITUDE == 0 else actual
    if variant == "swapped":
        return result(right, left)
    if category(left) not in ("finite", "zero") or category(right) != "finite":
        return actual
    a, b = rational(left), rational(right)
    if variant == "euclidean":
        divisor = abs(b)
        return encode(a - (a // divisor) * divisor)
    ratio = a / b
    if variant == "nearest":
        q = rounded_quotient(abs(ratio.numerator), ratio.denominator)
        q = -q if ratio < 0 else q
    elif variant == "rounded_division":
        divided = arithmetic(left, right, Operation.DIVIDE)
        if category(divided) in ("nan", "infinity"):
            return NAN
        q = quotient(rational(divided))
    else:
        raise AssertionError("closed test-only fault")
    return encode(a - q * b, left & SIGN)
