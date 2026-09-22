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
        if !input.definition().is_local() {
            let imported = reader
                .foreign_constants
                .get(&input.definition())
                .ok_or("foreign public constant read lacks registered Java producer")?;
            let (plan, literal) = constants::literal(input.value());
            let proof = imported.constant();
            if proof.declaration() != crate::source_origin::identity(reader.tcx, input.definition())
                || proof.value().literal().as_ref() != Some(&literal)
                || proof.ty() != &plan.java_type()
            {
                return Err(
                    "foreign Java constant read disagrees with its certified producer".into(),
                );
            }
            let value = Value::new(
                plan.clone(),
                JavaExpr {
                    ty: plan.java_type(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Value(JavaValueRef::Dependency(imported.clone())),
                },
            )?;
            #[cfg(constant_import_probe)]
            super::constant_import_ast::read(reader, input, &value);
            return Ok(value);
        }
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
