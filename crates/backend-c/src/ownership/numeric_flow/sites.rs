//! Each runtime obligation must originate inside that exact graph action.
use super::{E, Obligation};
use crate::ast::{
    CCall, CPlace, CValue, CValueKind,
    contextual::{
        access_walk::{self as walk, Access, Visitor},
        flow_graph::Action,
    },
};

struct Search<'a, 'b> {
    obligation: &'b Obligation<'a>,
    found: bool,
}
impl Search<'_, '_> {
    fn call(&mut self, call: &CCall) {
        if let Obligation::Call { call: actual, .. } = self.obligation {
            self.found |= std::ptr::eq(*actual, call);
        }
    }
}
impl Visitor for Search<'_, '_> {
    type Error = E;
    fn place(&mut self, place: &CPlace, _: Access) -> Result<(), E> {
        if let Obligation::Index { place: actual, .. } = self.obligation {
            self.found |= std::ptr::eq(*actual, place);
        }
        Ok(())
    }
    fn value(&mut self, value: &CValue) -> Result<(), E> {
        if let Obligation::Calculation { value: actual, .. } = self.obligation {
            self.found |= std::ptr::eq(*actual, value);
        }
        if let CValueKind::Call(call) = value.kind() {
            self.call(call);
        }
        Ok(())
    }
}
pub(super) fn check(action: &Action<'_>, obligation: &Obligation<'_>) -> Result<(), E> {
    let mut search = Search {
        obligation,
        found: false,
    };
    match action {
        Action::Declare(declaration) => {
            if let Some(initializer) = declaration.initializer() {
                walk::initializer(&mut search, initializer)?;
            }
        }
        Action::Assign(place, value) => {
            walk::place(&mut search, place, Access::Write)?;
            walk::expression(&mut search, value)?;
        }
        Action::Evaluate(effect) => {
            search.call(effect.call());
            walk::call(&mut search, effect.call())?;
        }
        Action::Read(value) | Action::Discard(value) | Action::Return(Some(value)) => {
            walk::expression(&mut search, value)?
        }
        _ => {}
    }
    if search.found {
        Ok(())
    } else {
        Err(E::InvalidNumericSite)
    }
}
