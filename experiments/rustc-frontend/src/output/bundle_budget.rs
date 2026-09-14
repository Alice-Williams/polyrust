//! Checked aggregate output estimates, independent of compiler/backend actions.
pub(super) struct BundleBudget(u64);
const MAX_BYTES: u64 = 256 * 1024 * 1024;

impl BundleBudget {
    pub(super) fn new(members: usize) -> Result<Self, String> {
        if members == 0 || members > 1024 {
            return Err("bundle requires 1..1024 members".into());
        }
        Ok(Self(128 + 256 * members as u64))
    }
    pub(super) fn include(&mut self, source: u64, manifest: u64) -> Result<(), String> {
        let bytes = self
            .0
            .checked_add(source)
            .and_then(|value| value.checked_add(manifest))
            .ok_or("bundle byte estimate overflow")?;
        if bytes > MAX_BYTES {
            return Err("bundle exceeds the 256 MiB byte policy".into());
        }
        self.0 = bytes;
        Ok(())
    }
    pub(super) fn bytes(&self) -> u64 {
        self.0
    }
}
