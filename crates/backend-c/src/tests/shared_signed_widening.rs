//! Exact lossless casts retain dependency, resource and source-package proofs.
use super::{dependency_fixture as f, project_c_package};
use crate::{
    ast::*,
    dialect::{CDependencyApi, CDialect, CStructuralRenderer},
};
use portable_codegen::*;

#[path = "shared_signed_widening_fixture.rs"]
mod fixture;
#[path = "shared_signed_widening_native.rs"]
mod native;

fn api(source: &f::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(source)).unwrap(),
    )
    .unwrap()
}

fn chain(mode: fixture::Body) -> Vec<CDependencyApi> {
    let input = f::api(811, &[CScalarType::I32]);
    let widened = api(&fixture::build(
        812,
        input.functions().next().cloned(),
        mode,
    ));
    let forwarded = api(&fixture::build(
        813,
        widened.functions().next().cloned(),
        fixture::Body::Forward,
    ));
    vec![input, widened, forwarded]
}

#[test]
fn signed_widening_certifies_exact_width_and_recursively_checks_operand() {
    for mode in [fixture::Body::Widen, fixture::Body::NestedComplement(32)] {
        let source = fixture::build(812, None, mode);
        let owner = api(&source);
        assert_eq!(owner.functions().count(), 1);
        let CFileItem::Definition(definition) = &source.files[1].items()[0] else {
            panic!("definition")
        };
        let CDefinitionKind::Function { body, .. } = definition.kind() else {
            panic!("function")
        };
        let CStatementKind::Return(Some(value)) = body.statements().last().unwrap().kind() else {
            panic!("return")
        };
        let CValueKind::Convert {
            conversion: CConversion::Numeric(CScalarType::I64),
            operand,
        } = value.kind()
        else {
            panic!("exact widening")
        };
        assert_eq!(
            operand.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::I32)
        );
        assert_eq!(
            value.ty().kind(),
            &CObjectTypeKind::Scalar(CScalarType::I64)
        );
    }
    for (mode, diagnostic) in [
        (fixture::Body::UnadmittedDivide, "only scalar comparisons"),
        (
            fixture::Body::NestedComplement(128),
            "verifier budget exceeded",
        ),
    ] {
        let source = fixture::build(812, None, mode);
        let errors = project_c_package(source.registry, source.files).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains(diagnostic)),
            "{errors:?}"
        );
    }
    // Syntactic/safety certification alone must not be mistaken for equivalence.
    api(&fixture::build(812, None, fixture::Body::Zero));
}

#[test]
fn signed_widening_preserves_original_imports_and_resource_bounds() {
    let owners = chain(fixture::Body::Widen);
    let foreign = fixture::build(
        814,
        owners[0].functions().next().cloned(),
        fixture::Body::Widen,
    );
    let local = fixture::build(812, None, fixture::Body::Widen);
    assert!(
        CExpressions::new(local.registry.registrations())
            .direct(foreign.imported[0].clone())
            .is_err()
    );
    for (index, owner) in owners.iter().enumerate() {
        assert!(owner.system_libraries().is_empty());
        for file in owner.package().ast().files() {
            for standard in [super::CStdType::U32, super::CStdType::U64] {
                assert!(!file.items()[0].unit.data.standards.contains(&standard));
            }
        }
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        let mut bytes = 0;
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(
                !text.contains("runtime") && !text.contains("goto") && !text.contains("math.h")
            );
            bytes += text.len() as u64;
        }
        assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
        let function = owner.functions().next().unwrap();
        assert!(function.stack_bound_bytes() > 0);
        if index > 0 {
            assert!(
                function.stack_bound_bytes()
                    > owners[index - 1]
                        .functions()
                        .next()
                        .unwrap()
                        .stack_bound_bytes()
            );
        }
    }
}
