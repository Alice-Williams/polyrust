//! Rust types map directly to the existing closed C type families.
use super::{Mapping, ObjectTypes, TypeInput};
use crate::c_lower::{Reader, Result, c, origin};
use portable_backend_c::ast::*;
use portable_codegen::RustSourceNode;
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct CObjectTypes;
impl Mapping for CObjectTypes {
    type Capability = ObjectTypes;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CObjectType;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: TypeInput<'tcx>,
    ) -> Result<CObjectType> {
        let value = input.0;
        match value.kind() {
            ty::Int(ty::IntTy::I32) => Ok(CObjectType::scalar(CScalarType::I32)),
            ty::Bool => Ok(CObjectType::scalar(CScalarType::Bool)),
            ty::Ref(_, pointee, rustc_hir::Mutability::Not) => {
                let pointee = c(reader.ty(*pointee)?.with_constness(CConstness::Const))?;
                Ok(CObjectType::pointer(CPointerTarget::Object(Box::new(
                    pointee,
                ))))
            }
            ty::Adt(definition, arguments)
                if definition.is_struct()
                    && arguments.is_empty()
                    && !definition.has_dtor(reader.tcx) =>
            {
                let repr = definition.repr();
                if repr.int.is_some()
                    || repr.align.is_some()
                    || repr.pack.is_some()
                    || repr.scalable.is_some()
                    || !repr.flags.is_empty()
                {
                    return Err("custom Rust record representations are not implemented".into());
                }
                if let Some(record) = reader.records.get(&definition.did()) {
                    return Ok(CObjectType::structure(record.clone()));
                }
                let name = format!("poly_record_{}", reader.records.len());
                let key = origin::key(
                    reader.tcx,
                    &mut reader.origins,
                    definition.did(),
                    RustSourceNode::Declaration,
                    reader.tcx.def_span(definition.did()),
                    &name,
                )?;
                let record = c(reader.registry.declare_struct(&reader.file, key))?;
                let owner = CAggregateRef::Struct(record.clone());
                let mut members = Vec::new();
                for (index, field) in definition.non_enum_variant().fields.iter().enumerate() {
                    let field_type = reader
                        .tcx
                        .try_normalize_erasing_regions(
                            ty::TypingEnv::fully_monomorphized(),
                            field.ty(reader.tcx, arguments),
                        )
                        .map_err(|_| "record field normalization failed")?;
                    if !matches!(field_type.kind(), ty::Int(ty::IntTy::I32) | ty::Bool) {
                        return Err("only scalar record fields are implemented".into());
                    }
                    let ty = reader.ty(field_type)?;
                    let key = origin::key(
                        reader.tcx,
                        &mut reader.origins,
                        field.did,
                        RustSourceNode::Declaration,
                        reader.tcx.def_span(field.did),
                        &format!("f{index}"),
                    )?;
                    members.push(c(reader.registry.register_member(&owner, key, ty))?);
                }
                c(reader.registry.define_aggregate(&owner, members))?;
                reader
                    .declarations
                    .push(CFileItem::Declaration(c(c(CDeclarations::new(
                        &reader.registry,
                        reader.file.clone(),
                    ))?
                    .aggregate(owner))?));
                reader.records.insert(definition.did(), record.clone());
                Ok(CObjectType::structure(record))
            }
            _ => Err(format!("C type mapping is not implemented: {value}")),
        }
    }
}
