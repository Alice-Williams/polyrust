//! Fixed dependency resolution shared by metadata emission and source checking.
use rustc_interface::interface;
use rustc_session::config::Externs;

pub(crate) fn configure(config: &mut interface::Config) {
    config.opts.externs = Externs::new(
        config
            .opts
            .externs
            .iter()
            .map(|(name, entry)| {
                let mut entry = entry.clone();
                entry.force = true;
                (name.clone(), entry)
            })
            .collect(),
    );
}
