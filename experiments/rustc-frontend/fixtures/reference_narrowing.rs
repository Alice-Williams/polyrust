//! Native standard checked conversions and deliberately faulty executable controls.
use std::io::{self, BufRead, Write};

#[derive(Clone, Copy)]
enum Mode {
    Correct,
    Wrap,
    Saturate,
    Exclusive,
    MissingLower,
    MissingUpper,
    Zero,
}

impl Mode {
    fn parse() -> Self {
        let args: Vec<_> = std::env::args().skip(1).collect();
        assert!(args.len() <= 1, "at most one fault selector");
        match args.first().map(String::as_str) {
            None => Self::Correct,
            Some("wrap") => Self::Wrap,
            Some("saturate") => Self::Saturate,
            Some("exclusive") => Self::Exclusive,
            Some("missing_lower") => Self::MissingLower,
            Some("missing_upper") => Self::MissingUpper,
            Some("zero") => Self::Zero,
            _ => panic!("unknown fault selector"),
        }
    }

    fn convert(self, value: i64) -> Option<i32> {
        let low = i64::from(i32::MIN);
        let high = i64::from(i32::MAX);
        match self {
            Self::Correct => i32::try_from(value).ok(),
            Self::Wrap => Some(value as i32),
            Self::Saturate => Some(value.clamp(low, high) as i32),
            Self::Exclusive => (low < value && value < high).then_some(value as i32),
            Self::MissingLower => (value <= high).then_some(value as i32),
            Self::MissingUpper => (value >= low).then_some(value as i32),
            Self::Zero => i32::try_from(value).ok().map(|_| 0),
        }
    }
}

fn token(value: Option<i32>) -> String {
    match value {
        Some(value) => format!("ok:{value}"),
        None => "err".into(),
    }
}

fn main() {
    let mode = Mode::parse();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let value: i64 = line.unwrap().parse().expect("expected signed i64 decimal");
        let via_into: Result<i32, _> = value.try_into();
        writeln!(
            output,
            "{} {}",
            token(mode.convert(value)),
            token(via_into.ok())
        )
        .unwrap();
    }
}
