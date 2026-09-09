//! Actual full-expression actions and declaration/assignment destination conversions.
use super::provenance::Origin;
use super::{
    E, Engine, Mode, Number, NumericDomain, State,
    storage::{self, Key, Root},
};
use crate::ast::{
    CAggregateRef, CInitializer, CInitializerKind, CObjectType, CObjectTypeKind, CScalarType,
    contextual::flow_graph::Action,
};
use crate::ownership::{
    constants::{CInteger, CNumber},
    ranges::NumericLoss,
};

impl<'a> Engine<'a, '_> {
    pub(in crate::ownership) fn action(
        &mut self,
        action: &Action<'a>,
        state: &mut State<'a>,
    ) -> Result<(), E> {
        match action {
            Action::Declare(declaration) => {
                let key = Key::local(declaration.local());
                state.fresh(key.root());
                if let Some(initializer) = declaration.initializer() {
                    self.initializer(initializer, declaration.local().ty(), Some(key), state)?;
                }
            }
            Action::Assign(place, value) => {
                let number = self.expression(value, state)?;
                self.place(place, state)?;
                let before = number.is_none().then(|| state.clone());
                let mut key = self.resolve(place, state)?;
                if self.resolver.is_some() {
                    if let Some(destination) = &key {
                        if destination.exact() {
                            state.write_key(destination);
                        } else {
                            state.poison(destination.root(), Origin::Write(place));
                            key = None;
                        }
                    }
                } else if let Some(root) = Root::place(place) {
                    state.kill(&root);
                    if key.is_none() {
                        state.poison(&root, Origin::Write(place));
                    }
                } else {
                    state.opaque(&self.addresses, Origin::Write(place));
                }
                if let (Some(key), Some(number), Some(ty)) = (
                    key.clone(),
                    number,
                    storage::scalar(self.registry, place.ty())?,
                ) {
                    let number = self.destination(value, number, ty)?;
                    state.set(key, number);
                } else if let (Some(key), Some(before)) = (key, before) {
                    self.aggregate_copy(value, place.ty(), &key, &before, state)?;
                }
            }
            Action::Evaluate(effect) => {
                self.call(effect.call(), state)?;
            }
            Action::Read(value) | Action::Discard(value) => {
                self.expression(value, state)?;
            }
            Action::Return(Some(value)) => {
                self.expression(value, state)?;
            }
            Action::ScopeExit(_)
            | Action::CleanupJump(_)
            | Action::Label(_)
            | Action::FunctionEnd
            | Action::CaseEnd
            | Action::Empty
            | Action::Return(None) => {}
        }
        Ok(())
    }

    fn destination(
        &mut self,
        value: &'a crate::ast::CValue,
        mut number: Number<'a>,
        ty: CScalarType,
    ) -> Result<Number<'a>, E> {
        let converted = match number.domain.convert(ty) {
            Ok(value) => value,
            Err(_) if self.mode == Mode::Solve => {
                number.domain = NumericDomain::full(ty)?;
                number.predicate = None;
                number.losses.push(Origin::Arithmetic(value));
                return Ok(number);
            }
            Err(error) => return Err(error),
        };
        if ty != number.domain.ty() && ty != CScalarType::Bool {
            number.predicate = None;
        }
        if converted.loss == NumericLoss::MayWrap {
            number.losses.push(Origin::Arithmetic(value));
        }
        number.domain = converted.domain;
        Ok(number)
    }

    fn initializer(
        &mut self,
        initializer: &'a CInitializer,
        destination: &CObjectType,
        key: Option<Key>,
        state: &mut State<'a>,
    ) -> Result<(), E> {
        match initializer.kind() {
            CInitializerKind::Expression(value) => {
                let number = self.expression(value, state)?;
                let before = number.is_none().then(|| state.clone());
                if let (Some(key), Some(number), Some(ty)) = (
                    key.clone(),
                    number,
                    storage::scalar(self.registry, destination)?,
                ) {
                    state.set(key, self.destination(value, number, ty)?);
                } else if let (Some(key), Some(before)) = (key, before) {
                    self.aggregate_copy(value, destination, &key, &before, state)?;
                }
            }
            CInitializerKind::Zero(_) => {
                if let Some(key) = key {
                    self.zero(destination, key, state)?;
                }
            }
            CInitializerKind::Array { elements, .. } => {
                let canonical = destination.canonical();
                let CObjectTypeKind::Array { element, .. } = canonical.kind() else {
                    return Err(E::ExpectedNumericValue);
                };
                for (index, initializer) in elements.iter().enumerate() {
                    self.initializer(
                        initializer,
                        element,
                        key.as_ref().map(|key| key.index(index as u64)),
                        state,
                    )?;
                }
            }
            CInitializerKind::Struct { members, .. } => {
                for (member, initializer) in members {
                    self.initializer(
                        initializer,
                        member.ty(),
                        key.as_ref().map(|key| key.member(member)),
                        state,
                    )?;
                }
            }
            CInitializerKind::Union { member, value, .. } => {
                self.initializer(value, member.ty(), key.map(|key| key.member(member)), state)?;
            }
        }
        Ok(())
    }

    fn zero(&mut self, ty: &CObjectType, key: Key, state: &mut State<'a>) -> Result<(), E> {
        if let Some(ty) = storage::scalar(self.registry, ty)? {
            let value = if ty == CScalarType::F64 {
                CNumber::Double(0.0)
            } else {
                CNumber::Integer(CInteger::checked(ty, 0)?)
            };
            state.set(key, Number::domain(NumericDomain::exact(value)));
        } else {
            let owner = match ty.canonical().kind() {
                CObjectTypeKind::Struct(value) => Some(CAggregateRef::Struct(value.clone())),
                // Active union-member proof and compressed zero-array facts
                // belong to storage composition; unknown numeric reads are safe.
                _ => None,
            };
            if let Some(owner) = owner {
                let members = self
                    .registry
                    .members(&owner)?
                    .ok_or(E::IncompleteLayout)?
                    .to_vec();
                for member in members {
                    self.zero(member.ty(), key.member(&member), state)?;
                }
            }
        }
        Ok(())
    }
}
