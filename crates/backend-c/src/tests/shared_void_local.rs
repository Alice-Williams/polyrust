//! Real local void effects participate in declaration order and effect proofs.
use super::*;
use crate::ast::*;
use portable_codegen::*;

fn fixture(call: bool, late: bool) -> (CFrozenRegistry, CSourceFile) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("void_local.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let functions: Vec<_> = ["caller", "callee"]
        .iter()
        .map(|name| {
            registry
                .register_function(
                    &file,
                    tests::key(name),
                    CFunctionType::new(CReturnType::Void, vec![]),
                )
                .unwrap()
        })
        .collect();
    let scopes: Vec<_> = functions
        .iter()
        .map(|function| {
            registry
                .register_scope(function, None, tests::key("body"))
                .unwrap()
        })
        .collect();
    let expressions = CExpressions::new(&registry);
    let mut bodies = vec![];
    for (index, function) in functions.iter().enumerate() {
        let statements = CStatements::new(&registry, function.clone()).unwrap();
        let mut body = vec![];
        if index == 0 && call {
            body.push(
                statements
                    .evaluate(
                        expressions
                            .call_effect(expressions.direct(functions[1].clone()).unwrap(), vec![])
                            .unwrap(),
                    )
                    .unwrap(),
            );
        }
        body.push(statements.return_statement(None).unwrap());
        bodies.push(statements.block(scopes[index].clone(), body).unwrap());
    }
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let prototypes: Vec<_> = functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            CFileItem::Declaration(
                declarations
                    .function_prototype(
                        function.clone(),
                        if index == 0 {
                            CLinkage::External
                        } else {
                            CLinkage::Internal
                        },
                    )
                    .unwrap(),
            )
        })
        .collect();
    let definitions: Vec<_> = functions
        .iter()
        .enumerate()
        .map(|(index, function)| {
            CFileItem::Definition(
                declarations
                    .function_definition(
                        function.clone(),
                        if index == 0 {
                            CLinkage::External
                        } else {
                            CLinkage::Internal
                        },
                        vec![],
                        bodies[index].clone(),
                    )
                    .unwrap(),
            )
        })
        .collect();
    let items = if late {
        vec![
            prototypes[0].clone(),
            definitions[0].clone(),
            prototypes[1].clone(),
            definitions[1].clone(),
        ]
    } else {
        vec![
            prototypes[0].clone(),
            prototypes[1].clone(),
            definitions[0].clone(),
            definitions[1].clone(),
        ]
    };
    let source = declarations.source_file(items).unwrap();
    (registry.freeze(), source)
}

#[test]
fn void_private_call_requires_prior_prototype_and_counts_as_syntactic_use() {
    for (call, late) in [(true, false), (true, true), (false, false)] {
        let (registry, source) = fixture(call, late);
        let ast = project_c_package(registry, vec![source]);
        if call && !late {
            let checked = verify_unresolved_package(&CDialect, ast.unwrap()).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
            let package = certify_resolved_package(&CDialect, linked).unwrap();
            let output = render_certified_package(&CStructuralRenderer, &package).unwrap();
            let OutputContents::Text(text) = output.files()[0].contents() else {
                panic!("C source");
            };
            assert!(text.contains("static void"));
        } else {
            let errors = ast.unwrap_err();
            assert!(
                errors.iter().any(|error| error.message.contains(if late {
                    "prototype before each call"
                } else {
                    "syntactic use"
                })),
                "{errors:?}"
            );
        }
    }
}
