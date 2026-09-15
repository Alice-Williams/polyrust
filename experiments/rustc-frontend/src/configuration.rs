//! Closed compiler arguments: source callers cannot forward arbitrary rustc flags.
#![forbid(unsafe_code)]
#[path = "configuration/graph.rs"]
pub mod graph;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Entry,
    PublicPackage,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CrateName(String);

impl CrateName {
    fn new(value: &str) -> Result<Self, String> {
        let mut bytes = value.bytes();
        if value.len() > 64
            || value == "_"
            || !bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(
                "crate name requires a nonempty ASCII identifier of at most 64 bytes".into(),
            );
        }
        Ok(Self(value.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CrateKey(String);

impl CrateKey {
    fn new(value: &str) -> Result<Self, String> {
        if value.is_empty()
            || value.len() > 256
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./:@-".contains(&byte))
        {
            return Err(
                "crate key requires 1..256 ASCII build-identity bytes (letters, digits, _./:@-)"
                    .into(),
            );
        }
        Ok(Self(value.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Identity {
    Experiment,
    Explicit { name: CrateName, key: CrateKey },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    mode: Mode,
    identity: Identity,
    inputs: Vec<String>,
}

impl Configuration {
    /// Explicit package configuration without an arbitrary compiler-flag channel.
    pub fn package(name: &str, key: &str) -> Result<Self, String> {
        Ok(Self {
            mode: Mode::PublicPackage,
            identity: Identity::Explicit {
                name: CrateName::new(name)?,
                key: CrateKey::new(key)?,
            },
            inputs: Vec::new(),
        })
    }

    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let (mode, flags) = match arguments.split_first() {
            Some((flag, rest)) if flag == "--package" => (Mode::PublicPackage, rest),
            _ => (Mode::Entry, arguments),
        };
        let mut name = None;
        let mut key = None;
        let mut inputs = Vec::new();
        for pair in flags.chunks(2) {
            let [flag, value] = pair else {
                return Err("compiler configuration requires option/value pairs".into());
            };
            match flag.as_str() {
                "--input" => inputs.extend(pair.iter().cloned()),
                "--crate-name" if mode == Mode::PublicPackage => {
                    if name.replace(CrateName::new(value)?).is_some() {
                        return Err("duplicate --crate-name".into());
                    }
                }
                "--crate-key" if mode == Mode::PublicPackage => {
                    if key.replace(CrateKey::new(value)?).is_some() {
                        return Err("duplicate --crate-key".into());
                    }
                }
                _ => return Err(format!("unsupported compiler configuration option: {flag}")),
            }
        }
        let identity = match (name, key) {
            (None, None) => Identity::Experiment,
            (Some(name), Some(key)) => Identity::Explicit { name, key },
            _ => return Err("explicit identity requires both --crate-name and --crate-key".into()),
        };
        Ok(Self {
            mode,
            identity,
            inputs,
        })
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn explicit_identity(&self) -> Option<(&str, &str)> {
        match &self.identity {
            Identity::Experiment => None,
            Identity::Explicit { name, key } => Some((&name.0, &key.0)),
        }
    }

    pub fn declared_inputs(&self) -> &[String] {
        &self.inputs
    }

    pub fn compiler_arguments(&self, root: &str, sysroot: &str) -> Vec<String> {
        let name = match &self.identity {
            Identity::Experiment => "poly_input",
            Identity::Explicit { name, .. } => &name.0,
        };
        let mut arguments = vec![
            "rustc".into(),
            root.into(),
            "--sysroot".into(),
            sysroot.into(),
            "--crate-type=lib".into(),
            format!("--crate-name={name}"),
            "--edition=2024".into(),
            "-Funsafe-code".into(),
            "-Flong-running-const-eval".into(),
            "-Copt-level=0".into(),
            "-Cpanic=abort".into(),
        ];
        if let Identity::Explicit { key, .. } = &self.identity {
            arguments.push(format!("-Cmetadata={}", key.0));
        }
        arguments
    }
}
