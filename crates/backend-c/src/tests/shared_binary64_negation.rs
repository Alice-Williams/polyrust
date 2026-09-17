//! Floating negate is distinct from the guarded signed-integer operation.
use super::*;
use crate::dialect::CDependencyFunction;

pub(super) fn owner(crate_id: u64, dependencies: &[CDependencyFunction]) -> CDependencyApi {
    let mut fixture = if dependencies.is_empty() {
        f::fixture(crate_id, &[CScalarType::F64; 2], &[], &[None, None])
    } else {
        assert_eq!(dependencies.len(), 2);
        f::materialized_double_calls(crate_id, dependencies)
    };
    let expressions = CExpressions::new(fixture.registry.registrations());
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let items = fixture.files[1]
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let CFileItem::Definition(definition) = item else {
                panic!("function")
            };
            let CDefinitionKind::Function {
                function,
                linkage,
                parameters,
                body,
            } = definition.kind()
            else {
                panic!("function")
            };
            let mut prefix = body.statements().to_vec();
            let returned = prefix.pop().unwrap();
            let CStatementKind::Return(Some(operand)) = returned.kind() else {
                panic!("fixture returns its input or materialized call");
            };
            let mut value = expressions
                .unary(CUnaryOperator::Negate, operand.clone())
                .unwrap();
            if dependencies.is_empty() && index == 1 {
                value = expressions.unary(CUnaryOperator::Negate, value).unwrap();
            }
            assert_eq!(
                value.ty().kind(),
                &CObjectTypeKind::Scalar(CScalarType::F64)
            );
            let statements =
                CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
            prefix.push(statements.return_statement(Some(value)).unwrap());
            let body = statements.block(body.scope().clone(), prefix).unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    fixture.files[1] = declarations.source_file(items).unwrap();
    api(&fixture)
}

#[test]
fn primitive_and_imported_double_negation_have_exact_certificates() {
    let first = owner(96, &[]);
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = owner(97, &imports);
    for api in [&first, &second] {
        assert_eq!(api.functions().count(), 2);
        let output = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(
                !text.contains("runtime") && !text.contains("memcpy") && !text.contains("0.0 -")
            );
        }
    }
}

#[path = "shared_binary64_negation_native.rs"]
mod native;
