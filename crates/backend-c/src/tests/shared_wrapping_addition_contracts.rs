//! New numeric children participate in recursive admission and verifier budgets.
use super::*;

#[test]
fn wrapping_addition_recurses_into_unsigned_children_and_enforces_depth_budget() {
    for width in [CScalarType::I32, CScalarType::I64] {
        let source = fixture::build_widths(
            701,
            &[],
            fixture::Body::Addition(Variant::NestedComplement(32)),
            &[width],
        );
        api(&source);
        for (variant, message) in [
            (Variant::NestedMultiply, "only scalar comparisons"),
            (Variant::NestedComplement(128), "verifier budget exceeded"),
        ] {
            let source =
                fixture::build_widths(701, &[], fixture::Body::Addition(variant), &[width]);
            let errors = project_c_package(source.registry, source.files).unwrap_err();
            assert!(
                errors.iter().any(|error| error.message.contains(message)),
                "{width:?} {variant:?}: {errors:?}"
            );
        }
    }
}

#[test]
fn wrapping_addition_dependencies_include_unsigned_types_only_where_used() {
    for (index, owner) in chain(Variant::Valid).iter().enumerate() {
        for file in owner.package().ast().files() {
            let unit = &file.items()[0];
            for standard in [super::super::CStdType::U32, super::super::CStdType::U64] {
                assert_eq!(unit.unit.data.standards.contains(&standard), index == 0);
            }
        }
        // The producer header enforces ABI queries for implementation and clients.
        // Its source uses unsigned locals; the signed forwarder needs neither.
        let files = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        for file in files.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            for spelling in ["uint32_t", "uint64_t"] {
                assert_eq!(text.contains(spelling), index == 0, "{}", file.path());
            }
        }
    }
}
