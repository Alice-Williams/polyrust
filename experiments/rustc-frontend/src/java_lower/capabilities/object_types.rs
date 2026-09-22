use super::{Mapping, ObjectTypes, TypeInput};
use crate::java_lower::{Reader, Result, TypePlan};
use rustc_middle::ty;

#[derive(Clone, Copy)]
pub(crate) struct JavaObjectTypes;

impl Mapping for JavaObjectTypes {
    type Capability = ObjectTypes;
    type Context<'tcx> = Reader<'tcx>;
    type Output = TypePlan;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: TypeInput<'tcx>) -> Result<TypePlan> {
        match input.0.kind() {
            ty::Int(ty::IntTy::I32) => Ok(TypePlan::I32),
            ty::Int(ty::IntTy::I64) => Ok(TypePlan::I64),
            ty::Bool => Ok(TypePlan::Bool),
            ty::Char => Ok(TypePlan::Char),
            ty::Float(ty::FloatTy::F64) => Ok(TypePlan::F64),
            ty::Ref(_, referent, rustc_hir::Mutability::Not) => {
                if matches!(referent.kind(), ty::Char) {
                    return Err("character references are not implemented".into());
                }
                Ok(TypePlan::Shared(Box::new(reader.ty(*referent)?)))
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
                reader.record(*definition, arguments)
            }
            _ => Err(format!(
                "Java source type mapping is not implemented: {}",
                input.0
            )),
        }
    }
}
