"""Independent exact integer truth for lossless signed i32-to-i64 widening."""

LOW, HIGH = -(1 << 31), (1 << 31) - 1


def result(value):
    assert LOW <= value <= HIGH
    return value


def cases():
    # Exhaust the entire signed i16 domain, then exercise all upper i32 bits.
    values = list(range(-(1 << 15), 1 << 15))
    values.extend([LOW, LOW + 1, HIGH - 1, HIGH, 0x55555555, -0x55555556])
    for bit in range(32):
        for sign in [-1, 1]:
            for delta in [-2, -1, 0, 1, 2]:
                value = sign * (1 << bit) + delta
                if LOW <= value <= HIGH:
                    values.append(value)
    state = 0x51A9D327
    for _ in range(8192):
        state = (1664525 * state + 1013904223) % (1 << 32)
        values.append(state - (1 << 32) if state >= (1 << 31) else state)
    return list(dict.fromkeys(values))


CASES = cases()


def faulty(value, fault):
    assert LOW <= value <= HIGH
    if fault == "zero_extend":
        return value % (1 << 32)
    if fault == "narrow":
        return (value + (1 << 15)) % (1 << 16) - (1 << 15)
    assert fault == "zero"
    return 0


def inputs():
    return "".join(f"{value}\n" for value in CASES)


def expected():
    return "".join(f"{result(value)} {result(value)}\n" for value in CASES)
