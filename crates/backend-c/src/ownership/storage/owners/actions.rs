//! Owner destinations are exact AST locals, never aliases or guessed names.
use super::super::{Engine, state::State, values::Pointer};
use super::{Owners, Plan, Slot, Transaction};
use crate::ast::contextual::flow_graph::Action;
use crate::ast::{
    CInitializerKind, CLocalRef, CObjectTypeKind, CPlaceKind, CPointerTarget, CValue, CValueKind,
};
use crate::ownership::{CSafetyError as E, paths::Root};

impl<'ast> Engine<'_, 'ast> {
    pub(in super::super) fn owner_action(
        &mut self,
        action: &Action<'ast>,
        memory: &State,
    ) -> Result<Plan, E> {
        let mut plan = Plan {
            owners: memory.owners.clone(),
            transferred: None,
        };
        if plan.owners.poisoned {
            return Err(E::UnprovedOwnership);
        }
        if plan.owners.pending.is_some() {
            plan.reset(action)?;
            return Ok(plan);
        }
        match action {
            Action::Declare(declaration) if self.is_owner(declaration.local()) => {
                let local = declaration.local();
                if plan
                    .owners
                    .slots
                    .get(local)
                    .is_some_and(|slot| !slot.settled())
                {
                    return Err(E::UnprovedOwnership);
                }
                plan.owners.slots.insert(local.clone(), Slot::Uninitialized);
                match declaration.initializer().map(|i| i.kind()) {
                    None => {}
                    Some(CInitializerKind::Zero(_)) => {
                        plan.owners.slots.insert(local.clone(), Slot::Empty);
                    }
                    Some(CInitializerKind::Expression(value)) => {
                        self.owner_write(&mut plan.owners, local, value, memory)?;
                    }
                    _ => return Err(E::UnprovedOwnership),
                }
            }
            Action::Assign(place, value) if !plan.owners.slots.is_empty() => {
                let path = self.place(place, memory)?;
                if let Root::Local(local) = path.root()
                    && self.is_owner(local)
                {
                    if !path.whole_root()
                        || !matches!(place.kind(), CPlaceKind::Local(exact) if exact == local)
                    {
                        return Err(E::UnprovedOwnership);
                    }
                    self.owner_write(&mut plan.owners, local, value, memory)?;
                }
            }
            Action::Evaluate(effect) => {
                self.owner_release(&mut plan.owners, effect.call(), memory)?
            }
            Action::Return(_) | Action::FunctionEnd => plan.owners.finish()?,
            _ => {}
        }
        Ok(plan)
    }

    fn is_owner(&self, local: &CLocalRef) -> bool {
        self.registry()
            .owner_slots()
            .any(|slot| slot.local() == local)
    }

    fn owner_write(
        &mut self,
        owners: &mut Owners,
        local: &CLocalRef,
        value: &'ast CValue,
        memory: &State,
    ) -> Result<(), E> {
        let old = owners.slots.get(local).ok_or(E::UnprovedOwnership)?;
        if !old.settled() {
            return Err(E::UnprovedOwnership);
        }
        let pointer = self.expression(value, memory)?.pointer()?;
        if let Some(source) = direct_read(value)
            && self.is_owner(source)
        {
            if !old.empty() || source == local {
                return Err(E::UnprovedOwnership);
            }
            let Some(Slot::Live(root)) = owners.slots.get(source) else {
                return Err(E::UnprovedOwnership);
            };
            if !matches!(&pointer, Pointer::Target(path) if path.whole_root() && path.root() == root.as_ref())
            {
                return Err(E::UnprovedOwnership);
            }
            let root = root.clone();
            owners.slots.insert(local.clone(), Slot::Live(root.clone()));
            owners.pending = Some(Transaction::Move {
                source: source.clone(),
                destination: Box::new(local.clone()),
                root,
            });
        } else if pointer == Pointer::Null {
            owners.slots.insert(local.clone(), Slot::Empty);
        } else {
            let root = owners.claim(pointer, memory, self.registry())?;
            let ty = local.ty().canonical();
            let CObjectTypeKind::Pointer(CPointerTarget::Object(target)) = ty.kind() else {
                return Err(E::UnprovedOwnership);
            };
            if !self.registry().pointee_types_match(
                root.shape().object_type().ok_or(E::UnprovedOwnership)?,
                target,
            )? {
                return Err(E::UnprovedOwnership);
            }
            owners
                .slots
                .insert(local.clone(), Slot::Live(Box::new(root)));
        }
        Ok(())
    }
}

pub(super) fn direct_read(value: &CValue) -> Option<&CLocalRef> {
    let CValueKind::Read(place) = value.kind() else {
        return None;
    };
    let CPlaceKind::Local(local) = place.kind() else {
        return None;
    };
    Some(local)
}
