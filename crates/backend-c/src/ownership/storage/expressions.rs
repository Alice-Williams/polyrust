//! Closed evaluated expression paths; no source fragments or optimistic effects.
use super::{
    Engine,
    state::State,
    values::{Cell, Pointer},
};
use crate::ast::{
    CBinaryOperator as B, CConversion, CInitializer, CInitializerKind, CLiteral, CObjectTypeKind,
    CPointerTarget, CPointerTest, CValue, CValueKind as V,
};
use crate::ownership::{CSafetyError as E, constants, layout::Layouts};
use std::collections::BTreeMap;

impl<'facts, 'ast> Engine<'facts, 'ast> {
    pub(super) fn expression(&mut self, value: &'ast CValue, state: &State) -> Result<Cell, E> {
        let cell = match value.kind() {
            V::Read(place) => {
                let path = self.place(place, state)?;
                state.read(&path, self.registry())?
            }
            V::AddressOf(place) => {
                Cell::Pointer(Pointer::Target(Box::new(self.place(place, state)?)))
            }
            V::FunctionAddress(function) => Cell::Pointer(Pointer::Function(function.clone())),
            V::Literal(CLiteral::NullPointer(_)) => Cell::Pointer(Pointer::Null),
            V::Call(_) => return Err(E::UnprovedStorageCall),
            V::Convert {
                conversion,
                operand,
            } => {
                let cell = self.expression(operand, state)?;
                match conversion {
                    CConversion::Numeric(_) => Cell::Initialized,
                    CConversion::AllocationRestore(_) => return Err(E::UnprovedAllocation),
                    CConversion::AdapterRestore(_) => {
                        if let Pointer::Target(path) = cell.pointer()? {
                            let CObjectTypeKind::Pointer(CPointerTarget::Object(target)) =
                                value.ty().kind()
                            else {
                                return Err(E::StorageTypeMismatch);
                            };
                            if !self.registry().pointee_types_match(&path.ty(), target)? {
                                return Err(E::StorageTypeMismatch);
                            }
                        } else if cell.pointer()? != Pointer::Null {
                            return Err(E::UnprovedStorage);
                        }
                        cell
                    }
                    CConversion::AddConst(_)
                    | CConversion::ObjectToVoid(_)
                    | CConversion::AdapterErase(_) => cell,
                }
            }
            V::Unary { operand, .. } => {
                self.expression(operand, state)?;
                Cell::Initialized
            }
            V::Binary {
                operator,
                left,
                right,
            } => {
                self.expression(left, state)?;
                if matches!(operator, B::LogicalAnd | B::LogicalOr) {
                    if let Some(mut branch) =
                        self.branch(left, *operator == B::LogicalAnd, state)?
                    {
                        branch.expression(right, state)?;
                    }
                } else {
                    self.expression(right, state)?;
                }
                Cell::Initialized
            }
            V::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                self.expression(condition, state)?;
                let mut result: Option<Cell> = None;
                for (truth, child) in [(true, then_value.as_ref()), (false, else_value.as_ref())] {
                    if let Some(mut branch) = self.branch(condition, truth, state)? {
                        let cell = branch.expression(child, state)?;
                        result = Some(match result {
                            None => cell,
                            Some(old) => old.join(&cell, value.ty(), self.registry())?,
                        });
                    }
                }
                result.ok_or(E::UnprovedStorage)?
            }
            V::PointerTest(test) => {
                match test {
                    CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => {
                        self.expression(value, state)?.pointer()?;
                    }
                    CPointerTest::SameSlot { left, right } => {
                        self.expression(left, state)?.pointer()?;
                        self.expression(right, state)?.pointer()?;
                    }
                }
                Cell::Initialized
            }
            V::Literal(_)
            | V::KnownConstant(_)
            | V::Enumerator(_)
            | V::SizeOf(_)
            | V::AlignOf(_) => Cell::Initialized,
        };
        // Every evaluated result must retain proved initialization/provenance,
        // even when discarded or produced by a conditional/interval join.
        cell.complete(value.ty(), self.registry())?;
        Ok(cell)
    }

    pub(super) fn initializer(
        &mut self,
        value: &'ast CInitializer,
        state: &State,
    ) -> Result<Cell, E> {
        Ok(match value.kind() {
            CInitializerKind::Expression(value) => self.expression(value, state)?,
            CInitializerKind::Zero(_) => Cell::Zero,
            CInitializerKind::Array { elements, .. } => Cell::Array {
                default: Box::new(Cell::Uninitialized),
                elements: elements
                    .iter()
                    .enumerate()
                    .map(|(i, value)| self.initializer(value, state).map(|cell| (i as u64, cell)))
                    .collect::<Result<_, E>>()?,
            },
            CInitializerKind::Struct { members, .. } => {
                let mut fields = BTreeMap::new();
                for (member, value) in members {
                    fields.insert(member.clone(), self.initializer(value, state)?);
                }
                Cell::Record(fields)
            }
            CInitializerKind::Union { member, value, .. } => Cell::Union {
                member: member.clone(),
                value: Box::new(self.initializer(value, state)?),
            },
        })
    }

    pub(super) fn branch(
        &self,
        value: &'ast CValue,
        truth: bool,
        state: &State,
    ) -> Result<Option<Self>, E> {
        if let Some(known) = self.pointer_truth(value, state)? {
            return Ok((known == truth).then(|| self.clone()));
        }
        if let Some(cursor) = &self.cursor {
            Ok(cursor.branch(value, truth)?.map(|cursor| Self {
                facts: self.facts,
                cursor: Some(cursor),
            }))
        } else {
            let known = constants::evaluate(&mut Layouts::new(self.registry()), value)
                .ok()
                .map(|n| n.truth());
            Ok((known.is_none() || known == Some(truth)).then(|| self.clone()))
        }
    }

    fn pointer_truth(&self, value: &'ast CValue, state: &State) -> Result<Option<bool>, E> {
        match value.kind() {
            V::Convert {
                conversion: CConversion::Numeric(crate::ast::CScalarType::Bool),
                operand,
            } => self.pointer_truth(operand, state),
            V::Unary {
                operator: crate::ast::CUnaryOperator::LogicalNot,
                operand,
            } => Ok(self.pointer_truth(operand, state)?.map(|v| !v)),
            V::PointerTest(CPointerTest::IsNull(operand) | CPointerTest::IsNonNull(operand)) => {
                let pointer = self.clone().expression(operand, state)?.pointer()?;
                let nonnull = match pointer {
                    Pointer::Null => Some(false),
                    Pointer::Target(_) | Pointer::Function(_) => Some(true),
                    Pointer::Unknown | Pointer::Expired => None,
                };
                let is_nonnull = matches!(value.kind(), V::PointerTest(CPointerTest::IsNonNull(_)));
                // The outer test, not its operand, owns the polarity.
                Ok(nonnull.map(|v| if is_nonnull { v } else { !v }))
            }
            _ => Ok(None),
        }
    }
}
