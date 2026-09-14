/// Root crate sees the middle crate, not its transitive extern namespace.
pub fn identity(value: i32) -> i32 {
    renamed_middle::identity(value)
}
