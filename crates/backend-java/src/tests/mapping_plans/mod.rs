//! Independent certificate tests always execute builder-fetched mappings.
mod boolean_logic;
mod constructors;
mod declarations;
mod intrinsics;
mod owned_roots;
mod review_regressions;
mod uninhabited;
mod wrapper;
use super::support::{JavaCapabilityMapping, JavaMappingOutput, plans::JavaMappingPlan};
use crate::dialect::JavaDialect;
use portable_build::{CapabilityMapping, Supports};

fn checked<M>(mapping: M, input: M::Input) -> (M::Plan, M::Output)
where
    M: JavaCapabilityMapping<Context = ()> + 'static,
    M::Output: JavaMappingOutput,
    super::JavaCapabilitySet: Supports<
            M::Capability,
            Dialect = JavaDialect,
            Mapping = super::support::CheckedJavaMapping<M>,
        >,
{
    let plan = mapping.select_plan(&input).expect("input plan");
    let plugin = super::java_capabilities();
    let output = plugin
        .mapping_for::<M::Capability>()
        .lower(&mut (), input)
        .expect("checked mapping");
    assert!(plan.verify_output(&output));
    (plan, output)
}
