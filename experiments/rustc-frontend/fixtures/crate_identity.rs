//! Explicit crate identity fixture.
pub fn identity(value: i32) -> i32 {
    helper(value)
}

fn helper(value: i32) -> i32 {
    value
}
