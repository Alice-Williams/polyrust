//! Register one compiler-authenticated ordinary readonly C object.
use super::{ConstantDeclarationInput, Mapping, PublicConstants};
use crate::c_lower::{Result, c, constants, origin, package::State};
use portable_backend_c::ast::*;
use portable_codegen::RustSourceNode;

#[derive(Clone, Copy)]
pub(crate) struct CPublicConstants;
impl Mapping for CPublicConstants {
    type Capability = PublicConstants;
    type Context<'tcx> = State;
    type Output = CObjectRef;
    fn lower<'tcx>(
        &self,
        state: &mut State,
        input: ConstantDeclarationInput<'tcx>,
    ) -> Result<CObjectRef> {
        let header = state
            .header
            .as_ref()
            .ok_or("public constant requires a package header")?
            .clone();
        if state.constants.contains_key(&input.definition()) {
            return Err("duplicate compiler public constant registration".into());
        }
        let tcx = input.tcx();
        let identity = origin::identity(tcx, input.definition());
        let spelling = format!(
            "constant_{:016x}_{:016x}",
            identity.crate_id, identity.definition_path_hash
        );
        let key = origin::key(
            tcx,
            &mut state.origins,
            input.definition(),
            RustSourceNode::Declaration,
            tcx.def_span(input.definition()),
            &spelling,
        )?;
        let ty = constants::value(input.value()).ty();
        let ty = c(ty.with_constness(CConstness::Const))?;
        let object = c(state.registry.register_object(&header, key, ty))?;
        state
            .constants
            .insert(input.definition(), (object.clone(), input.value()));
        #[cfg(public_constant_ast_probe)]
        super::public_constant_ast::declaration(state, input, &object);
        Ok(object)
    }
}
