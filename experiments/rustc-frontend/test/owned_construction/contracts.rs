//! Each cfg must fail for its specified typing error, not an incidental error.
#![allow(dead_code)]

#[cfg(owned_contract_missing)]
fn missing() {
    let _ = crate::owned_source::Builder::new().build();
}

#[cfg(owned_contract_duplicate)]
fn duplicate() {
    let _ = crate::owned_source::Builder::new()
        .construction(super::assertions::Observe)
        .construction(super::assertions::Observe);
}

#[cfg(owned_contract_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::assertions::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::assertions::Observe.lower(context, input);
}

#[cfg(owned_contract_private_input)]
fn private_input<'tcx>(input: crate::owned_source::BoxConstructionInput<'tcx>) {
    let _ = crate::owned_source::BoxConstructionInput {
        owner: input.owner(),
        call: input.call(),
        argument: input.argument(),
        constructor: input.constructor(),
        arguments: input.arguments(),
        result: input.result(),
    };
}

#[cfg(any(owned_contract_wrong_context, owned_contract_wrong_output))]
mod wrong_signature {
    use crate::{
        owned_source::{BoxConstructionInput, OwnedBoxConstruction},
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedBoxConstruction;
        #[cfg(owned_contract_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(owned_contract_wrong_output)]
        type Context<'tcx> = super::super::assertions::Context<'tcx>;
        #[cfg(owned_contract_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(owned_contract_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: BoxConstructionInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::assertions::bindings(Wrong);
    }
}

#[cfg(owned_contract_wrong_capability)]
mod wrong_capability {
    use crate::source_capabilities::{Capability, Mapping};
    struct Other;
    impl Capability for Other {
        type Input<'tcx> = ();
    }
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = Other;
        type Context<'tcx> = ();
        type Output = ();
        fn lower<'tcx>(&self, _: &mut (), _: ()) -> Result<(), String> {
            Ok(())
        }
    }
    fn wrong() {
        let _ = crate::owned_source::Builder::new().construction(Wrong);
    }
}
