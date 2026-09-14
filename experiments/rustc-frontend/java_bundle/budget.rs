pub(crate) const MAX_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct Budget(pub(crate) u64);

impl Budget {
    pub(crate) fn add(&mut self, bytes: u64) -> Result<(), String> {
        let next = self
            .0
            .checked_add(bytes)
            .ok_or("Java bundle byte overflow")?;
        if next > MAX_BYTES {
            return Err("Java bundle exceeds 256 MiB".into());
        }
        self.0 = next;
        Ok(())
    }
}
