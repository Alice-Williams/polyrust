use std::{
    fs,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

pub(super) fn publish(
    path: &Path,
    files: &[(String, String)],
    directories: &[String],
) -> Result<(), String> {
    let helper = std::env::var_os("POLYRUST_DIRECTORY_PUBLISHER")
        .map(PathBuf::from)
        .ok_or("no declared Linux directory publisher")?;
    if !helper.is_absolute() || !helper.is_file() {
        return Err("directory publisher must be an explicit absolute executable path".into());
    }
    let absolute = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join(path);
    let name = absolute
        .file_name()
        .ok_or("package output must name a new directory")?;
    let parent = absolute
        .parent()
        .ok_or("package output has no parent")?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let destination = parent.join(name);
    absent(&destination)?;
    let mut stage = Stage::create(&parent)?;
    for relative in directories {
        let path = stage.directory.join(relative);
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|error| error.to_string())?;
        stage.directories.push(path);
    }
    for (name, contents) in files {
        let path = stage.directory.join(name);
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .map_err(|error| error.to_string())?;
        stage.files.push(path);
        output
            .write_all(contents.as_bytes())
            .and_then(|()| output.sync_all())
            .map_err(|error| error.to_string())?;
    }
    absent(&destination)?;
    let status = std::process::Command::new(helper)
        .arg(&stage.directory)
        .arg(&destination)
        .status()
        .map_err(|error| format!("directory publisher: {error}"))?;
    if !status.success() {
        return Err("atomic no-replace directory publication failed".into());
    }
    // After rename we own nothing at the old staging path. Do not touch it.
    stage.published = true;
    Ok(())
}

fn absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
        Ok(_) => Err("package output already exists; use a new output directory".into()),
    }
}

struct Stage {
    directory: PathBuf,
    directories: Vec<PathBuf>,
    files: Vec<PathBuf>,
    published: bool,
}
impl Stage {
    fn create(parent: &Path) -> Result<Self, String> {
        for attempt in 0..128 {
            let directory =
                parent.join(format!(".polyrust-stage-{}-{attempt}", std::process::id()));
            match fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        directory,
                        directories: Vec::new(),
                        files: Vec::new(),
                        published: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("package staging directory budget exhausted".into())
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        if self.published {
            return;
        }
        // Only exact paths created by this transaction; never traverse a destination.
        for file in self.files.iter().rev() {
            let _ = fs::remove_file(file);
        }
        for directory in self.directories.iter().rev() {
            let _ = fs::remove_dir(directory);
        }
        let _ = fs::remove_dir(&self.directory);
    }
}
