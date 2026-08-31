use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use portable_codegen::{OutputContents, OutputManifest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializeError {
    pub message: String,
}

impl std::fmt::Display for MaterializeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MaterializeError {}

impl From<io::Error> for MaterializeError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Journal {
    output_root: String,
    root_existed: bool,
    files: Vec<JournalFile>,
}

#[derive(Serialize, Deserialize)]
struct JournalFile {
    path: String,
    had_original: bool,
}

pub fn materialize(output: &Path, manifest: &OutputManifest) -> Result<(), MaterializeError> {
    materialize_impl(output, manifest, None)
}

fn materialize_impl(
    output: &Path,
    manifest: &OutputManifest,
    fail_after_commits: Option<usize>,
) -> Result<(), MaterializeError> {
    let (root, transaction) = resolve_roots(output)?;
    if transaction.exists() {
        reject_link(&transaction)?;
        if !transaction.is_dir() {
            return Err(error("recovery transaction path is not a directory"));
        }
        recover(&transaction, &root)?;
    }
    if root.exists() {
        reject_link(&root)?;
        if !root.is_dir() {
            return Err(error(format!(
                "output root {} is not a directory",
                root.display()
            )));
        }
    }

    fs::create_dir(&transaction)?;
    let result = stage_and_commit(&root, &transaction, manifest, fail_after_commits);
    if let Err(commit_error) = result {
        let recovery = recover(&transaction, &root);
        return match recovery {
            Ok(()) => Err(commit_error),
            Err(recovery_error) => Err(error(format!(
                "{}; automatic recovery also failed: {}",
                commit_error, recovery_error
            ))),
        };
    }
    fs::remove_dir_all(&transaction)?;
    Ok(())
}

fn stage_and_commit(
    root: &Path,
    transaction: &Path,
    manifest: &OutputManifest,
    fail_after_commits: Option<usize>,
) -> Result<(), MaterializeError> {
    let stage = transaction.join("stage");
    let backup = transaction.join("backup");
    fs::create_dir(&stage)?;
    fs::create_dir(&backup)?;

    for output_file in manifest.files() {
        let staged = checked_join(&stage, output_file.path())?;
        create_safe_parents(&stage, staged.parent().expect("file has a parent"))?;
        let bytes = match output_file.contents() {
            OutputContents::Text(text) => text.as_bytes(),
            OutputContents::Bytes(bytes) => bytes,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    let root_existed = root.exists();
    let mut journal = Journal {
        output_root: root.to_string_lossy().into_owned(),
        root_existed,
        files: Vec::with_capacity(manifest.files().len()),
    };
    for output_file in manifest.files() {
        let destination = checked_join(root, output_file.path())?;
        journal.files.push(JournalFile {
            path: output_file.path().to_owned(),
            had_original: destination.exists(),
        });
    }
    write_journal(transaction, &journal)?;

    if !root_existed {
        fs::create_dir(root)?;
    }
    reject_link(root)?;

    for (index, output_file) in manifest.files().iter().enumerate() {
        let destination = checked_join(root, output_file.path())?;
        create_safe_parents(root, destination.parent().expect("file has a parent"))?;
        if destination.exists() {
            reject_link(&destination)?;
            if !destination.is_file() {
                return Err(error(format!(
                    "output path {} is not a regular file",
                    destination.display()
                )));
            }
            let backup_file = checked_join(&backup, output_file.path())?;
            create_safe_parents(&backup, backup_file.parent().expect("file has a parent"))?;
            fs::rename(&destination, backup_file)?;
        }
        let staged = checked_join(&stage, output_file.path())?;
        fs::rename(staged, destination)?;
        if fail_after_commits == Some(index + 1) {
            return Err(error("simulated interruption"));
        }
    }
    Ok(())
}

fn resolve_roots(output: &Path) -> Result<(PathBuf, PathBuf), MaterializeError> {
    if output.as_os_str().is_empty() {
        return Err(error("output path is empty"));
    }
    let absolute = if output.is_absolute() {
        output.to_owned()
    } else {
        std::env::current_dir()?.join(output)
    };
    let name = absolute
        .file_name()
        .ok_or_else(|| error("output must name a directory below an existing parent"))?;
    let parent = absolute
        .parent()
        .ok_or_else(|| error("output has no parent"))?;
    let canonical_parent = fs::canonicalize(parent)?;
    reject_link_components(&canonical_parent)?;
    let root = canonical_parent.join(name);
    if root.exists() {
        reject_link(&root)?;
        let canonical_root = fs::canonicalize(&root)?;
        if canonical_root != root {
            return Err(error("output root resolves outside its explicit path"));
        }
    }
    let transaction =
        canonical_parent.join(format!(".{}.polyrust-transaction", name.to_string_lossy()));
    Ok((root, transaction))
}

fn checked_join(root: &Path, relative: &str) -> Result<PathBuf, MaterializeError> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error(format!("unsafe relative output path {relative:?}")));
    }
    let candidate = root.join(relative_path);
    if !candidate.starts_with(root) {
        return Err(error(format!("output path {relative:?} escaped its root")));
    }
    Ok(candidate)
}

fn create_safe_parents(root: &Path, parent: &Path) -> Result<(), MaterializeError> {
    if !parent.starts_with(root) {
        return Err(error("parent path escaped its root"));
    }
    let relative = parent
        .strip_prefix(root)
        .map_err(|_| error("parent path escaped its root"))?;
    let mut current = root.to_owned();
    for component in relative.components() {
        let Component::Normal(segment) = component else {
            return Err(error("non-normal output path component"));
        };
        current.push(segment);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if is_link(&metadata) || !metadata.is_dir() {
                    return Err(error(format!(
                        "output ancestor {} is not a safe directory",
                        current.display()
                    )));
                }
                let resolved = fs::canonicalize(&current)?;
                if !resolved.starts_with(root) {
                    return Err(error(format!(
                        "output ancestor {} resolves outside the root",
                        current.display()
                    )));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn reject_link_components(path: &Path) -> Result<(), MaterializeError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if current.as_os_str().is_empty() {
            continue;
        }
        if let Ok(metadata) = fs::symlink_metadata(&current)
            && is_link(&metadata)
        {
            return Err(error(format!(
                "path component {} is a symlink or reparse point",
                current.display()
            )));
        }
    }
    Ok(())
}

fn reject_link(path: &Path) -> Result<(), MaterializeError> {
    let metadata = fs::symlink_metadata(path)?;
    if is_link(&metadata) {
        Err(error(format!(
            "{} is a symlink or reparse point",
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn is_link(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn write_journal(transaction: &Path, journal: &Journal) -> Result<(), MaterializeError> {
    let path = transaction.join("journal.json");
    let bytes = serde_json::to_vec(journal).map_err(|error| MaterializeError {
        message: error.to_string(),
    })?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn recover(transaction: &Path, expected_root: &Path) -> Result<(), MaterializeError> {
    reject_link(transaction)?;
    let journal_path = transaction.join("journal.json");
    if !journal_path.exists() {
        fs::remove_dir_all(transaction)?;
        return Ok(());
    }
    let journal: Journal = serde_json::from_slice(&fs::read(&journal_path)?)
        .map_err(|json_error| error(format!("invalid recovery journal: {json_error}")))?;
    if Path::new(&journal.output_root) != expected_root {
        return Err(error("recovery journal output root does not match"));
    }
    let backup = transaction.join("backup");
    if backup.exists() {
        reject_link(&backup)?;
    }
    for file in journal.files.iter().rev() {
        let destination = checked_join(expected_root, &file.path)?;
        let backup_file = checked_join(&backup, &file.path)?;
        if backup_file.exists() {
            if destination.exists() {
                reject_link(&destination)?;
                if !destination.is_file() {
                    return Err(error("cannot recover over a non-file destination"));
                }
                fs::remove_file(&destination)?;
            }
            create_safe_parents(
                expected_root,
                destination.parent().expect("file has a parent"),
            )?;
            fs::rename(backup_file, destination)?;
        } else if !file.had_original && destination.exists() {
            reject_link(&destination)?;
            if !destination.is_file() {
                return Err(error("cannot remove a non-file generated destination"));
            }
            fs::remove_file(destination)?;
        }
    }
    remove_empty_generated_directories(expected_root, &journal.files)?;
    if !journal.root_existed && expected_root.exists() {
        let _ = fs::remove_dir(expected_root);
    }
    fs::remove_dir_all(transaction)?;
    Ok(())
}

fn remove_empty_generated_directories(
    root: &Path,
    files: &[JournalFile],
) -> Result<(), MaterializeError> {
    let mut directories = Vec::new();
    for file in files {
        let mut parent = checked_join(root, &file.path)?
            .parent()
            .expect("file has a parent")
            .to_owned();
        while parent != root {
            directories.push(parent.clone());
            parent = parent
                .parent()
                .ok_or_else(|| error("generated directory escaped output root"))?
                .to_owned();
        }
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    directories.dedup();
    for directory in directories {
        if directory.exists() {
            let _ = fs::remove_dir(directory);
        }
    }
    Ok(())
}

fn error(message: impl Into<String>) -> MaterializeError {
    MaterializeError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::atomic::{AtomicU64, Ordering},
    };

    use portable_codegen::OutputFile;

    use super::*;

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn sandbox(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "polyrust-m09-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn manifest(files: &[(&str, &str)]) -> OutputManifest {
        OutputManifest::new(
            files
                .iter()
                .map(|(path, text)| OutputFile::text(*path, *text))
                .collect(),
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
        fn visit(root: &Path, current: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
            if !current.exists() {
                return;
            }
            let mut entries: Vec<_> = fs::read_dir(current).unwrap().map(Result::unwrap).collect();
            entries.sort_by_key(fs::DirEntry::file_name);
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    visit(root, &path, result);
                } else {
                    result.insert(
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut result = BTreeMap::new();
        visit(root, root, &mut result);
        result
    }

    #[test]
    fn writes_manifest_and_preserves_unknown_files() {
        let sandbox = sandbox("success");
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("unknown.txt"), "keep").unwrap();
        materialize(
            &output,
            &manifest(&[("src/generated.rs", "new"), ("data.bin", "bytes")]),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(output.join("unknown.txt")).unwrap(),
            "keep"
        );
        assert_eq!(
            fs::read_to_string(output.join("src/generated.rs")).unwrap(),
            "new"
        );
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn simulated_interruption_rolls_back_byte_for_byte() {
        let sandbox = sandbox("rollback");
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("existing.txt"), "old").unwrap();
        fs::write(output.join("unknown.txt"), "keep").unwrap();
        let before = tree(&output);
        let result = materialize_impl(
            &output,
            &manifest(&[("existing.txt", "new"), ("nested/new.txt", "new")]),
            Some(1),
        );
        assert!(
            result
                .unwrap_err()
                .message
                .contains("simulated interruption")
        );
        assert_eq!(tree(&output), before);
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[test]
    fn stale_interrupted_transaction_is_recoverable() {
        let sandbox = sandbox("stale-recovery");
        let output = sandbox.join("out");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("existing.txt"), "old").unwrap();
        let before = tree(&output);
        let (root, transaction) = resolve_roots(&output).unwrap();
        fs::create_dir(&transaction).unwrap();
        assert!(
            stage_and_commit(
                &root,
                &transaction,
                &manifest(&[("existing.txt", "new"), ("new.txt", "new")]),
                Some(1),
            )
            .is_err()
        );
        assert!(transaction.exists());
        recover(&transaction, &root).unwrap();
        assert_eq!(tree(&output), before);
        assert!(!transaction.exists());
        fs::remove_dir_all(sandbox).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_ancestor_without_changing_either_tree() {
        use std::os::unix::fs::symlink;

        let sandbox = sandbox("symlink");
        let output = sandbox.join("out");
        let outside = sandbox.join("outside");
        fs::create_dir(&output).unwrap();
        fs::create_dir(&outside).unwrap();
        symlink(&outside, output.join("linked")).unwrap();
        let before_output = tree(&output);
        let before_outside = tree(&outside);
        assert!(materialize(&output, &manifest(&[("linked/escape.txt", "bad")])).is_err());
        assert_eq!(tree(&output), before_output);
        assert_eq!(tree(&outside), before_outside);
        fs::remove_dir_all(sandbox).unwrap();
    }
}
