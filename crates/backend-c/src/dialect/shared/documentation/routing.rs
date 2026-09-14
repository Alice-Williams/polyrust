//! Compiler export membership controls which side owns module documentation.
use crate::ast::{CFileRef, CSourceFile};
use portable_codegen::{RustModuleDocumentation, RustSourceOrigin};

pub(super) enum FilePolicy<'a> {
    Single(&'a CFileRef),
    PublicPair {
        header: &'a CFileRef,
        implementation: &'a CFileRef,
    },
}

impl<'a> FilePolicy<'a> {
    pub(super) fn new(sources: &'a [CSourceFile]) -> Result<Self, String> {
        match super::super::profile::ordered_sources(sources)?.as_slice() {
            [single] => Ok(Self::Single(single.identity())),
            [header, implementation] => Ok(Self::PublicPair {
                header: header.identity(),
                implementation: implementation.identity(),
            }),
            _ => Err("documentation requires a closed C file layout".into()),
        }
    }

    pub(super) fn module(
        &self,
        origin: &RustSourceOrigin,
        module: &RustModuleDocumentation,
    ) -> &CFileRef {
        match self {
            Self::Single(file) => file,
            Self::PublicPair {
                header,
                implementation,
            } => {
                if origin
                    .crate_exports
                    .modules
                    .contains_key(&module.declaration)
                {
                    header
                } else {
                    implementation
                }
            }
        }
    }
}
