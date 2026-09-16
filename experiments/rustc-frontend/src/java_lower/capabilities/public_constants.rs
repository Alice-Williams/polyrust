//! Register an ordinary public static final field on package state.
use super::{ConstantDeclarationInput, Mapping, PublicConstants};
use crate::java_lower::{Result, constants, name, package::State, source};
use portable_backend_java::{ast::*, dialect::JavaDialect};
use portable_codegen::*;
#[derive(Clone, Copy)]
pub(crate) struct JavaPublicConstants;
impl Mapping for JavaPublicConstants {
    type Capability = PublicConstants;
    type Context<'tcx> = State;
    type Output = GeneratedValueId;
    fn lower<'tcx>(
        &self,
        state: &mut State,
        input: ConstantDeclarationInput<'tcx>,
    ) -> Result<GeneratedValueId> {
        if !state.public_api {
            return Err("public constant requires package mode".into());
        }
        if state.constants.contains_key(&input.definition()) {
            return Err("duplicate compiler public constant registration".into());
        }
        let tcx = input.tcx();
        let origin = crate::source_origin::read(
            tcx,
            &mut state.origins,
            input.definition(),
            RustSourceNode::Declaration,
            tcx.def_span(input.definition()),
        )?;
        let name = name(&format!(
            "constant{:016x}",
            origin.declaration.definition_path_hash
        ))?;
        let (plan, literal) = constants::literal(input.value());
        let ty = plan.java_type();
        let id = state.builder.value(GeneratedValue {
            name: name.as_str().into(),
            ty: JavaDialect.registered_type(&ty),
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::RustSource(origin),
            source: source(),
        });
        let field = JavaField {
            declared: Some(id),
            modifiers: vec![
                JavaModifier::Public,
                JavaModifier::Static,
                JavaModifier::Final,
            ],
            ty: ty.clone(),
            name,
            initializer: Some(JavaExpr::literal(ty, literal)),
        };
        state.constants.insert(
            input.definition(),
            constants::Constant {
                definition: input.definition(),
                id,
                field,
                value: input.value(),
            },
        );
        #[cfg(public_constant_ast_probe)]
        super::public_constant_ast::declaration(state, input, id);
        Ok(id)
    }
}
