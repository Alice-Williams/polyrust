//! Type traversal delegates to the registered compiler-type mapping.
use super::{
    Reader, Result,
    capabilities::{Mapping, ObjectTypes, Supports, TypeInput},
};
use portable_backend_c::ast::CObjectType;
use rustc_middle::ty::Ty;
impl<'tcx> Reader<'tcx> {
    pub(super) fn ty(&mut self, value: Ty<'tcx>) -> Result<CObjectType> {
        let mapping = Supports::<ObjectTypes>::mapping(&self.mappings);
        mapping.lower(self, TypeInput(value))
    }
}
