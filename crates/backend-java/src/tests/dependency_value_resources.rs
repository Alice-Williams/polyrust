//! Values and functions share one registration budget, even when unused.
use super::*;
use crate::{dialect::JavaDialect, tests::source_constant_consumer_fixture as f};
use portable_codegen::{TargetLinker, verify_unresolved_package};

#[test]
fn values_and_functions_charge_shared_registration_owner_and_name_limits() {
    for mixed in [false, true] {
        let fixture = f::Fixture::new(mixed);
        let paths: Vec<_> = fixture
            .values
            .iter()
            .map(|v| v.constant().path())
            .chain(fixture.callable.iter().map(|c| c.function().path()))
            .collect();
        for used in [false, true] {
            let verified = verify_unresolved_package(&JavaDialect, fixture.draft(used)).unwrap();
            let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
            let exact = || Limits {
                bindings: paths.len(),
                owners: 1,
                names: paths.iter().map(|p| p.encoded_len()).sum(),
                name: paths.iter().map(|p| p.encoded_len()).max().unwrap(),
            };
            assert!(check(&linked, &exact()).is_empty());
            for fault in 0..4 {
                let mut limits = exact();
                match fault {
                    0 => limits.bindings -= 1,
                    1 => limits.owners -= 1,
                    2 => limits.names -= 1,
                    3 => limits.name -= 1,
                    _ => unreachable!(),
                }
                assert!(
                    !check(&linked, &limits).is_empty(),
                    "mixed={mixed} used={used} fault={fault}"
                );
            }
        }
    }
}
