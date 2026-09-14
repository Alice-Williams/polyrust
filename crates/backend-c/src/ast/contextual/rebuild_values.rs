//! Rebuild actual expression children before consulting their derived types.

use super::super::{
    CCall, CCallable, CCallableKind, CConversion, CEffect, CLiteral, CNullPointer, CPointerTest,
    CValue, CValueKind,
};
use super::rebuild_expression_work::{self as work, Built, Children, Node};
use super::{CContextError as E, Recheck, exact};

impl Recheck<'_> {
    pub(super) fn value(&self, value: &CValue) -> Result<CValue, E> {
        match work::rebuild(self, Node::Value(value))? {
            Built::Value(value) => Ok(*value),
            Built::Place(_) => Err(E::StoredStructureMismatch),
        }
    }

    pub(super) fn value_node(&self, value: &CValue, children: &mut Children) -> Result<CValue, E> {
        let ast = &self.expressions;
        let rebuilt = match value.kind() {
            CValueKind::KnownConstant(value) => ast.known_constant(*value),
            CValueKind::Literal(value) => {
                let literal = match value {
                    CLiteral::NullPointer(pointer) => CLiteral::NullPointer(
                        CNullPointer::new(pointer.declared_type().clone())
                            .map_err(super::super::CExpressionError::from)?,
                    ),
                    other => other.clone(),
                };
                ast.literal(literal)?
            }
            CValueKind::Call(call) => {
                let (callable, arguments) = self.call_parts(call, children)?;
                ast.call_value(callable, arguments)?
            }
            CValueKind::Read(_) => ast.read(work::place(children)?)?,
            CValueKind::AddressOf(_) => ast.address_of(work::place(children)?)?,
            CValueKind::Enumerator(value) => ast.enumerator(value.clone())?,
            CValueKind::FunctionAddress(value) => ast.function_address(value.clone())?,
            CValueKind::Unary { operator, .. } => ast.unary(*operator, work::value(children)?)?,
            CValueKind::Binary { operator, .. } => {
                ast.binary(*operator, work::value(children)?, work::value(children)?)?
            }
            CValueKind::PointerTest(test) => ast.pointer_test(match test {
                CPointerTest::IsNull(_) => CPointerTest::IsNull(Box::new(work::value(children)?)),
                CPointerTest::IsNonNull(_) => {
                    CPointerTest::IsNonNull(Box::new(work::value(children)?))
                }
                CPointerTest::SameSlot { .. } => CPointerTest::SameSlot {
                    left: Box::new(work::value(children)?),
                    right: Box::new(work::value(children)?),
                },
            })?,
            CValueKind::Conditional { .. } => ast.conditional(
                work::value(children)?,
                work::value(children)?,
                work::value(children)?,
            )?,
            CValueKind::Convert { conversion, .. } => {
                self.conversion(conversion, work::value(children)?)?
            }
            CValueKind::SizeOf(ty) => ast.size_of(ty.clone())?,
            CValueKind::AlignOf(ty) => ast.align_of(ty.clone())?,
        };
        // Descendants have already been checked. Compare only this node's
        // cached fields here; the complete root equality checks payloads once.
        if value.ty() != rebuilt.ty() || value.brand != rebuilt.brand {
            return Err(E::StoredStructureMismatch);
        }
        Ok(rebuilt)
    }

    fn conversion(&self, conversion: &CConversion, operand: CValue) -> Result<CValue, E> {
        let ast = &self.expressions;
        Ok(match conversion {
            CConversion::Numeric(ty) => ast.numeric_conversion(*ty, operand)?,
            CConversion::AddConst(ty) => ast.add_const(ty.clone(), operand)?,
            CConversion::ObjectToVoid(ty) => ast.object_to_void(ty.clone(), operand)?,
            CConversion::AllocationRestore(value) => {
                ast.allocation_restore((**value).clone(), operand)?
            }
            CConversion::AdapterErase(value) => ast.adapter_erase((**value).clone(), operand)?,
            CConversion::AdapterRestore(value) => {
                ast.adapter_restore((**value).clone(), operand)?
            }
        })
    }

    fn call_parts(
        &self,
        call: &CCall,
        children: &mut Children,
    ) -> Result<(CCallable, Vec<CValue>), E> {
        let original = call.callable();
        let callable = match original.kind() {
            CCallableKind::Direct(function) => self.expressions.direct((**function).clone())?,
            CCallableKind::Indirect {
                contract_function, ..
            } => self
                .expressions
                .indirect(work::value(children)?, (**contract_function).clone())?,
            CCallableKind::Known(call) => self.expressions.known(*call),
        };
        if callable.brand != original.brand {
            return Err(E::StoredStructureMismatch);
        }
        let arguments = call
            .arguments()
            .iter()
            .map(|_| work::value(children))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((callable, arguments))
    }

    pub(super) fn effect(&self, effect: &CEffect) -> Result<CEffect, E> {
        let mut children = work::call_children(effect.call())
            .into_iter()
            .map(|node| work::rebuild(self, node))
            .collect::<Result<Children, E>>()?;
        let (callable, arguments) = self.call_parts(effect.call(), &mut children)?;
        if !children.is_empty() {
            return Err(E::StoredStructureMismatch);
        }
        exact(effect, self.expressions.call_effect(callable, arguments)?)
    }
}
