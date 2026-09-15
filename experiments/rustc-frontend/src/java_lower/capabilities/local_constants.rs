//! A checked local constant declaration has no target runtime action.
use super::{LiteralValue, LocalConstantInput, LocalConstants, Mapping};
use crate::java_lower::{Reader, Result};

#[derive(Clone, Copy)]
pub(crate) struct JavaLocalConstants;

impl Mapping for JavaLocalConstants {
    type Capability = LocalConstants;
    type Context<'tcx> = Reader<'tcx>;
    type Output = ();

    fn lower<'tcx>(
        &self,
        _reader: &mut Reader<'tcx>,
        input: LocalConstantInput<'tcx>,
    ) -> Result<()> {
        #[cfg(local_constant_ast_probe)]
        let before = super::local_constant_ast::snapshot(_reader);
        let result = match input.value() {
            LiteralValue::Bool(_) | LiteralValue::I32(_) | LiteralValue::I64(_) => Ok(()),
        };
        #[cfg(local_constant_ast_probe)]
        super::local_constant_ast::check(_reader, input, &before);
        result
    }
}
