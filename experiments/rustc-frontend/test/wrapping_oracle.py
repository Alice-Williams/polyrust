"""Independent modular-integer truth; no generated text or metadata informs it."""
NAMES = ["method", "associated", "nested", "leaf", "ordinary", "absolute"]


def cases():
    rows = []
    for width in [32, 64]:
        low, high = -(1 << (width - 1)), (1 << (width - 1)) - 1
        values = {low, low + 1, high - 1, high, -1, 0, 1}
        for bit in range(width - 1):
            for sign in [-1, 1]:
                for delta in [-1, 0, 1]:
                    value = sign * (1 << bit) + delta
                    if low <= value <= high:
                        values.add(value)
        state, mask = 0x6A09E667F3BCC909, (1 << width) - 1
        for _ in range(4096):
            state = (state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
            bits = state & mask
            values.add(bits if bits <= high else bits - (1 << width))
        rows.extend((width, value) for value in sorted(values))
    return rows


def expected(rows):
    lines = []
    for width, value in rows:
        modulus = 1 << width
        negative = (-value) % modulus
        if negative >= modulus // 2:
            negative -= modulus
        results = [negative, negative, value, negative, value,
                   negative if value < 0 else value]
        lines.append(" ".join(map(str, results)) + "\n")
    return "".join(lines)


def traces(rows, variant):
    count = {"plain": 0, "traced": 6, "drop": 5, "duplicate": 7}[variant]
    return "".join(("A" if width == 32 else "B") * count + "\n" for width, _ in rows)
