//! Definition ownership matrix, using real Core IDs without certifying mappings.

use super::*;
use portable_codegen::RelativeOutputPath;
use portable_core_ir::{CoreDeclaration, CoreProgram};

pub(super) fn core() -> CoreProgram {
    let parsed = portable_ir::v0::from_json(include_bytes!(
        "../../../build/testdata/registration.poly.json"
    ))
    .unwrap();
    portable_core_ir::lower_checked(&portable_check::v0::check_program(parsed).unwrap()).unwrap()
}
fn key(name: &str, origin: CGeneratedOrigin) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin,
    }
}
fn file(registry: &mut CRegistry, path: &str, role: CFileRole) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(path).unwrap(),
            role,
        })
        .unwrap()
}
fn production(core: &CoreProgram) -> CGeneratedOrigin {
    CGeneratedOrigin::CoreDeclaration(
        *core
            .module()
            .declarations
            .iter()
            .find(|d| matches!(d, CoreDeclaration::Function(_)))
            .unwrap(),
    )
}

#[test]
fn every_origin_row_is_checked_against_every_file_role() {
    use CFileRole as F;
    use CGeneratedOrigin as O;
    use CSynthesisReason as S;
    let core = core();
    let mut rows = core
        .module()
        .declarations
        .iter()
        .map(|d| {
            let expected = if matches!(d, CoreDeclaration::Test(_)) {
                [false, false, false, false, false, true]
            } else {
                [true, true, false, false, true, false]
            };
            (O::CoreDeclaration(*d), expected)
        })
        .collect::<Vec<_>>();
    rows.push((
        O::CoreExpression(core.blocks().iter().find_map(|(_, b)| b.result).unwrap()),
        [false, true, false, false, true, true],
    ));
    rows.extend([
        (
            O::Synthesized(S::Runtime),
            [false, false, true, true, true, false],
        ),
        (
            O::Synthesized(S::OwnershipAdapter),
            [true, true, false, false, true, true],
        ),
        (
            O::Synthesized(S::InterfaceAdapter),
            [true, true, false, false, true, true],
        ),
        (
            O::Synthesized(S::EvaluationTemporary),
            [false, true, false, false, true, true],
        ),
        (
            O::Synthesized(S::TestHarness),
            [false, false, false, false, false, true],
        ),
        (O::Synthesized(S::PlatformAssertion), [true; 6]),
    ]);
    for (origin, permitted) in rows {
        for (role, allowed) in [
            F::GeneratedPublicHeader,
            F::GeneratedSource,
            F::RuntimePublicHeader,
            F::RuntimeSource,
            F::PrivateHeader,
            F::TestSource,
        ]
        .into_iter()
        .zip(permitted)
        {
            let mut registry = CRegistry::new();
            let file = file(&mut registry, "matrix.c", role);
            let enumeration = registry
                .declare_enum(&file, key("Choice", origin.clone()))
                .unwrap();
            let zero = registry
                .register_enumerator(
                    &enumeration,
                    key("Zero", O::Synthesized(S::PlatformAssertion)),
                    0,
                )
                .unwrap();
            registry.define_enum(&enumeration, vec![zero]).unwrap();
            let ast = CDeclarations::new(&registry, file).unwrap();
            let source = ast
                .source_file(vec![CFileItem::Declaration(
                    ast.enumeration(enumeration).unwrap(),
                )])
                .unwrap();
            let expected = if allowed {
                Ok(())
            } else {
                Err(CContextError::OriginRoleMismatch)
            };
            assert_eq!(
                registry.check_context(&[source]),
                expected,
                "{origin:?} in {role:?}"
            );
        }
    }
}

#[test]
fn private_headers_cannot_launder_the_actual_definition_family() {
    for runtime in [false, true] {
        for matching_body in [false, true] {
            let mut registry = CRegistry::new();
            let header = file(&mut registry, "private.h", CFileRole::PrivateHeader);
            let body_runtime = runtime == matching_body;
            let source = file(
                &mut registry,
                "body.c",
                if body_runtime {
                    CFileRole::RuntimeSource
                } else {
                    CFileRole::GeneratedSource
                },
            );
            let origin = if runtime {
                CGeneratedOrigin::Synthesized(CSynthesisReason::Runtime)
            } else {
                production(&core())
            };
            let function = registry
                .register_function(
                    &header,
                    key("run", origin),
                    CFunctionType::new(CReturnType::Void, vec![]),
                )
                .unwrap();
            let scope = registry
                .register_scope(
                    &function,
                    None,
                    key(
                        "root",
                        CGeneratedOrigin::Synthesized(CSynthesisReason::PlatformAssertion),
                    ),
                )
                .unwrap();
            let h = CDeclarations::new(&registry, header).unwrap();
            let c = CDeclarations::new(&registry, source).unwrap();
            let body = CStatements::new(&registry, function.clone())
                .unwrap()
                .block(scope, vec![])
                .unwrap();
            let files = [
                h.source_file(vec![CFileItem::Declaration(
                    h.function_prototype(function.clone(), CLinkage::External)
                        .unwrap(),
                )])
                .unwrap(),
                c.source_file(vec![CFileItem::Definition(
                    c.function_definition(function, CLinkage::External, vec![], body)
                        .unwrap(),
                )])
                .unwrap(),
            ];
            assert_eq!(
                registry.check_context(&files),
                if matching_body {
                    Ok(())
                } else {
                    Err(CContextError::OriginRoleMismatch)
                }
            );
        }
    }
}

#[test]
fn public_function_children_use_body_role_and_tests_may_reference_production() {
    let mut registry = CRegistry::new();
    let header = file(&mut registry, "api.h", CFileRole::GeneratedPublicHeader);
    let implementation = file(&mut registry, "api.c", CFileRole::GeneratedSource);
    let test = file(&mut registry, "test.c", CFileRole::TestSource);
    let core = core();
    let function = registry
        .register_function(
            &header,
            key("run", production(&core)),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let expression =
        CGeneratedOrigin::CoreExpression(core.blocks().iter().find_map(|(_, b)| b.result).unwrap());
    let root = registry
        .register_scope(&function, None, key("root", expression.clone()))
        .unwrap();
    let local = registry
        .register_local(
            &root,
            key("temporary", expression),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let test_origin = CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness);
    let harness = registry
        .register_function(
            &test,
            key("harness", test_origin.clone()),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let test_root = registry
        .register_scope(&harness, None, key("root", test_origin))
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let body = ast
        .block(root, vec![ast.declare(local, None).unwrap()])
        .unwrap();
    let test_ast = CStatements::new(&registry, harness.clone()).unwrap();
    let test_body = test_ast
        .block(
            test_root,
            vec![
                test_ast
                    .discard(values.function_address(function.clone()).unwrap())
                    .unwrap(),
            ],
        )
        .unwrap();
    let h = CDeclarations::new(&registry, header).unwrap();
    let c = CDeclarations::new(&registry, implementation).unwrap();
    let t = CDeclarations::new(&registry, test).unwrap();
    let files = [
        h.source_file(vec![CFileItem::Declaration(
            h.function_prototype(function.clone(), CLinkage::External)
                .unwrap(),
        )])
        .unwrap(),
        c.source_file(vec![CFileItem::Definition(
            c.function_definition(function, CLinkage::External, vec![], body)
                .unwrap(),
        )])
        .unwrap(),
        t.source_file(vec![CFileItem::Definition(
            t.function_definition(harness, CLinkage::External, vec![], test_body)
                .unwrap(),
        )])
        .unwrap(),
    ];
    registry.check_context(&files).unwrap();
}
