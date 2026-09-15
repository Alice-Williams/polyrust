//! Compile-negative signatures fail only for their intended type boundary.
#![allow(dead_code)]
#[cfg(any(
    local_call_missing,
    local_call_missing_base,
    local_call_duplicate,
    local_call_wrong_capability
))]
use crate::owned_source::Builder;
#[cfg(local_call_missing)]
use crate::owned_source::local_call::OwnedLocalCall;
#[cfg(local_call_missing)]
use crate::source_capabilities::Supports;
#[cfg(local_call_missing)]
fn missing() {
    fn needs(_: impl Supports<OwnedLocalCall>) {}
    needs(
        Builder::new()
            .construction(super::mapping::BoxMapping)
            .build(),
    );
}
#[cfg(local_call_missing_base)]
fn missing_base() {
    let _ = Builder::new().local_call(super::mapping::Observe).build();
}
#[cfg(local_call_duplicate)]
fn duplicate() {
    let _ = Builder::new()
        .local_call(super::mapping::Observe)
        .local_call(super::mapping::Observe);
}
#[cfg(local_call_wrong_capability)]
fn wrong_capability() {
    let _ = Builder::new().local_call(super::mapping::BoxMapping);
}
#[cfg(local_call_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::mapping::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::mapping::Observe.lower(context, input);
}
#[cfg(local_call_private_input)]
fn fake(input: crate::owned_source::local_call::LocalCallInput<'_>) {
    let _ = crate::owned_source::local_call::LocalCallInput { ..input };
}
#[cfg(any(local_call_wrong_context, local_call_wrong_output))]
mod wrong {
    use crate::{
        owned_source::local_call::{LocalCallInput, OwnedLocalCall},
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedLocalCall;
        #[cfg(local_call_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(local_call_wrong_output)]
        type Context<'tcx> = super::super::mapping::Context<'tcx>;
        #[cfg(local_call_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(local_call_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: LocalCallInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::mapping::bindings(Wrong);
    }
}
