//! bool Not uses ordinary typed C unary and conversion nodes.
use super::{BooleanNegation, Mapping, NegationInput};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::{CScalarType, CUnaryOperator, CValue};

#[derive(Clone, Copy)]
pub(crate) struct CBooleanNegation;

impl Mapping for CBooleanNegation {
    type Capability = BooleanNegation;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: NegationInput<'tcx>) -> Result<CValue> {
        let operand = reader.expr(input.operand())?;
        let result = c(reader
            .expressions()
            .unary(CUnaryOperator::LogicalNot, operand))?;
        c(reader
            .expressions()
            .numeric_conversion(CScalarType::Bool, result))
    }
}
