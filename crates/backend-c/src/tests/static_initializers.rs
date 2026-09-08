//! Static storage accepts only actual constant trees, including nested shapes.
use super::{
    declarations::source_file,
    registry_nominals::{key, registry},
};
use crate::ast::{
    CAggregateRef, CArrayLength, CBinaryOperator, CDeclarations, CExpressions, CFileError as E,
    CFileRole, CFunctionType, CIndexBase, CInitializerKind, CKnownConstant, CLiteral, CNullPointer,
    CObjectType, CPointerTarget, CReturnType, CScalarType as T, CSignedLiteral, CUnaryOperator,
};
#[test]
fn static_arithmetic_addresses_and_nested_initializers_have_contamination_controls() {
    let (mut registry, _) = registry();
    let source = source_file(&mut registry, "storage.c", CFileRole::GeneratedSource);
    let scalar = CObjectType::scalar(T::I32);
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(scalar.clone())));
    let array_type = CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap();
    let record = registry.declare_struct(&source, key("Record")).unwrap();
    let record_owner = CAggregateRef::Struct(record.clone());
    let array_member = registry
        .register_member(&record_owner, key("items"), array_type.clone())
        .unwrap();
    registry
        .define_aggregate(&record_owner, vec![array_member.clone()])
        .unwrap();
    let union = registry.declare_union(&source, key("Payload")).unwrap();
    let union_owner = CAggregateRef::Union(union.clone());
    let record_member = registry
        .register_member(
            &union_owner,
            key("record"),
            CObjectType::structure(record.clone()),
        )
        .unwrap();
    registry
        .define_aggregate(&union_owner, vec![record_member.clone()])
        .unwrap();
    let global = registry
        .register_object(&source, key("input"), scalar.clone())
        .unwrap();
    let scalar_out = registry
        .register_object(&source, key("scalar_out"), scalar.clone())
        .unwrap();
    let array_out = registry
        .register_object(&source, key("array_out"), array_type.clone())
        .unwrap();
    let record_out = registry
        .register_object(
            &source,
            key("record_out"),
            CObjectType::structure(record.clone()),
        )
        .unwrap();
    let union_out = registry
        .register_object(&source, key("union_out"), CObjectType::union(union.clone()))
        .unwrap();
    let pointer_out = registry
        .register_object(&source, key("pointer_out"), pointer.clone())
        .unwrap();
    let floating_out = registry
        .register_object(&source, key("floating_out"), CObjectType::scalar(T::F64))
        .unwrap();
    let function = registry
        .register_function(
            &source,
            key("function"),
            CFunctionType::new(CReturnType::Void, vec![]),
        )
        .unwrap();
    let callback_out = registry
        .register_object(
            &source,
            key("callback_out"),
            CObjectType::pointer(CPointerTarget::Function(Box::new(
                function.signature().clone(),
            ))),
        )
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let local = registry
        .register_local(&scope, key("local"), scalar.clone())
        .unwrap();
    let enumeration = registry.declare_enum(&source, key("Status")).unwrap();
    let enumerator = registry
        .register_enumerator(&enumeration, key("Ready"), 1)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![enumerator.clone()])
        .unwrap();
    let ast = CExpressions::new(&registry);
    let declarations = CDeclarations::new(&registry, source).unwrap();
    let one = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(1)))
        .unwrap();
    let two = ast
        .literal(CLiteral::Signed(CSignedLiteral::I32(2)))
        .unwrap();
    let dynamic = ast.read(ast.global(global.clone()).unwrap()).unwrap();
    let constants = [
        one.clone(),
        ast.known_constant(CKnownConstant::CharBit),
        ast.enumerator(enumerator).unwrap(),
        ast.unary(CUnaryOperator::Negate, one.clone()).unwrap(),
        ast.binary(CBinaryOperator::Add, one.clone(), two.clone())
            .unwrap(),
        ast.conditional(
            ast.literal(CLiteral::Bool(true)).unwrap(),
            one.clone(),
            two.clone(),
        )
        .unwrap(),
        ast.numeric_conversion(T::I32, ast.size_of(scalar.clone()).unwrap())
            .unwrap(),
        ast.numeric_conversion(T::I32, ast.align_of(scalar).unwrap())
            .unwrap(),
    ];
    for value in constants {
        let initializer = ast.expression_initializer(value).unwrap();
        assert!(
            declarations
                .object_definition(
                    scalar_out.clone(),
                    crate::ast::CLinkage::Internal,
                    initializer
                )
                .is_ok()
        );
    }
    let floating = ast
        .expression_initializer(ast.numeric_conversion(T::F64, one.clone()).unwrap())
        .unwrap();
    assert!(
        declarations
            .object_definition(floating_out, crate::ast::CLinkage::Internal, floating)
            .is_ok()
    );
    let callback = ast
        .expression_initializer(ast.function_address(function).unwrap())
        .unwrap();
    assert!(
        declarations
            .object_definition(callback_out, crate::ast::CLinkage::Internal, callback)
            .is_ok()
    );
    let zero = ast
        .literal(CLiteral::Signed(CSignedLiteral::Int(0)))
        .unwrap();
    let array_place = ast.global(array_out.clone()).unwrap();
    let record_place = ast.global(record_out.clone()).unwrap();
    let member_array = ast.member(record_place, array_member.clone()).unwrap();
    for value in [
        ast.literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
            .unwrap(),
        ast.address_of(ast.global(global.clone()).unwrap()).unwrap(),
        ast.address_of(
            ast.index(CIndexBase::Array(Box::new(array_place)), zero.clone())
                .unwrap(),
        )
        .unwrap(),
        ast.address_of(
            ast.index(CIndexBase::Array(Box::new(member_array)), zero)
                .unwrap(),
        )
        .unwrap(),
    ] {
        assert!(
            declarations
                .object_definition(
                    pointer_out.clone(),
                    crate::ast::CLinkage::Internal,
                    ast.expression_initializer(value).unwrap()
                )
                .is_ok()
        );
    }
    let local_address = ast
        .expression_initializer(ast.address_of(ast.local(local).unwrap()).unwrap())
        .unwrap();
    assert_eq!(
        declarations.object_definition(pointer_out, crate::ast::CLinkage::Internal, local_address),
        Err(E::ExpectedStaticInitializer)
    );
    for value in [
        dynamic.clone(),
        ast.binary(CBinaryOperator::Add, one.clone(), dynamic.clone())
            .unwrap(),
        ast.conditional(
            ast.literal(CLiteral::Bool(true)).unwrap(),
            one.clone(),
            dynamic.clone(),
        )
        .unwrap(),
    ] {
        assert_eq!(
            declarations.object_definition(
                scalar_out.clone(),
                crate::ast::CLinkage::Internal,
                ast.expression_initializer(value).unwrap()
            ),
            Err(E::ExpectedStaticInitializer)
        );
    }
    for contaminated in [false, true] {
        let first = ast.expression_initializer(one.clone()).unwrap();
        let second = ast
            .expression_initializer(if contaminated {
                dynamic.clone()
            } else {
                two.clone()
            })
            .unwrap();
        let array = ast
            .array_initializer(array_type.clone(), vec![first, second])
            .unwrap();
        let record_initializer = ast
            .struct_initializer(record.clone(), vec![(array_member.clone(), array.clone())])
            .unwrap();
        let union_initializer = ast
            .union_initializer(
                union.clone(),
                record_member.clone(),
                record_initializer.clone(),
            )
            .unwrap();
        assert!(matches!(array.kind(), CInitializerKind::Array { .. }));
        for (object, initializer) in [
            (array_out.clone(), array),
            (record_out.clone(), record_initializer),
            (union_out.clone(), union_initializer),
        ] {
            let result =
                declarations.object_definition(object, crate::ast::CLinkage::Internal, initializer);
            if contaminated {
                assert_eq!(result, Err(E::ExpectedStaticInitializer));
            } else {
                assert!(result.is_ok());
            }
        }
    }
}
