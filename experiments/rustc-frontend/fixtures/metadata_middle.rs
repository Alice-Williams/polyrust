/// Middle crate preserves an explicitly renamed dependency edge.
pub fn identity(value: i32) -> i32 {
    renamed_leaf::identity(value)
}
