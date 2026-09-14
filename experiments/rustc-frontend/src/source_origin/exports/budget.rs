//! Shared admission before storing public binding names; not a rustc memory limit.
const BINDINGS: usize = 100_000;
const NAME_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub(super) struct Budget {
    bindings: usize,
    bytes: usize,
}

impl Budget {
    pub(super) fn binding(&mut self) -> Result<(), &'static str> {
        self.bindings = self
            .bindings
            .checked_add(1)
            .ok_or("source export inventory count overflow")?;
        if self.bindings > BINDINGS {
            Err("source export inventory binding budget exceeded")
        } else {
            Ok(())
        }
    }

    pub(super) fn name(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or("source export inventory byte accounting overflow")?;
        if self.bytes > NAME_BYTES {
            Err("source export inventory name budget exceeded")
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_binding_limit_and_one_over() {
        let mut budget = Budget::default();
        for _ in 0..BINDINGS {
            budget.binding().unwrap();
        }
        assert_eq!(
            budget.binding(),
            Err("source export inventory binding budget exceeded")
        );
    }

    #[test]
    fn cumulative_name_bytes_are_checked_before_copying() {
        let mut budget = Budget::default();
        budget.name(NAME_BYTES - 1).unwrap();
        budget.name(1).unwrap();
        budget.name(0).unwrap();
        assert_eq!(
            budget.name(1),
            Err("source export inventory name budget exceeded")
        );
    }

    #[test]
    fn both_counters_reject_overflow_without_wrapping() {
        let mut budget = Budget {
            bindings: usize::MAX,
            bytes: usize::MAX,
        };
        assert_eq!(
            budget.binding(),
            Err("source export inventory count overflow")
        );
        assert_eq!(
            budget.name(1),
            Err("source export inventory byte accounting overflow")
        );
    }
}
