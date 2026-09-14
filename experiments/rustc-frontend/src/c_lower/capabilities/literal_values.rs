//! Admission and lowering of exact i32/bool source literals, including i32::MIN.
use super::{LiteralInput, LiteralValues, Mapping};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CLiteral, CSignedLiteral, CValue};
use rustc_ast::LitKind;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct CLiteralValues;

impl Mapping for CLiteralValues {
    type Capability = LiteralValues;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: LiteralInput<'tcx>) -> Result<CValue> {
        let expression = input.0;
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("literal compiler adjustment is not implemented".into());
        }
        let literal = match expression.kind {
            hir::ExprKind::Lit(literal) => match literal.node {
                LitKind::Int(magnitude, _)
                    if reader.checked.expr_ty(expression) == reader.tcx.types.i32 =>
                {
                    CLiteral::Signed(CSignedLiteral::I32(
                        i32::try_from(magnitude.0).map_err(|_| "i32 literal overflow")?,
                    ))
                }
                LitKind::Bool(value)
                    if reader.checked.expr_ty(expression) == reader.tcx.types.bool =>
                {
                    CLiteral::Bool(value)
                }
                _ => return Err("literal is not implemented".into()),
            },
            hir::ExprKind::Unary(hir::UnOp::Neg, operand)
                if reader.checked.expr_ty(expression) == reader.tcx.types.i32 =>
            {
                let hir::ExprKind::Lit(literal) = operand.kind else {
                    return Err("only negative integer literals are implemented".into());
                };
                let LitKind::Int(magnitude, _) = literal.node else {
                    return Err("only negative integer literals are implemented".into());
                };
                let magnitude =
                    i64::try_from(magnitude.0).map_err(|_| "integer literal overflow")?;
                CLiteral::Signed(CSignedLiteral::I32(
                    i32::try_from(-magnitude).map_err(|_| "i32 literal overflow")?,
                ))
            }
            _ => return Err("literal capability received an unsupported source shape".into()),
        };
        c(reader.expressions().literal(literal))
    }
}
