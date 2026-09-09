//! Only evaluated children get numeric effects; all syntax was reconstructed first.
use super::provenance::Origin;
use super::{E, Engine, Number, NumericDomain, Obligation, State, storage};
use crate::ast::{
    CBinaryOperator as B, CConversion, CIndexBase, CPlace, CPlaceKind, CPointerTest, CScalarType,
    CUnaryOperator, CValue, CValueKind as V,
};
use crate::ownership::{
    constants::{self, CInteger, CNumber},
    numeric_flow::state::NaNPolarity,
};

impl<'a> Engine<'a> {
    pub(super) fn expression(
        &mut self,
        value: &'a CValue,
        state: &mut State<'a>,
    ) -> Result<Option<Number<'a>>, E> {
        let result = match value.kind() {
            V::Read(place) => {
                self.place(place, state)?;
                let Some(ty) = storage::scalar(self.registry, value.ty())? else {
                    return Ok(None);
                };
                let key = storage::Key::place(place, &mut self.layouts);
                if let Some(key) = key {
                    state.number(&key, ty)?
                } else {
                    let mut number = Number::domain(NumericDomain::full(ty)?);
                    number.losses.push(Origin::Read(place));
                    number
                }
            }
            V::Unary { operator, operand } => {
                let child = self.numeric(operand, state)?;
                let mut number =
                    self.calculation(value, child.domain.unary(*operator), &[&child])?;
                if *operator == CUnaryOperator::LogicalNot {
                    number.losses.clear();
                    number.predicate = child.predicate.map(|mut p| {
                        p.polarity = match p.polarity {
                            NaNPolarity::Nonzero => NaNPolarity::Zero,
                            NaNPolarity::Zero => NaNPolarity::Nonzero,
                        };
                        p
                    });
                }
                number
            }
            V::Binary {
                operator,
                left,
                right,
            } => {
                if matches!(operator, B::LogicalAnd | B::LogicalOr) {
                    return self.logical(value, *operator, left, right, state).map(Some);
                }
                let lhs = self.numeric(left, state)?;
                let rhs = self.numeric(right, state)?;
                let transfer = self.bounded_binary(*operator, [(left, &lhs), (right, &rhs)], state);
                let mut number = self.calculation(value, transfer, &[&lhs, &rhs])?;
                if matches!(
                    operator,
                    B::Equal | B::NotEqual | B::Less | B::LessEqual | B::Greater | B::GreaterEqual
                ) {
                    number.losses.clear();
                }
                number
            }
            V::Convert {
                conversion: CConversion::Numeric(ty),
                operand,
            } => {
                let child = self.numeric(operand, state)?;
                let mut number = self.calculation(value, child.domain.convert(*ty), &[&child])?;
                if *ty == CScalarType::Bool {
                    number.losses.clear();
                    number.predicate = child.predicate;
                } else if *ty == child.domain.ty() {
                    number.predicate = child.predicate;
                }
                number
            }
            V::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                self.numeric(condition, state)?;
                let mut results = Vec::new();
                for (truth, child) in [(true, then_value.as_ref()), (false, else_value.as_ref())] {
                    if let Some(mut branch) = self.refine(state, condition, truth)?
                        && let Some(number) = self.expression(child, &mut branch)?
                    {
                        results.push(number);
                    }
                }
                let Some(ty) = storage::scalar(self.registry, value.ty())? else {
                    return Ok(None);
                };
                let mut number = Number::domain(NumericDomain::empty(ty));
                for child in results {
                    let converted = self.calculation(value, child.domain.convert(ty), &[&child])?;
                    number.domain = number.domain.join(&converted.domain)?;
                    number.inherit_losses(&converted);
                }
                number
            }
            V::Call(call) => return self.call(call, state),
            V::AddressOf(place) => {
                self.place(place, state)?;
                return Ok(None);
            }
            V::Convert { operand, .. } => {
                self.expression(operand, state)?;
                return Ok(None);
            }
            V::FunctionAddress(_) => return Ok(None),
            V::PointerTest(test) => {
                match test {
                    CPointerTest::IsNull(value) | CPointerTest::IsNonNull(value) => {
                        self.expression(value, state)?;
                    }
                    CPointerTest::SameSlot { left, right } => {
                        self.expression(left, state)?;
                        self.expression(right, state)?;
                    }
                }
                boolean(None)?
            }
            V::Literal(_)
            | V::KnownConstant(_)
            | V::Enumerator(_)
            | V::SizeOf(_)
            | V::AlignOf(_) => {
                if storage::scalar(self.registry, value.ty())?.is_none() {
                    return Ok(None);
                }
                Number::domain(NumericDomain::exact(constants::evaluate(
                    &mut self.layouts,
                    value,
                )?))
            }
        };
        Ok(Some(result))
    }

    fn logical(
        &mut self,
        value: &'a CValue,
        operator: B,
        left: &'a CValue,
        right: &'a CValue,
        state: &mut State<'a>,
    ) -> Result<Number<'a>, E> {
        self.numeric(left, state)?;
        let selected = operator == B::LogicalAnd;
        let mut result = Number::domain(NumericDomain::empty(CScalarType::Int));
        if self.refine(state, left, !selected)?.is_some() {
            result.domain = boolean(Some(!selected))?.domain;
        }
        if let Some(mut branch) = self.refine(state, left, selected)? {
            let right = self.numeric(right, &mut branch)?;
            result.domain = result.domain.join(&boolean(right.domain.truth())?.domain)?;
        }
        self.record(|| Obligation::Calculation {
            value,
            number: result.clone(),
        });
        Ok(result)
    }

    pub(super) fn place(&mut self, place: &'a CPlace, state: &mut State<'a>) -> Result<(), E> {
        match place.kind() {
            CPlaceKind::Local(_) | CPlaceKind::Parameter(_) | CPlaceKind::Global(_) => {}
            CPlaceKind::Member { base, .. } => self.place(base, state)?,
            CPlaceKind::Dereference(pointer) => {
                self.expression(pointer, state)?;
            }
            CPlaceKind::Index { base, index } => {
                match base {
                    CIndexBase::Array(base) => self.place(base, state)?,
                    CIndexBase::Pointer(pointer) => {
                        self.expression(pointer, state)?;
                    }
                }
                let index = self.numeric(index, state)?;
                self.record(|| Obligation::Index { place, index });
            }
        }
        Ok(())
    }
}

pub(super) fn boolean(value: Option<bool>) -> Result<Number<'static>, E> {
    Ok(Number::domain(match value {
        Some(value) => NumericDomain::exact(CNumber::Integer(CInteger::checked(
            CScalarType::Int,
            i128::from(value),
        )?)),
        None => NumericDomain::full(CScalarType::Int)?.restrict_integer(0, 1)?,
    }))
}
