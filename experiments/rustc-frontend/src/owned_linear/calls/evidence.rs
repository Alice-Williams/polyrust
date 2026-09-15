//! Read-only normal-flow projections; these values cannot construct a body token.
use rustc_middle::mir;

pub(crate) struct CallSite {
    pub(super) location: mir::Location,
    pub(super) argument: mir::Local,
    pub(super) destination: mir::Local,
    pub(super) staging: Vec<mir::Location>,
}
impl CallSite {
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
    pub(crate) fn argument(&self) -> mir::Local {
        self.argument
    }
    pub(crate) fn destination(&self) -> mir::Local {
        self.destination
    }
    pub(crate) fn staging(&self) -> &[mir::Location] {
        &self.staging
    }
}
pub(crate) enum BodyStep {
    Allocation(CallSite),
    LocalCall(CallSite),
    Move(mir::Location),
}
pub(crate) enum BodyEnding {
    ReadDrop {
        cast: mir::Location,
        read: mir::Location,
        drop: mir::Location,
    },
    OwnerReturn(mir::Location),
    DirectReturn,
}
