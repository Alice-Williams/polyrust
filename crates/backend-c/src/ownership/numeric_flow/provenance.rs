//! Closed reasons why an extent computation is not yet proved nonwrapping.
use crate::ast::{CCall, CObjectRef, CPlace, CValue};

#[derive(Clone, Debug)]
pub(super) enum Origin<'a> {
    Arithmetic(&'a CValue),
    Aggregate(&'a CValue),
    Read(&'a CPlace),
    Write(&'a CPlace),
    Call(&'a CCall),
    Global(Box<CObjectRef>),
}
impl PartialEq for Origin<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Arithmetic(a), Self::Arithmetic(b))
            | (Self::Aggregate(a), Self::Aggregate(b)) => std::ptr::eq(*a, *b),
            (Self::Read(a), Self::Read(b)) | (Self::Write(a), Self::Write(b)) => {
                std::ptr::eq(*a, *b)
            }
            (Self::Call(a), Self::Call(b)) => std::ptr::eq(*a, *b),
            (Self::Global(a), Self::Global(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Origin<'_> {}

pub(super) fn merge<'a>(target: &mut Vec<Origin<'a>>, source: &[Origin<'a>]) {
    for origin in source {
        if !target.contains(origin) {
            target.push(origin.clone());
        }
    }
}
