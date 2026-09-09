//! Full-expression storage transfers derived from actual immutable graph actions.
use super::{Engine, state::State, values::Cell};
use crate::ast::CInitializerKind;
use crate::ast::contextual::flow_graph::Action;
use crate::ownership::{CSafetyError as E, paths::Root};

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn action(&mut self, action: &Action<'ast>, state: &mut State) -> Result<(), E> {
        match action {
            Action::Declare(value) => {
                let root = Root::Local(value.local().clone());
                state.expire(&root);
                state.roots.insert(root.clone(), Cell::Uninitialized);
                if let Some(initializer) = value.initializer() {
                    let cell = match initializer.kind() {
                        CInitializerKind::Expression(value) => self.action_value(value, state)?,
                        _ => self.initializer(initializer, state)?,
                    };
                    state.roots.insert(root, cell);
                }
            }
            Action::Assign(place, value) => {
                let cell = self.action_value(value, state)?;
                let path = self.place(place, state)?;
                state.write(&path, &cell, self.registry())?;
            }
            Action::Read(value) | Action::Discard(value) => {
                self.action_value(value, state)?;
            }
            Action::Return(Some(value)) => {
                let cell = self.action_value(value, state)?;
                if cell.automatic_address(value.ty(), self.registry())? {
                    return Err(E::AutomaticAddressEscape);
                }
            }
            Action::Evaluate(effect) => self.effect(effect.call(), state)?,
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
}
