"""Signed modular truth using unbounded integers, never target/native arithmetic."""
WIDTHS = (32, 64)


def limits(width):
    assert width in WIDTHS
    half = 1 << (width - 1)
    return -half, half - 1


def signed(value, width):
    low, _ = limits(width)
    return (value - low) % (1 << width) + low


def result(left, right, width):
    low, high = limits(width)
    assert low <= left <= high and low <= right <= high
    return signed(left + right, width)


def cases(width):
    low, high = limits(width)
    edges = [low, low + 1, low + 2, low + 3, -3, -2, -1, 0, 1, 2, 3, high - 3, high - 2, high - 1, high]
    pairs = [(left, right) for left in edges for right in edges]
    for bit in range(width):
        for magnitude in [(1 << bit) - 1, 1 << bit, (1 << bit) + 1]:
            for sign in [-1, 1]:
                left = signed(sign * magnitude, width)
                for right in [low, high, -1, 0, 1, left, signed(-left, width), signed(~left, width)]:
                    pairs.extend([(left, right), (right, left)])
    state = 0x4D595DF4D0F33173
    for _ in range(4096):
        state = (state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
        left = signed(state, width)
        state = (state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
        right = signed(state, width)
        pairs.append((left, right))
    return list(dict.fromkeys(pairs))


CASES = [(width, left, right) for width in WIDTHS for left, right in cases(width)]


def inputs():
    return "".join(f"{width} {left} {right}\n" for width, left, right in CASES)


def expected():
    return "".join(f"{result(left, right, width)}\n" for width, left, right in CASES)


def faulty(left, right, width, fault):
    low, high = limits(width)
    if fault == "saturating":
        return min(high, max(low, left + right))
    if fault == "carryless":
        return signed(left ^ right, width)
    if fault == "subtract":
        return signed(left - right, width)
    assert fault == "narrow"
    narrow = width // 2
    half = 1 << (narrow - 1)
    return (left + right + half) % (1 << narrow) - half
