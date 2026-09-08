//! Partial array/member coverage without expanding an arbitrary target bound.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn variable_index_reads_require_all_possible_local_elements() {
    for initialize_all in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let ty = CObjectType::array(
            CObjectType::scalar(CScalarType::I32),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let array = registry.register_local(&scope, key("array"), ty).unwrap();
        let index = registry
            .register_local(&scope, key("index"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let base = values.local(array.clone()).unwrap();
        let index_value = values.read(values.local(index.clone()).unwrap()).unwrap();
        let zero = values
            .literal(CLiteral::Signed(CSignedLiteral::I32(0)))
            .unwrap();
        let mut body = vec![
            ast.declare(array, None).unwrap(),
            ast.declare(index, Some(values.expression_initializer(zero).unwrap()))
                .unwrap(),
        ];
        for n in 0..if initialize_all { 2 } else { 1 } {
            let slot = values
                .index(
                    CIndexBase::Array(Box::new(base.clone())),
                    values
                        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(n)))
                        .unwrap(),
                )
                .unwrap();
            body.push(ast.assign(slot, int(&values)).unwrap());
        }
        let selected = values
            .index(CIndexBase::Array(Box::new(base)), index_value)
            .unwrap();
        body.push(ast.discard(values.read(selected).unwrap()).unwrap());
        let result = registry.check_context(&[package(&registry, file, function, scope, body)]);
        if initialize_all {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}

#[test]
fn member_read_behind_variable_array_index_cannot_hide_uninitialized_storage() {
    for initialized in [true, false] {
        let (mut registry, file, function, scope) = fixture();
        let record = registry.declare_struct(&file, key("Cell")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let member = registry
            .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        registry
            .define_aggregate(&owner, vec![member.clone()])
            .unwrap();
        let ty = CObjectType::array(
            CObjectType::structure(record),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let array = registry
            .register_local(&scope, key("array"), ty.clone())
            .unwrap();
        let index = registry
            .register_local(&scope, key("index"), CObjectType::scalar(CScalarType::I32))
            .unwrap();
        let values = CExpressions::new(&registry);
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let selected = values
            .index(
                CIndexBase::Array(Box::new(values.local(array.clone()).unwrap())),
                values.read(values.local(index.clone()).unwrap()).unwrap(),
            )
            .unwrap();
        let read = values
            .read(values.member(selected, member).unwrap())
            .unwrap();
        let zero = values
            .literal(CLiteral::Signed(CSignedLiteral::I32(0)))
            .unwrap();
        let body = vec![
            ast.declare(
                array,
                initialized.then(|| values.zero_initializer(ty).unwrap()),
            )
            .unwrap(),
            ast.declare(index, Some(values.expression_initializer(zero).unwrap()))
                .unwrap(),
            ast.discard(read).unwrap(),
        ];
        let mut source = package(&registry, file.clone(), function, scope, body);
        source.items.insert(
            0,
            CFileItem::Declaration(
                CDeclarations::new(&registry, file)
                    .unwrap()
                    .aggregate(owner)
                    .unwrap(),
            ),
        );
        let result = registry.check_context(&[source]);
        if initialized {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CContextError::UninitializedRead));
        }
    }
}

#[test]
fn huge_array_bound_does_not_require_enumerating_storage() {
    let (mut registry, file, function, scope) = fixture();
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::U8),
        CArrayLength::new(u64::MAX).unwrap(),
    )
    .unwrap();
    let array = registry
        .register_local(&scope, key("array"), ty.clone())
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let selected = values
        .index(
            CIndexBase::Array(Box::new(values.local(array.clone()).unwrap())),
            values
                .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
                .unwrap(),
        )
        .unwrap();
    let body = vec![
        ast.declare(array, Some(values.zero_initializer(ty).unwrap()))
            .unwrap(),
        ast.discard(values.read(selected).unwrap()).unwrap(),
    ];
    // Contextual coverage only. Object-size/resource rejection remains 02D/04;
    // no native fixture attempts to allocate this object.
    registry
        .check_context(&[package(&registry, file, function, scope, body)])
        .unwrap();
}

#[test]
fn all_integer_leaf_indices_retain_exact_element_coverage() {
    // Every admitted literal category, plus a registered enumerator, denotes
    // index zero. A write must cover that slot, but never its sibling.
    let literals = [
        CLiteral::Bool(false),
        CLiteral::CharByte(0),
        CLiteral::Signed(CSignedLiteral::PlainChar(0)),
        CLiteral::Signed(CSignedLiteral::Int(0)),
        CLiteral::Signed(CSignedLiteral::I8(0)),
        CLiteral::Signed(CSignedLiteral::I16(0)),
        CLiteral::Signed(CSignedLiteral::I32(0)),
        CLiteral::Signed(CSignedLiteral::I64(0)),
        CLiteral::Unsigned(CUnsignedLiteral::U8(0)),
        CLiteral::Unsigned(CUnsignedLiteral::U16(0)),
        CLiteral::Unsigned(CUnsignedLiteral::U32(0)),
        CLiteral::Unsigned(CUnsignedLiteral::U64(0)),
        CLiteral::Unsigned(CUnsignedLiteral::Size(0)),
    ];
    for literal in literals.into_iter().map(Some).chain([None]) {
        for sibling in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let enumeration = registry.declare_enum(&file, key("Index")).unwrap();
            let zero = registry
                .register_enumerator(&enumeration, key("Zero"), 0)
                .unwrap();
            registry
                .define_enum(&enumeration, vec![zero.clone()])
                .unwrap();
            let ty = CObjectType::array(
                CObjectType::scalar(CScalarType::I32),
                CArrayLength::new(2).unwrap(),
            )
            .unwrap();
            let array = registry.register_local(&scope, key("array"), ty).unwrap();
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let index = match &literal {
                Some(value) => values.literal(value.clone()).unwrap(),
                None => values.enumerator(zero).unwrap(),
            };
            let slot = |index| {
                values
                    .index(
                        CIndexBase::Array(Box::new(values.local(array.clone()).unwrap())),
                        index,
                    )
                    .unwrap()
            };
            let read_index = if sibling {
                values
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(1)))
                    .unwrap()
            } else {
                index.clone()
            };
            let statements = vec![
                ast.declare(array.clone(), None).unwrap(),
                ast.assign(slot(index), int(&values)).unwrap(),
                ast.discard(values.read(slot(read_index)).unwrap()).unwrap(),
            ];
            let mut source = package(&registry, file.clone(), function, scope, statements);
            source.items.insert(
                0,
                CFileItem::Declaration(
                    CDeclarations::new(&registry, file)
                        .unwrap()
                        .enumeration(enumeration)
                        .unwrap(),
                ),
            );
            let result = registry.check_context(&[source]);
            if sibling {
                assert_eq!(result, Err(CContextError::UninitializedRead), "{literal:?}");
            } else {
                assert_eq!(result, Ok(()), "{literal:?}");
            }
        }
    }
}
