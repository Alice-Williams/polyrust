//! Generated value references are selected by the checked Rust DefId.
use super::{Mapping, PublicConstantReadInput, PublicConstantReads};
use crate::java_lower::{Reader, Result, Value, constants};
use portable_backend_java::ast::*;
use portable_codegen::GeneratedSymbolId;
#[derive(Clone, Copy)]
pub(crate) struct JavaPublicConstantReads;
impl Mapping for JavaPublicConstantReads {
    type Capability = PublicConstantReads;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;
    fn lower<'tcx>(
        &self,
        reader: &mut Reader<'tcx>,
        input: PublicConstantReadInput<'tcx>,
    ) -> Result<Value> {
        let constant = reader
            .constants
            .get(&input.definition())
            .ok_or("public constant read lacks registered owned Java identity")?;
        if constant.definition != input.definition() || constant.value != input.value() {
            return Err("Java constant read disagrees with registered compiler value".into());
        }
        let (plan, _) = constants::literal(input.value());
        let value = Value::new(
            plan.clone(),
            JavaExpr {
                ty: plan.java_type(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::Generated(GeneratedSymbolId::Value(
                    constant.id,
                ))),
            },
        )?;
        #[cfg(public_constant_ast_probe)]
        super::public_constant_ast::read(reader, input, &value);
        Ok(value)
    }
}
