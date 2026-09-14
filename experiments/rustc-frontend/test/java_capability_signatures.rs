//! One independently wrong executable signature for every registered slot.
use super::{Builder, Capability, Mapping};

macro_rules! incorrect {
    ($name:ident, $slot:ident, $cap:ty, $wrong:ty, $context:ty, $output:ty) => {
        #[derive(Clone, Copy)]
        struct $name;
        impl Mapping for $name {
            #[cfg(java_contract_wrong_capability)]
            type Capability = $wrong;
            #[cfg(not(java_contract_wrong_capability))]
            type Capability = $cap;
            #[cfg(java_contract_wrong_context)]
            type Context<'tcx> = u8;
            #[cfg(not(java_contract_wrong_context))]
            type Context<'tcx> = $context;
            #[cfg(java_contract_wrong_output)]
            type Output = ();
            #[cfg(not(java_contract_wrong_output))]
            type Output = $output;

            fn lower<'tcx>(
                &self,
                _: &mut Self::Context<'tcx>,
                _: <Self::Capability as Capability>::Input<'tcx>,
            ) -> Result<Self::Output, String> {
                Err("deliberately incorrect mapping contract".into())
            }
        }
        impl $name {
            #[allow(dead_code)]
            fn must_not_compile() {
                Builder::new().$slot(Self);
            }
        }
    };
}

incorrect!(
    Literal,
    literal_values,
    super::LiteralValues,
    super::ScalarComparisons,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Value
);
incorrect!(
    Comparison,
    scalar_comparisons,
    super::ScalarComparisons,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Value
);
incorrect!(
    Places,
    resolved_places,
    super::ResolvedPlaces,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Place
);
incorrect!(
    Borrows,
    shared_borrows,
    super::SharedBorrows,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Value
);
incorrect!(
    Types,
    object_types,
    super::ObjectTypes,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::TypePlan
);
incorrect!(
    Records,
    record_initializers,
    super::RecordInitializers,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Value
);
incorrect!(
    Control,
    lexical_control,
    super::LexicalControl,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    portable_backend_java::ast::JavaBlock
);
incorrect!(
    Entry,
    entry_signatures,
    super::EntrySignatures,
    super::LiteralValues,
    (),
    portable_backend_java::ast::JavaMethodSignature
);
incorrect!(
    Calls,
    direct_calls,
    super::DirectCalls,
    super::LiteralValues,
    crate::java_lower::Reader<'tcx>,
    crate::java_lower::Value
);
incorrect!(
    Functions,
    function_signatures,
    super::FunctionSignatures,
    super::LiteralValues,
    (),
    portable_backend_java::ast::JavaMethodSignature
);
