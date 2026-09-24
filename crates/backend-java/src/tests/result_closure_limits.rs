//! Original Result owners in transitive and unused nominal closure budgets.
use super::*;
use crate::{
    dialect::{JavaDependencyScope, JavaResultTypeRole},
    tests::result_imports::trace::fixture::{Mutation, packages},
};

#[test]
fn result_closure_counts_original_family_producer_and_relay_once() {
    let packages = packages(Mutation::None);
    let entry = packages.relay.functions().next().unwrap();
    let unused = packages
        .family
        .result_families()
        .nth(1)
        .unwrap()
        .ty(JavaResultTypeRole::Interface);
    let (scope, _) = JavaDependencyScope::new().import(entry.clone()).unwrap();
    let (scope, _) = scope.import_result_type(unused).unwrap();
    let bindings = scope.finish();
    // Direct owners 7 and 10, with producer 9 retained transitively. The shared
    // family is not charged again, but its unused second nominal still registers.
    assert_eq!(bindings.result_types().count(), 1);
    verify_owners(bindings.owners().collect(), &BTreeSet::new(), 3).unwrap();
    let errors = verify_owners(bindings.owners().collect(), &BTreeSet::new(), 2).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, DiagnosticCode::TargetResourceLimit);
    assert!(errors[0].message.contains("closure exceeds 2 owners"));
}
