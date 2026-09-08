use super::*;
use crate::capabilities::{
    self as c,
    support::{self, plans::values::JavaValuePlan},
};
use crate::lower::{diagnostic, i32_literal};
use portable_build::BoolValues;
use portable_diagnostics::Diagnostic;
use std::sync::atomic::{AtomicUsize, Ordering};
static STAGE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
enum Fault {
    WrongOutput,
    SelectError,
    LowerError,
}
#[derive(Clone, Copy)]
struct FaultyMapping {
    fault: Fault,
}
impl support::sealed::JavaCapabilityMapping for FaultyMapping {}
impl JavaCapabilityMapping for FaultyMapping {
    type Plan = JavaValuePlan;
    fn select_plan(&self, input: &Self::Input) -> Result<Self::Plan, Vec<Diagnostic>> {
        assert_eq!(
            STAGE.swap(1, Ordering::SeqCst),
            0,
            "select must run before lower"
        );
        match self.fault {
            Fault::SelectError => Err(vec![diagnostic("selected failure")]),
            Fault::WrongOutput | Fault::LowerError => c::JavaBoolValues.select_plan(input),
        }
    }
}
impl CapabilityMapping<JavaDialect> for FaultyMapping {
    type Capability = BoolValues;
    type Context = ();
    type Input = c::bool_values::JavaBoolValuesInput;
    type Output = support::JavaValueNode;
    type Error = Vec<Diagnostic>;
    fn lower(&self, _: &mut (), _: Self::Input) -> Result<Self::Output, Self::Error> {
        assert_eq!(STAGE.swap(2, Ordering::SeqCst), 1);
        match self.fault {
            Fault::WrongOutput => Ok(support::JavaValueNode::Expression(Box::new(i32_literal(1)))),
            Fault::LowerError => Err(vec![diagnostic("lower failure")]),
            Fault::SelectError => panic!("lower must not run after selector rejection"),
        }
    }
}

#[test]
fn builder_automatically_checks_the_same_instance_and_preserves_errors() {
    for (fault, final_stage, expected) in [
        (
            Fault::WrongOutput,
            2,
            "BoolValues mapping violated its Direct output plan",
        ),
        (Fault::SelectError, 1, "selected failure"),
        (Fault::LowerError, 2, "lower failure"),
    ] {
        STAGE.store(0, Ordering::SeqCst);
        let plugin = c::java_plugin_builder()
            .support(FaultyMapping { fault })
            .build();
        let mapping: &support::CheckedJavaMapping<FaultyMapping> =
            plugin.mapping_for::<BoolValues>();
        let errors = mapping
            .lower(&mut (), c::bool_values::JavaBoolValuesInput::Value(true))
            .err()
            .expect("rejected");
        assert!(format!("{errors:?}").contains(expected));
        assert_eq!(STAGE.load(Ordering::SeqCst), final_stage);
    }
}
