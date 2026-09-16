//! Closed file layouts and declaration placement for the scalar C profile.
use crate::ast::{
    CAggregateRef, CDeclarationKind, CDefinitionKind, CFileItem, CFileRole, CLinkage, CSourceFile,
};

pub(super) enum Layout<'a> {
    Single(&'a CSourceFile),
    PublicPair {
        header: &'a CSourceFile,
        implementation: &'a CSourceFile,
    },
}

impl<'a> Layout<'a> {
    pub(super) fn new(sources: &'a [CSourceFile]) -> Result<Self, String> {
        let layout = match sources {
            [source] if matches!(source.identity().key().role,
                CFileRole::GeneratedSource | CFileRole::TestSource)
                && source.identity().key().path.as_str().ends_with(".c") => Self::Single(source),
            [left, right] => match (left.identity().key().role, right.identity().key().role) {
                (CFileRole::GeneratedPublicHeader, CFileRole::GeneratedSource) =>
                    Self::PublicPair { header: left, implementation: right },
                (CFileRole::GeneratedSource, CFileRole::GeneratedPublicHeader) =>
                    Self::PublicPair { header: right, implementation: left },
                _ => return Err("C paired profile requires one public header and one implementation".into()),
            },
            _ => return Err("C profile requires a .c implementation or test unit, or one public header/source pair".into()),
        };
        if let Self::PublicPair {
            header,
            implementation,
        } = layout
        {
            check_pair(header, implementation)?;
        }
        Ok(layout)
    }

    pub(super) fn has_public_declaration(&self) -> bool {
        match self {
            Self::Single(_) => true,
            Self::PublicPair { header, .. } => header
                .items()
                .iter()
                .any(|item| matches!(item, CFileItem::Declaration(_))),
        }
    }

    pub(super) fn ordered(&self) -> Vec<&'a CSourceFile> {
        match self {
            Self::Single(source) => vec![source],
            Self::PublicPair {
                header,
                implementation,
            } => vec![header, implementation],
        }
    }
}

fn check_pair(header: &CSourceFile, implementation: &CSourceFile) -> Result<(), String> {
    if !header.identity().key().path.as_str().ends_with(".h")
        || !implementation
            .identity()
            .key()
            .path
            .as_str()
            .ends_with(".c")
    {
        return Err("C paired profile requires .h/.c file names".into());
    }
    for item in header.items() {
        match item {
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::FunctionPrototype {
                    function,
                    linkage: CLinkage::External,
                } if function.file() == header.identity() => {}
                CDeclarationKind::ObjectDeclaration(object)
                    if object.file() == header.identity() =>
                {
                    super::constants::object(object)?;
                }
                _ => {
                    return Err(
                        "C public header admits only primary scalar functions and constants".into(),
                    );
                }
            },
            CFileItem::Comment(_) | CFileItem::StaticAssert(_) => {}
            CFileItem::Definition(_) => {
                return Err("C public header cannot contain a definition".into());
            }
        }
    }
    for item in implementation.items() {
        match item {
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::FunctionPrototype {
                    function,
                    linkage: CLinkage::Internal,
                } if function.file() == implementation.identity() => {}
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(record),
                    ..
                } if record.file() == implementation.identity() => {}
                _ => return Err("C implementation declarations must remain source-private".into()),
            },
            CFileItem::Definition(definition) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    linkage: CLinkage::External,
                    ..
                } if function.file() == header.identity() => {}
                CDefinitionKind::Function {
                    function,
                    linkage: CLinkage::Internal,
                    ..
                } if function.file() == implementation.identity() => {}
                CDefinitionKind::Object {
                    object,
                    linkage: CLinkage::External,
                    initializer,
                } if object.file() == header.identity() => {
                    super::constants::initializer(object, initializer)?;
                }
                _ => {
                    return Err(
                        "C definition linkage does not match header/private ownership".into(),
                    );
                }
            },
            CFileItem::Comment(_) => {}
            CFileItem::StaticAssert(_) => {
                return Err("C paired platform assertions belong in the header".into());
            }
        }
    }
    Ok(())
}
