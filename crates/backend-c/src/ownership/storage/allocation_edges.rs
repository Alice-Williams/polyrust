//! Refine actual allocation outcomes on typed predicate edges, never by names.
use super::{Engine, state::State, values::Pointer};
use crate::ast::{CConversion, CPointerTest, CScalarType, CUnaryOperator, CValue, CValueKind as V};
use crate::ownership::CSafetyError as E;

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn refine_allocations(
        &self,
        value: &'ast CValue,
        truth: bool,
        state: &mut State,
    ) -> Result<bool, E> {
        match value.kind() {
            V::Convert {
                conversion: CConversion::Numeric(CScalarType::Bool),
                operand,
            } => self.refine_allocations(operand, truth, state),
            V::Unary {
                operator: CUnaryOperator::LogicalNot,
                operand,
            } => self.refine_allocations(operand, !truth, state),
            V::PointerTest(CPointerTest::IsNull(operand) | CPointerTest::IsNonNull(operand)) => {
                if let Pointer::Allocation(origin) =
                    self.clone().expression(operand, state)?.pointer()?
                {
                    let nonnull =
                        matches!(value.kind(), V::PointerTest(CPointerTest::IsNonNull(_))) == truth;
                    state.allocations.refine(&origin, nonnull)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(true),
        }
    }
}
