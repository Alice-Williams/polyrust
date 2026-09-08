//! Rebuild actual expression children before consulting their derived types.

use super::super::{
    CCall, CCallable, CCallableKind, CConversion, CEffect, CLiteral, CNullPointer, CPointerTest,
    CValue, CValueKind,
};
use super::{CContextError as E, Recheck, exact};

impl Recheck<'_> {
    pub(super) fn value(&self, value: &CValue) -> Result<CValue, E> {
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
                let (callable, arguments) = self.call_parts(call)?;
                ast.call_value(callable, arguments)?
            }
            CValueKind::Read(place) => ast.read(self.place(place)?)?,
            CValueKind::AddressOf(place) => ast.address_of(self.place(place)?)?,
            CValueKind::Enumerator(value) => ast.enumerator(value.clone())?,
            CValueKind::FunctionAddress(value) => ast.function_address(value.clone())?,
            CValueKind::Unary { operator, operand } => {
                ast.unary(*operator, self.value(operand)?)?
            }
            CValueKind::Binary {
                operator,
                left,
                right,
            } => ast.binary(*operator, self.value(left)?, self.value(right)?)?,
            CValueKind::PointerTest(test) => ast.pointer_test(match test {
                CPointerTest::IsNull(value) => CPointerTest::IsNull(Box::new(self.value(value)?)),
                CPointerTest::IsNonNull(value) => {
                    CPointerTest::IsNonNull(Box::new(self.value(value)?))
                }
                CPointerTest::SameSlot { left, right } => CPointerTest::SameSlot {
                    left: Box::new(self.value(left)?),
                    right: Box::new(self.value(right)?),
                },
            })?,
            CValueKind::Conditional {
                condition,
                then_value,
                else_value,
            } => ast.conditional(
                self.value(condition)?,
                self.value(then_value)?,
                self.value(else_value)?,
            )?,
            CValueKind::Convert {
                conversion,
                operand,
            } => self.conversion(conversion, self.value(operand)?)?,
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

    fn call_parts(&self, call: &CCall) -> Result<(CCallable, Vec<CValue>), E> {
        let original = call.callable();
        let function = original.contract_function().clone();
        let callable = match original.kind() {
            CCallableKind::Direct => self.expressions.direct(function)?,
            CCallableKind::Indirect(pointer) => {
                self.expressions.indirect(self.value(pointer)?, function)?
            }
        };
        if callable.brand != original.brand {
            return Err(E::StoredStructureMismatch);
        }
        let arguments = call
            .arguments()
            .iter()
            .map(|value| self.value(value))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((callable, arguments))
    }

    pub(super) fn effect(&self, effect: &CEffect) -> Result<CEffect, E> {
        let (callable, arguments) = self.call_parts(effect.call())?;
        exact(effect, self.expressions.call_effect(callable, arguments)?)
    }
}
