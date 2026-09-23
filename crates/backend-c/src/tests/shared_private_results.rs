//! Exact private result transport admission, identities and native proof.
use super::{
    CDialect, CStructuralRenderer, project_c_package,
    result_fixture::{Mutation, fixture},
};
use crate::ast::*;
use portable_codegen::*;

#[path = "shared_private_results_native.rs"]
mod native;

fn linked(mode: Mutation) -> Result<LinkedTargetPackage<CDialect>, String> {
    let (registry, source) = fixture(mode);
    let ast = project_c_package(registry, vec![source]).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))
}

#[test]
fn private_result_certifies_initialized_copies_and_calls() {
    let linked = linked(Mutation::None).unwrap();
    let measured = super::resources::measure(&linked.files()[0].items()[0]).unwrap();
    assert_eq!(measured.function_frames.len(), 4);
    // Four parameters/locals groups: 13 + 16 + 9 + 22 bytes, including
    // each complete 8-byte aggregate rather than only its scalar payload.
    assert_eq!(measured.automatic_bytes, 60);
    assert!(measured.value_bytes >= 3 * 8);
    assert!(measured.frame_bound > *measured.function_frames.values().max().unwrap());
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output = render_certified_package(&CStructuralRenderer, &certified).unwrap();
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("text")
    };
    assert!(text.len() as u64 <= measured.source_bound);
    assert!(!text.contains("runtime") && !text.contains("malloc") && !text.contains("union "));
}

#[test]
fn private_result_does_not_open_public_signatures_order_or_recursion() {
    for (mode, diagnostic) in [
        (Mutation::Public, "scalar parameters"),
        (
            Mutation::LateDeclaration,
            "record declarations before signatures",
        ),
        (Mutation::Recursive, "call"),
        (Mutation::Uninitialized, "initialized locals"),
    ] {
        let error = match linked(mode) {
            Err(error) => error,
            Ok(package) => format!(
                "{:?}",
                certify_resolved_package(&CDialect, package).unwrap_err()
            ),
        };
        assert!(
            error.to_lowercase().contains(diagnostic),
            "{mode:?}: {error}"
        );
    }
}

fn record(r: &mut CRegistry, fields: &[CObjectType], suffix: &str, complete: bool) -> CObjectType {
    let file = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{suffix}.c")).unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let record = r.declare_struct(&file, super::tests::key(suffix)).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let members = fields
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            r.register_member(&owner, super::tests::key(&format!("field{i}")), ty.clone())
                .unwrap()
        })
        .collect();
    if complete {
        r.define_aggregate(&owner, members).unwrap();
    }
    CObjectType::structure(record)
}

#[test]
fn private_result_layout_is_exact_complete_registered_and_not_name_based() {
    use crate::ownership::value_transport::scalar_result as admits;
    let b = CObjectType::scalar(CScalarType::Bool);
    let i = CObjectType::scalar(CScalarType::I32);
    let mut r = CRegistry::new();
    let valid = record(&mut r, &[b.clone(), i.clone()], "arbitrary_names", true);
    assert!(admits(Some(&r), &valid));
    let layout = crate::ownership::layout::Layouts::new(&r)
        .object(&valid)
        .unwrap();
    assert_eq!((layout.size(), layout.alignment()), (8, 4));
    assert!(!admits(None, &valid));
    assert!(!admits(Some(&CRegistry::new()), &valid));
    assert!(!admits(
        Some(&r),
        &valid.clone().with_constness(CConstness::Const).unwrap()
    ));
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(i.clone())));
    for (index, fields) in [
        vec![i.clone(), b.clone()],
        vec![b.clone()],
        vec![b.clone(), i.clone(), i.clone()],
        vec![b.clone(), CObjectType::scalar(CScalarType::I64)],
        vec![b.clone(), pointer.clone()],
        vec![b.clone(), valid.clone()],
        vec![
            b.clone(),
            i.clone().with_constness(CConstness::Const).unwrap(),
        ],
    ]
    .into_iter()
    .enumerate()
    {
        let invalid = record(&mut r, &fields, &format!("invalid{index}"), true);
        assert!(!admits(Some(&r), &invalid), "{index}");
    }
    let incomplete = record(&mut r, &[b.clone(), i.clone()], "incomplete", false);
    assert!(!admits(Some(&r), &incomplete));
    assert!(!admits(Some(&r), &pointer));
    let file = r
        .register_file(CFileKey {
            path: RelativeOutputPath::new("union.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let union = r
        .declare_union(&file, super::tests::key("union_result"))
        .unwrap();
    assert!(!admits(Some(&r), &CObjectType::union(union)));
    let other = record(&mut r, &[b, i], "other", true);
    let CObjectTypeKind::Struct(original) = valid.kind() else {
        panic!("struct")
    };
    let CObjectTypeKind::Struct(other) = other.kind() else {
        panic!("struct")
    };
    let members = r
        .members(&CAggregateRef::Struct(original.clone()))
        .unwrap()
        .unwrap();
    let e = CExpressions::new(&r);
    let initializers: Vec<_> = members
        .iter()
        .map(|member| {
            let value = match member.ty().kind() {
                CObjectTypeKind::Scalar(CScalarType::Bool) => CLiteral::Bool(false),
                _ => CLiteral::Signed(CSignedLiteral::I32(0)),
            };
            (
                member.clone(),
                e.expression_initializer(e.literal(value).unwrap()).unwrap(),
            )
        })
        .collect();
    assert!(
        e.struct_initializer(original.clone(), initializers[..1].to_vec())
            .is_err()
    );
    assert!(e.struct_initializer(other.clone(), initializers).is_err());
}
