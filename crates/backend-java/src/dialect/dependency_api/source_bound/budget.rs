//! Each node pays punctuation, separators and all of its possible line indents.
const LIMIT: u64 = 256 * 1024 * 1024;

pub(super) struct Budget {
    bytes: u64,
}
impl Budget {
    pub(super) fn new() -> Self {
        Self { bytes: 0 }
    }
    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }
    pub(super) fn add(&mut self, count: u64) -> Result<(), String> {
        let bytes = self
            .bytes
            .checked_add(count)
            .filter(|bytes| *bytes <= LIMIT)
            .ok_or("Java source reservation exceeds its 256 MiB byte policy")?;
        self.bytes = bytes;
        Ok(())
    }
    pub(super) fn node(&mut self, depth: usize) -> Result<(), String> {
        if depth > 256 {
            return Err("Java source reservation depth limit exceeded".into());
        }
        // Admitted syntax emits at most 256 fixed bytes and four depth-based
        // four-space indents per node. All variable names/docs are additional.
        self.add(256 + 16 * depth as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_one_over_and_overflow_are_transactional() {
        let mut budget = Budget::new();
        budget.add(LIMIT - 1).unwrap();
        budget.add(1).unwrap();
        for amount in [1, u64::MAX] {
            assert!(budget.add(amount).is_err());
            assert_eq!(budget.bytes(), LIMIT);
        }
        let mut depth = Budget::new();
        depth.node(256).unwrap();
        let before = depth.bytes();
        assert!(depth.node(257).is_err());
        assert!(depth.node(usize::MAX).is_err());
        assert_eq!(before, depth.bytes());
    }
}
