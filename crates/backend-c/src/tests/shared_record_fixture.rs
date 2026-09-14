//! Records, nested const references and branch scopes use the real C builders.
use super::tests::key;
use crate::ast::{
    CAggregateRef, CBinaryOperator, CComment, CConstness, CDeclarations, CExpressions, CFileItem,
    CFileKey, CFileRole, CFrozenRegistry, CFunctionType, CLinkage, CLiteral, CObjectType,
    CParameterType, CPointerTarget, CRegistry, CReturnType, CReturnValue, CScalarType,
    CSignedLiteral, CSourceFile, CStatements,
};
use portable_codegen::RelativeOutputPath;

#[cfg(test)]
fn hostile_comment() -> CComment {
    CComment::new("Hostile */ ??/ \\\n#include <bad>\r\n☃")
}

pub(super) fn fixture() -> (CFrozenRegistry, CSourceFile) {
    comparison_fixture(CBinaryOperator::GreaterEqual)
}

pub(super) fn comparison_fixture(operator: CBinaryOperator) -> (CFrozenRegistry, CSourceFile) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/record.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let record = registry.declare_struct(&file, key("record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let member = registry
        .register_member(&owner, key("input"), scalar.clone())
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let function = registry
        .register_function(
            &file,
            key("identity"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
                vec![CParameterType::new(scalar).unwrap()],
            ),
        )
        .unwrap();
    // Same readable field/parameter name must not merge their identities.
    let parameter = registry
        .register_parameter(&function, 0, key("input"), CConstness::Unqualified)
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("body"))
        .unwrap();
    let yes = registry
        .register_scope(&function, Some(&scope), key("yes"))
        .unwrap();
    let no = registry
        .register_scope(&function, Some(&scope), key("no"))
        .unwrap();
    let record_ty = CObjectType::structure(record.clone());
    let local = registry
        .register_local(&scope, key("record"), record_ty.clone())
        .unwrap();
    let shared = CObjectType::pointer(CPointerTarget::Object(Box::new(
        record_ty.with_constness(CConstness::Const).unwrap(),
    )));
    let reference = registry
        .register_local(&scope, key("reference"), shared.clone())
        .unwrap();
    let nested_ty = CObjectType::pointer(CPointerTarget::Object(Box::new(
        shared.clone().with_constness(CConstness::Const).unwrap(),
    )));
    let nested = registry
        .register_local(&scope, key("nested"), nested_ty.clone())
        .unwrap();
    let expressions = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let parameter_value = expressions
        .read(expressions.parameter(parameter.clone()).unwrap())
        .unwrap();
    let initializer = expressions
        .struct_initializer(
            record,
            vec![(
                member.clone(),
                expressions.expression_initializer(parameter_value).unwrap(),
            )],
        )
        .unwrap();
    let address = expressions
        .address_of(expressions.local(local.clone()).unwrap())
        .unwrap();
    let address = expressions.add_const(shared, address).unwrap();
    let reference_initializer = expressions.expression_initializer(address).unwrap();
    let address = expressions
        .address_of(expressions.local(reference.clone()).unwrap())
        .unwrap();
    let address = expressions.add_const(nested_ty, address).unwrap();
    let nested_initializer = expressions.expression_initializer(address).unwrap();
    let pointer = expressions
        .read(expressions.local(nested.clone()).unwrap())
        .unwrap();
    let pointer = expressions
        .read(expressions.dereference(pointer).unwrap())
        .unwrap();
    let record_place = expressions.dereference(pointer).unwrap();
    let value = expressions
        .read(expressions.member(record_place, member).unwrap())
        .unwrap();
    let minimum = expressions
        .literal(CLiteral::Signed(CSignedLiteral::I32(i32::MIN)))
        .unwrap();
    let zero = expressions
        .literal(CLiteral::Signed(CSignedLiteral::I32(0)))
        .unwrap();
    let condition = expressions.binary(operator, value.clone(), zero).unwrap();
    let condition = expressions
        .numeric_conversion(CScalarType::Bool, condition)
        .unwrap();
    let yes = statements
        .block(yes, vec![statements.return_statement(Some(value)).unwrap()])
        .unwrap();
    let no = statements
        .block(
            no,
            vec![statements.return_statement(Some(minimum)).unwrap()],
        )
        .unwrap();
    let body = statements
        .block(
            scope,
            vec![
                statements.declare(local, Some(initializer)).unwrap(),
                statements
                    .declare(reference, Some(reference_initializer))
                    .unwrap(),
                statements
                    .declare(nested, Some(nested_initializer))
                    .unwrap(),
                statements.if_statement(condition, yes, no).unwrap(),
            ],
        )
        .unwrap();
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let source = declarations
        .source_file(vec![
            CFileItem::Comment(hostile_comment()),
            CFileItem::Declaration(declarations.aggregate(owner).unwrap()),
            CFileItem::Declaration(
                declarations
                    .function_prototype(function.clone(), CLinkage::External)
                    .unwrap(),
            ),
            CFileItem::Definition(
                declarations
                    .function_definition(function, CLinkage::External, vec![parameter], body)
                    .unwrap(),
            ),
        ])
        .unwrap();
    (registry.freeze(), source)
}
