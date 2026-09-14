/// Two direct aliases name one separately compiled dependency.
pub fn identity(value: i32) -> i32 {
    left::identity(right::identity(value))
}
