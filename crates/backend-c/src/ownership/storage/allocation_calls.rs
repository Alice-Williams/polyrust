//! Only closed standard identities can establish raw allocation transitions.
use super::{
    Engine,
    state::State,
    values::{Cell, Pointer},
};
use crate::ast::{CCall, CCallableKind, CValue, CValueKind};
use crate::dialect::CKnownCall;
use crate::ownership::CSafetyError as E;

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn action_value(
        &mut self,
        value: &'ast CValue,
        state: &mut State,
    ) -> Result<Cell, E> {
        let CValueKind::Call(call) = value.kind() else {
            return self.expression(value, state);
        };
        if call.callable().kind() != &CCallableKind::Known(CKnownCall::Allocate) {
            return Err(E::UnprovedStorageCall);
        }
        for argument in call.arguments() {
            self.expression(argument, state)?;
        }
        let request = self.facts.allocation_request(call)?;
        let origin = state.allocations.start(request)?;
        state.expire_allocation(&origin);
        Ok(Cell::Pointer(Pointer::Allocation(Box::new(origin))))
    }

    pub(super) fn effect(&mut self, call: &'ast CCall, state: &mut State) -> Result<(), E> {
        if call.callable().kind() != &CCallableKind::Known(CKnownCall::Release) {
            return Err(E::UnprovedStorageCall);
        }
        match self.expression(&call.arguments()[0], state)?.pointer()? {
            Pointer::Null => Ok(()),
            Pointer::Allocation(origin) => {
                if state.allocations.release(&origin)? {
                    state.expire_allocation(&origin);
                }
                Ok(())
            }
            Pointer::Target(_) | Pointer::Function(_) | Pointer::Unknown | Pointer::Expired => {
                Err(E::InvalidAllocationRelease)
            }
        }
    }
}
