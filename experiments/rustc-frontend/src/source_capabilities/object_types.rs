//! Compiler-owned input for ObjectTypes; no target representation.
use super::Capability;
use rustc_middle::ty::Ty;

pub(crate) struct ObjectTypes;
pub(crate) struct TypeInput<'tcx>(pub(crate) Ty<'tcx>);
impl Capability for ObjectTypes {
    type Input<'tcx> = TypeInput<'tcx>;
}
