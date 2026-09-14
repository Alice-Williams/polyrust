//! Syntactic input descriptors; existence/canonical ownership is a driver check.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct InputPath(String);

impl InputPath {
    pub(super) fn new(value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 4096
            || value.chars().any(|c| c.is_control() || c == '=')
        {
            return Err("input path requires 1..4096 non-control bytes without '='".into());
        }
        Ok(Self(value.into()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LogicalPath(String);

impl LogicalPath {
    fn new(value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 4096
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
            || value.split('/').any(|part| matches!(part, "" | "." | ".."))
        {
            return Err(
                "logical source name requires a relative ASCII path without dot segments".into(),
            );
        }
        Ok(Self(value.into()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

/// One explicitly declared physical source/doc file and its stable logical name.
/// This descriptor is not proof that the file exists or that rustc checked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputMapping {
    pub(super) physical: InputPath,
    pub(super) logical: LogicalPath,
}

impl InputMapping {
    pub fn new(physical: &str, logical: &str) -> Result<Self, String> {
        Ok(Self {
            physical: InputPath::new(physical)?,
            logical: LogicalPath::new(logical)?,
        })
    }

    pub fn physical(&self) -> &str {
        self.physical.as_str()
    }

    pub fn logical(&self) -> &str {
        self.logical.as_str()
    }
}
