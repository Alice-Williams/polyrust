//! Compile-negative signatures fail only for their intended type boundary.
#![allow(dead_code)]
#[cfg(any(
    box_clone_missing,
    box_clone_missing_base,
    box_clone_duplicate,
    box_clone_wrong_capability
))]
use crate::owned_source::Builder;
#[cfg(box_clone_missing)]
use crate::owned_source::cloning::OwnedBoxClone;
#[cfg(box_clone_missing)]
use crate::source_capabilities::Supports;
#[cfg(box_clone_missing)]
fn missing() {
    fn needs(_: impl Supports<OwnedBoxClone>) {}
    needs(
        Builder::new()
            .construction(super::mapping::BoxMapping)
            .build(),
    );
}
#[cfg(box_clone_missing_base)]
fn missing_base() {
    let _ = Builder::new().box_clone(super::mapping::Observe).build();
}
#[cfg(box_clone_duplicate)]
fn duplicate() {
    let _ = Builder::new()
        .box_clone(super::mapping::Observe)
        .box_clone(super::mapping::Observe);
}
#[cfg(box_clone_wrong_capability)]
fn wrong_capability() {
    let _ = Builder::new().box_clone(super::mapping::BoxMapping);
}
#[cfg(box_clone_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::mapping::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::mapping::Observe.lower(context, input);
}
#[cfg(box_clone_private_input)]
fn fake(input: crate::owned_source::cloning::BoxCloneInput<'_>) {
    let _ = crate::owned_source::cloning::BoxCloneInput { ..input };
}
#[cfg(any(box_clone_wrong_context, box_clone_wrong_output))]
mod wrong {
    use crate::{
        owned_source::cloning::{BoxCloneInput, OwnedBoxClone},
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedBoxClone;
        #[cfg(box_clone_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(box_clone_wrong_output)]
        type Context<'tcx> = super::super::mapping::Context<'tcx>;
        #[cfg(box_clone_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(box_clone_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: BoxCloneInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::mapping::bindings(Wrong);
    }
}
