//! Actual Rust constant evaluation and runtime observations, not target code.
use std::io::{self, BufRead, Write};

const POSITIVE: f64 = f64::INFINITY;
const NEGATIVE: f64 = f64::NEG_INFINITY;
const VALUES: [(&str, f64); 20] = [
    ("named_positive", f64::INFINITY),
    ("named_negative", f64::NEG_INFINITY),
    ("negate_negative", -f64::NEG_INFINITY),
    ("negate_positive", -f64::INFINITY),
    ("positive_over_positive_zero", 1.0 / 0.0),
    ("positive_over_negative_zero", 1.0 / -0.0),
    ("negative_over_negative_zero", -1.0 / -0.0),
    ("negative_over_positive_zero", -1.0 / 0.0),
    ("positive_add_overflow", f64::MAX + f64::MAX),
    ("negative_add_overflow", -f64::MAX - f64::MAX),
    ("positive_mul_overflow", f64::MAX * 2.0),
    ("negative_mul_overflow", -f64::MAX * 2.0),
    ("positive_div_overflow", 1.0 / f64::from_bits(1)),
    ("negative_div_overflow", -1.0 / f64::from_bits(1)),
    ("bits_positive", f64::from_bits(0x7ff0_0000_0000_0000)),
    ("bits_negative", f64::from_bits(0xfff0_0000_0000_0000)),
    ("negative_times_negative", f64::NEG_INFINITY * -2.0),
    ("positive_times_negative", f64::INFINITY * -2.0),
    ("alias_positive", POSITIVE),
    ("alias_negative", NEGATIVE),
];

fn main() {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (name, value) in VALUES {
        let sign_loss = value.abs();
        let finite_clamp = if value.is_sign_negative() {
            -f64::MAX
        } else {
            f64::MAX
        };
        let zero = 0.0_f64;
        writeln!(
            output,
            "constant {name} {:016x} {:016x} {:016x} {:016x}",
            value.to_bits(),
            sign_loss.to_bits(),
            finite_clamp.to_bits(),
            zero.to_bits()
        )
        .unwrap();
    }
    for line in io::stdin().lock().lines() {
        let bits = u64::from_str_radix(&line.unwrap(), 16).unwrap();
        let value = f64::from_bits(bits);
        let nan = value.is_nan();
        let infinite = value.is_infinite();
        let category = if nan {
            "nan"
        } else if infinite {
            if value.is_sign_negative() {
                "negative"
            } else {
                "positive"
            }
        } else {
            "finite"
        };
        writeln!(output, "class {bits:016x} {category} {infinite} {nan}").unwrap();
    }
}
