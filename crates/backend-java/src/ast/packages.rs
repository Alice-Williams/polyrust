//! Closed namespace vocabulary; crate IDs are naming data, not proof authority.
use super::JavaFilePlacement;
use std::borrow::Cow;

#[cfg(test)]
#[path = "../tests/crate_packages.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JavaPackage {
    Generated,
    RustCrate(u64),
}

impl JavaPackage {
    pub fn name(self) -> Cow<'static, str> {
        match self {
            Self::Generated => Cow::Borrowed("org.polyrust.generated"),
            Self::RustCrate(id) => Cow::Owned(format!("org.polyrust.generated.r{id:016x}")),
        }
    }

    pub fn source_directory(self, placement: JavaFilePlacement) -> String {
        let root = match placement {
            JavaFilePlacement::Main | JavaFilePlacement::Runtime => "src/main/java",
            JavaFilePlacement::NativeTest
            | JavaFilePlacement::Conformance
            | JavaFilePlacement::NegativeTest => "src/test/java",
        };
        format!("{root}/{}/", self.name().replace('.', "/"))
    }
}
