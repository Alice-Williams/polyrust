//! Resolve an owned public constant through its registered object identity.
use super::{Mapping, PublicConstantReadInput, PublicConstantReads};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::CValue;
#[derive(Clone, Copy)]
pub(crate) struct CPublicConstantReads;
impl Mapping for CPublicConstantReads {
    type Capability = PublicConstantReads;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: PublicConstantReadInput<'tcx>,
    ) -> Result<CValue> {
        let (object, value) = reader
            .constants
            .get(&input.definition())
            .ok_or("public constant read lacks registered owned C identity")?;
        let portable_backend_c::ast::CGeneratedOrigin::RustSource(origin) = &object.key().origin
        else {
            return Err("C constant read lacks compiler provenance".into());
        };
        if origin.declaration != crate::source_origin::identity(reader.tcx, input.definition())
            || Some(object.file()) != reader.header.as_ref()
            || *value != input.value()
        {
            return Err("C constant read disagrees with registered compiler value".into());
        }
        let place = c(reader.expressions().global(object.clone()))?;
        let value = c(reader.expressions().read(place))?;
        #[cfg(public_constant_ast_probe)]
        super::public_constant_ast::read(reader, input, &value);
        Ok(value)
    }
}
