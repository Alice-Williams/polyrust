//! Void is a result/effect, never a value, with retained dependency costs.
use super::*;
use crate::ast::CReturnType;
use portable_codegen::*;

#[test]
fn void_chain_retains_original_authority_and_positive_stack_cost() {
    let owners = void_fixture::chain();
    for (index, owner) in owners.iter().enumerate() {
        let function = owner.functions().next().unwrap();
        assert_eq!(
            matches!(function.signature().return_type(), CReturnType::Void),
            index < 2
        );
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
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        let byte_bound = c_output_byte_bound(owner.package()).unwrap();
        let total: usize = output
            .files()
            .iter()
            .map(|file| match file.contents() {
                OutputContents::Text(text) => text.len(),
                _ => panic!("C text"),
            })
            .sum();
        assert!(total as u64 <= byte_bound);
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("C source");
            };
            assert!(!text.contains("runtime"));
            for forbidden in ["struct ", "typedef ", "poly_unit"] {
                assert!(
                    !text.contains(forbidden),
                    "unexpected unit storage: {forbidden}"
                );
            }
            if index > 0 && file.path().ends_with(".c") {
                let callee = owners[index - 1]
                    .functions()
                    .next()
                    .unwrap()
                    .symbol()
                    .as_str();
                let calls = text
                    .lines()
                    .filter(|line| {
                        let line = line.trim();
                        line.starts_with(&format!("{callee}(")) && line.ends_with(");")
                    })
                    .count();
                assert_eq!(calls, 1, "void call must render exactly once: {text}");
            }
            if index < 2 {
                assert!(text.contains("void poly_operation_"));
            }
        }
    }
}

#[test]
fn shared_signatures_reject_void_parameters() {
    use portable_codegen::TypedAstDialect;
    let signature = TargetCallableSignature {
        receiver: None,
        parameters: vec![TargetTypeRef::Primitive(CPrimitiveType::Void)],
        return_type: TargetTypeRef::Primitive(CPrimitiveType::Void),
        invocation: CInvocation::Function,
    };
    assert!(!CDialect.verify_signature(&signature).is_empty());
}

#[test]
fn void_imports_do_not_accept_forged_results_owners_or_signatures() {
    let owners = void_fixture::chain();
    let imported = owners[0].functions().next().unwrap().clone();
    let ast = void_fixture::ast(74, Some(imported), false);
    let catalogue = CDialect.package_symbol_catalogue(&ast).unwrap();
    catalogue.verify(&CDialect).unwrap();
    let recertified =
        CDependencyApi::from_certificate(void_fixture::package(71, None, false)).unwrap();
    let replacement = recertified.functions().next().unwrap().package_identity();
    assert_ne!(
        replacement,
        owners[0].functions().next().unwrap().package_identity()
    );
    for mutation in 0..5 {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_callables[0];
        match mutation {
            0 => {
                spec.signature.return_type =
                    TargetTypeRef::Primitive(CPrimitiveType::Scalar(crate::ast::CScalarType::I32))
            }
            1 => spec.signature.parameters.clear(),
            2 => spec.owner = owners[1].functions().next().unwrap().package_identity(),
            3 => spec.owner = replacement.clone(),
            _ => spec.name = crate::ast::CIdentifier::new("invented").unwrap(),
        }
        assert!(changed.verify(&CDialect).is_err(), "mutation {mutation}");
    }
    let alien = owners[0].functions().next().unwrap().function().clone();
    let registry = crate::ast::CRegistry::new();
    assert!(
        crate::ast::CExpressions::new(&registry)
            .direct(alien)
            .is_err()
    );
}
