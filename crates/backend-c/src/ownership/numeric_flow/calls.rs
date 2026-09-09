//! Closed numerical results; pointer/effect preconditions remain explicit obligations.
use super::provenance::Origin;
use super::state::{NaNPolarity, Predicate};
use super::{E, Engine, Number, NumericDomain, Obligation, State, storage};
use crate::ast::{CCall, CCallableKind, CReturnType, CScalarType};
use crate::dialect::CKnownCall;
use crate::ownership::constants::{CInteger, CNumber};

impl<'a> Engine<'a> {
    pub(super) fn call(
        &mut self,
        call: &'a CCall,
        state: &mut State<'a>,
    ) -> Result<Option<Number<'a>>, E> {
        if let CCallableKind::Indirect { pointer, .. } = call.callable().kind() {
            self.expression(pointer, state)?;
        }
        let mut arguments = Vec::new();
        for argument in call.arguments() {
            arguments.push(self.expression(argument, state)?);
        }
        let known = match call.callable().kind() {
            CCallableKind::Known(known) => Some(*known),
            CCallableKind::Direct(_) | CCallableKind::Indirect { .. } => None,
        };
        let byte_product = if known == Some(CKnownCall::WriteBytes) {
            let size = arguments[1].as_ref().ok_or(E::ExpectedNumericValue)?;
            let count = arguments[2].as_ref().ok_or(E::ExpectedNumericValue)?;
            Some(self.bounded_binary(
                crate::ast::CBinaryOperator::Multiply,
                [(&call.arguments()[1], size), (&call.arguments()[2], count)],
                state,
            )?)
        } else {
            None
        };
        self.record(|| Obligation::Call {
            call,
            arguments: arguments.clone(),
            byte_product,
        });
        // These exact closed operations cannot write generated object storage.
        // All other calls stay opaque until authenticated storage/call summaries.
        if !matches!(
            known,
            Some(
                CKnownCall::IsNan
                    | CKnownCall::SignBit
                    | CKnownCall::FloatRemainder
                    | CKnownCall::FloatTruncate
            )
        ) {
            state.opaque(&self.addresses, Origin::Call(call));
        }
        let signature = call.callable().signature();
        let CReturnType::Value(result) = signature.return_type() else {
            return Ok(None);
        };
        let Some(ty) = storage::scalar(self.registry, result.ty())? else {
            return Ok(None);
        };
        let mut number = Number::domain(NumericDomain::full(ty)?);
        if matches!(
            known,
            Some(CKnownCall::FloatTruncate | CKnownCall::FloatRemainder)
        ) {
            for argument in arguments.iter().flatten() {
                number.inherit_losses(argument);
            }
        } else if known.is_none() {
            number.losses.push(Origin::Call(call));
        }
        match known {
            Some(CKnownCall::IsNan) => {
                let argument = arguments[0].as_ref().ok_or(E::ExpectedNumericValue)?;
                if !argument.domain.may_nan() {
                    number.domain = NumericDomain::exact(CNumber::Integer(CInteger::checked(
                        CScalarType::Int,
                        0,
                    )?));
                } else if argument.domain.floating_bounds().is_none() {
                    number.domain = number.domain.exclude_integer(0)?;
                }
                number.predicate = Some(Predicate {
                    operand: &call.arguments()[0],
                    dependencies: storage::dependencies(&call.arguments()[0])?,
                    polarity: NaNPolarity::Nonzero,
                });
            }
            Some(CKnownCall::WriteBytes) => {
                if let Some(count) = &arguments[2]
                    && let Some((_, max)) = count.domain.integer_bounds()
                {
                    number.domain = number.domain.restrict_integer(0, max)?;
                }
            }
            Some(CKnownCall::FloatTruncate) => {
                if let Some(CNumber::Double(value)) = arguments[0]
                    .as_ref()
                    .and_then(|number| number.domain.exact_value())
                {
                    number.domain = NumericDomain::exact(CNumber::Double(value.trunc()));
                }
            }
            Some(
                CKnownCall::Allocate
                | CKnownCall::Release
                | CKnownCall::CopyBytes
                | CKnownCall::CompareBytes
                | CKnownCall::FloatRemainder
                | CKnownCall::SignBit
                | CKnownCall::StreamError,
            )
            | None => {}
        }
        Ok(Some(number))
    }
}
