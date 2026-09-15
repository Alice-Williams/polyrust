//! Public constants participate in the same source and symbol inventory as calls.
use super::Constant;
use crate::{
    ast::*,
    dialect::{CDialect, c_defined_constants},
};
use portable_codegen::{
    RenderReadyPackage, RustCrateExports, RustDeclarationId, RustSourceNode, RustVisibility,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Context<'a> {
    pub exports: &'a RustCrateExports,
    pub public: &'a BTreeSet<RustDeclarationId>,
    pub header: &'a CFileRef,
    pub implementation: &'a CFileRef,
}

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
    context: Context<'_>,
    definitions: &mut BTreeSet<RustDeclarationId>,
    symbols: &mut BTreeSet<CIdentifier>,
) -> Result<BTreeMap<RustDeclarationId, Constant>, String> {
    let mut constants = BTreeMap::new();
    for definition in c_defined_constants(package) {
        let object = definition.object();
        let CGeneratedOrigin::RustSource(origin) = &object.key().origin else {
            return Err("C dependency constant lacks Rust-source provenance".into());
        };
        let id = origin.declaration;
        if origin.node != RustSourceNode::Declaration
            || id.crate_id != context.exports.root.crate_id
            || origin.crate_exports.as_ref() != context.exports
            || !context.public.contains(&id)
            || !origin.externally_reachable
            || origin.visibility != RustVisibility::Public
            || object.file() != context.header
            || definition.implementation() != context.implementation
            || definition.linkage() != CLinkage::External
            || object.ty().constness() != CConstness::Const
            || !definitions.insert(id)
        {
            return Err("C dependency constant/provenance/file/linkage inventory disagrees".into());
        }
        let read_type = definition.read_type();
        if read_type.constness() != CConstness::Unqualified
            || !matches!(
                (read_type.kind(), definition.value()),
                (
                    CObjectTypeKind::Scalar(CScalarType::Bool),
                    CLiteral::Bool(_)
                ) | (
                    CObjectTypeKind::Scalar(CScalarType::I32),
                    CLiteral::Signed(CSignedLiteral::I32(_))
                ) | (
                    CObjectTypeKind::Scalar(CScalarType::I64),
                    CLiteral::Signed(CSignedLiteral::I64(_))
                )
            )
        {
            return Err("C dependency constant lacks an exact bool/i32/i64 literal".into());
        }
        if !symbols.insert(definition.name().clone()) {
            return Err("C dependency public symbols are not distinct".into());
        }
        constants.insert(
            id,
            Constant {
                object: object.clone(),
                symbol: definition.name().clone(),
                value: definition.value().clone(),
                read_type,
            },
        );
    }
    Ok(constants)
}
