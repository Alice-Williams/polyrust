#![forbid(unsafe_code)]

/// Confirms that the pinned Bazel Rust toolchain can build library code.
pub fn target_message(target: &str) -> String {
    format!("portable code generation target: {target}")
}

#[cfg(test)]
mod tests {
    use super::target_message;

    #[test]
    fn formats_a_target_name() {
        assert_eq!(
            target_message("rust"),
            "portable code generation target: rust"
        );
    }
}
