use std::collections::BTreeSet;

const MAX_BYTES: usize = 256 * 1024 * 1024;
const MAX_FILES: usize = 3073;
const MAX_DIRECTORIES: usize = 4096;
const MAX_DEPTH: usize = 32;
const MAX_PATH_BYTES: usize = 4096;

/// Per-inventory ceilings, further bounded by the publisher's hard limits.
#[derive(Clone, Copy)]
pub struct TreeLimits {
    pub files: usize,
    pub directories: usize,
    /// Number of directory components, excluding the payload basename.
    pub depth: usize,
    pub path_bytes: usize,
    pub bytes: usize,
}

#[derive(Clone, Copy)]
pub enum PathPolicy {
    Flat,
    RelativeTree(TreeLimits),
}

pub(super) fn validate(
    files: &[(String, String)],
    policy: PathPolicy,
) -> Result<Vec<String>, String> {
    let limits = match policy {
        PathPolicy::Flat => TreeLimits {
            files: MAX_FILES,
            directories: 0,
            depth: 0,
            path_bytes: MAX_PATH_BYTES,
            bytes: MAX_BYTES,
        },
        PathPolicy::RelativeTree(limits) => limits,
    };
    if limits.files > MAX_FILES
        || limits.directories > MAX_DIRECTORIES
        || limits.depth > MAX_DEPTH
        || limits.path_bytes > MAX_PATH_BYTES
        || limits.bytes > MAX_BYTES
    {
        return Err("publication limits exceed the hard resource policy".into());
    }
    if files.is_empty() || files.len() > limits.files {
        return Err("publication file count exceeds the inventory policy".into());
    }
    let mut names = BTreeSet::new();
    let mut directories = BTreeSet::new();
    let mut bytes = 0usize;
    for (name, contents) in files {
        if name.len() > limits.path_bytes {
            return Err("publication path exceeds the byte policy".into());
        }
        // Inspect the raw spelling: Path::components normalizes dot/empty parts.
        if name.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
        }) {
            return Err("publication requires canonical relative ASCII paths".into());
        }
        if !names.insert(name.as_str()) {
            return Err("publication requires unique payload paths".into());
        }
        for (depth, (offset, _)) in name.match_indices('/').enumerate() {
            if depth >= limits.depth {
                return Err("publication directory depth exceeds the policy".into());
            }
            directories.insert(&name[..offset]);
            if directories.len() > limits.directories {
                return Err("publication directory count exceeds the policy".into());
            }
        }
        bytes = bytes
            .checked_add(contents.len())
            .ok_or("publication byte overflow")?;
        if bytes > limits.bytes {
            return Err("publication payload exceeds the byte policy".into());
        }
    }
    if names.iter().any(|name| directories.contains(name)) {
        return Err("publication file/directory prefix conflict".into());
    }
    // Lexical order places every parent before its children. Reverse cleanup
    // therefore removes children first, without filesystem traversal.
    Ok(directories.into_iter().map(str::to_owned).collect())
}
