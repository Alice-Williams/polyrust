//! The exact public fixture is charged by the unchanged private JVM budgets.
#[allow(dead_code)]
#[path = "../../test/canonical/fixture.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "../../test/error_family/fixture.rs"]
mod local_fixture;
#[allow(dead_code)]
#[path = "../../test/error_family/model.rs"]
mod model;
#[path = "../../test/canonical/native.rs"]
mod native;

pub(crate) fn package() -> portable_codegen::RenderReadyPackage<crate::dialect::JavaDialect> {
    model::certify(fixture::Owner::new(false).finish()).unwrap()
}

#[test]
fn canonical_owner_all_four_classfiles_fit_their_own_measured_reservations() {
    native::prove(|linked, directory| {
        let budgets = super::budget_oracle::collect(linked);
        assert_eq!(budgets.len(), 4);
        for budget in &budgets {
            assert!(budget.pool > 0 && budget.metadata > 0, "{budget:?}");
        }
        super::budget_oracle::verify_directory(directory, &budgets);
    });
}
