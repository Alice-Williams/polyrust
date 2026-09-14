//! Eager scalar storage checks use an explicit evaluation stack.
//! Short-circuit/conditional expressions retain their branch-aware evaluator.
use super::{Engine, state::State, values::Cell};
use crate::ast::{CBinaryOperator as B, CConversion, CValue, CValueKind as V};
use crate::ownership::CSafetyError as E;

enum Work<'a> {
    Evaluate(&'a CValue),
    FinishScalar(&'a CValue),
}

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn expression(&mut self, root: &'ast CValue, state: &State) -> Result<Cell, E> {
        let mut pending = vec![Work::Evaluate(root)];
        let mut result = None;
        while let Some(work) = pending.pop() {
            match work {
                Work::Evaluate(value) => match value.kind() {
                    V::Call(call) if self.context.scalar_call(call.callable()) => {
                        pending.push(Work::FinishScalar(value));
                        pending.extend(call.arguments().iter().rev().map(Work::Evaluate));
                    }
                    V::Convert {
                        conversion: CConversion::Numeric(_),
                        operand,
                    }
                    | V::Unary { operand, .. } => {
                        pending.push(Work::FinishScalar(value));
                        pending.push(Work::Evaluate(operand));
                    }
                    V::Binary {
                        operator,
                        left,
                        right,
                    } if !matches!(operator, B::LogicalAnd | B::LogicalOr) => {
                        pending.push(Work::FinishScalar(value));
                        pending.push(Work::Evaluate(right));
                        pending.push(Work::Evaluate(left));
                    }
                    _ => result = Some(self.expression_leaf(value, state)?),
                },
                Work::FinishScalar(value) => {
                    // Both original scalar operation branches discard child
                    // cells after checking them. Numeric safety is a separate
                    // mandatory pass; no range or initialization proof is skipped.
                    let cell = Cell::Initialized;
                    cell.complete(value.ty(), self.registry())?;
                    result = Some(cell);
                }
            }
        }
        result.ok_or(E::UnprovedStorage)
    }
}
