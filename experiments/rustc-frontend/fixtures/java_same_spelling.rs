//! Same source names in distinct modules must keep distinct callable identities.
pub mod first {
    /// The first value operation.
    pub fn value(input: i32) -> i32 {
        input
    }
}

pub mod second {
    /// The second value operation delegates to the first.
    pub fn value(input: i32) -> i32 {
        super::first::value(input)
    }
}

pub use first::value as first_value;
pub use second::value as second_value;
