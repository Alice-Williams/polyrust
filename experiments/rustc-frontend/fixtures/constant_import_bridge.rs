//! Both intermediate crates retain the same constants-only producer.
pub const MARKER: i32 = 3;
pub fn via_dependency() -> i64 {
    constants::WIDE
}
