"""Exact integer truth for fallible i64-to-i32 conversion, independent of casts."""

LOW, HIGH = -(1 << 31), (1 << 31) - 1
MIN, MAX = -(1 << 63), (1 << 63) - 1


def result(value):
    assert type(value) is int and MIN <= value <= MAX
    return value if LOW <= value <= HIGH else None


def cases():
    values = list(range(-(1 << 15), 1 << 15))
    for edge in [MIN, LOW, HIGH, MAX]:
        values.extend(v for v in range(edge - 32, edge + 33) if MIN <= v <= MAX)
    for bit in range(64):
        for sign in [-1, 1]:
            for delta in [-2, -1, 0, 1, 2]:
                value = sign * (1 << bit) + delta
                if MIN <= value <= MAX:
                    values.append(value)
    # Separate deterministic full-i32 and full-i64 samples retain successes
    # as well as failures; uniformly sampling only i64 would miss most successes.
    for width, seed in [(32, 0xC051A932), (64, 0xDA7A59C023115A9B)]:
        state = seed
        for _ in range(4096):
            state = (6364136223846793005 * state + 1442695040888963407) % (1 << width)
            values.append(state - (1 << width) if state >= (1 << (width - 1)) else state)
    return list(dict.fromkeys(values))


CASES = cases()
FAULTS = ("wrap", "saturate", "exclusive", "missing_lower", "missing_upper", "zero")


def faulty(value, fault):
    assert MIN <= value <= MAX
    wrapped = (value - LOW) % (1 << 32) + LOW
    if fault == "wrap":
        return wrapped
    if fault == "saturate":
        return min(HIGH, max(LOW, value))
    if fault == "exclusive":
        return value if LOW < value < HIGH else None
    if fault == "missing_lower":
        return wrapped if value <= HIGH else None
    if fault == "missing_upper":
        return wrapped if value >= LOW else None
    assert fault == "zero"
    return 0 if result(value) is not None else None


def token(value):
    return "err" if value is None else f"ok:{value}"


def parse_token(text):
    if text == "err":
        return None
    assert text.startswith("ok:")
    number = text[3:]
    value = int(number)
    assert str(value) == number and LOW <= value <= HIGH
    return value


def decode(text, count):
    lines = text.splitlines()
    assert len(lines) == count and text.endswith("\n")
    assert text == "\n".join(lines) + "\n"
    rows = []
    for line in lines:
        columns = line.split(" ")
        assert len(columns) == 2
        row = tuple(parse_token(column) for column in columns)
        assert line == " ".join(token(value) for value in row)
        rows.append(row)
    return rows


def inputs():
    return "".join(f"{value}\n" for value in CASES)
