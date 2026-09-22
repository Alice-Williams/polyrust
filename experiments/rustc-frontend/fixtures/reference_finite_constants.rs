//! Native compile-time arrays/expressions, independent of the Python oracle.
const FRACTIONS: [u64; 5] = [0, 1, 2, 1 << 51, (1 << 52) - 1];
const COUNT: usize = 2047 * 2 * FRACTIONS.len() + 4096;

const fn constants() -> [f64; COUNT] {
    let mut values = [0.0; COUNT];
    let mut exponent = 0;
    let mut index = 0;
    while exponent < 2047 {
        let mut sign = 0;
        while sign < 2 {
            let mut fraction = 0;
            while fraction < FRACTIONS.len() {
                values[index] =
                    f64::from_bits((sign << 63) | (exponent << 52) | FRACTIONS[fraction]);
                index += 1;
                fraction += 1;
            }
            sign += 1;
        }
        exponent += 1;
    }
    let mut state = 0x51a9_d327_82e6_41b5_u64;
    while index < COUNT {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if ((state >> 52) & 0x7ff) != 0x7ff {
            values[index] = f64::from_bits(state);
            index += 1;
        }
    }
    values
}
static VALUES: [f64; COUNT] = constants();

const ROUNDED_INTEGER: f64 = 9007199254740993.0;
const COMPUTED: [f64; 10] = [
    -0.0,
    0.1,
    ROUNDED_INTEGER,
    1.0 + 1.1102230246251565e-16,
    0.1 + 0.2,
    1.0 / 3.0,
    f64::MIN_POSITIVE / 2.0,
    f64::from_bits(1) * 2.0,
    f64::MAX / 2.0,
    -0.0 * 2.0,
];
const NONFINITE: [f64; 4] = [
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
    f64::from_bits(0xfff0_0000_0000_0001),
];
fn main() {
    for value in VALUES.iter().chain(COMPUTED.iter()).copied() {
        let collapsed = if value == 0.0 { 0.0 } else { value };
        let narrowed = f64::from(value as f32);
        let wrong = f64::from_bits(value.to_bits() ^ 1);
        println!(
            "{:016x} {:016x} {:016x} {:016x}",
            value.to_bits(),
            collapsed.to_bits(),
            narrowed.to_bits(),
            wrong.to_bits()
        );
    }
    for value in NONFINITE {
        println!(
            "{}",
            if value.is_nan() {
                "nan"
            } else if value.is_sign_negative() {
                "-infinity"
            } else {
                "+infinity"
            }
        );
    }
}
