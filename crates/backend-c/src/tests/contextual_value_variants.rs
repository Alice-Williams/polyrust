//! Every value/place reconstruction branch is exercised below a valid parent.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

pub(super) fn prove(
    registry: &CRegistry,
    file: &CFileRef,
    function: &CFunctionRef,
    scope: &CScopeRef,
    value: CValue,
) {
    let values = CExpressions::new(registry);
    let ast = CStatements::new(registry, function.clone()).unwrap();
    let wrapped = values
        .conditional(
            values.literal(CLiteral::Bool(true)).unwrap(),
            value.clone(),
            value,
        )
        .unwrap();
    let make = |value| {
        package(
            registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![ast.discard(value).unwrap()],
        )
    };
    registry
        .check_local_structure(&[make(wrapped.clone())])
        .unwrap();
    let mut bad = wrapped;
    let CValueKind::Conditional { then_value, .. } = &mut bad.kind else {
        unreachable!()
    };
    then_value.ty = if then_value.ty() == &CObjectType::scalar(CScalarType::F64) {
        CObjectType::scalar(CScalarType::I32)
    } else {
        CObjectType::scalar(CScalarType::F64)
    };
    // The enclosing conditional's cached type remains unchanged.
    assert!(registry.check_local_structure(&[make(bad)]).is_err());
}

#[test]
fn value_and_place_variants_rederive_nested_types_from_actual_children() {
    let (mut registry, file, function, scope) = fixture();
    let scalar = CObjectType::scalar(CScalarType::I32);
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let member = registry
        .register_member(&owner, key("field"), scalar.clone())
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let local = registry
        .register_local(&scope, key("local"), scalar.clone())
        .unwrap();
    let aggregate = registry
        .register_local(&scope, key("record"), CObjectType::structure(record))
        .unwrap();
    let array = registry
        .register_local(
            &scope,
            key("array"),
            CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap(),
        )
        .unwrap();
    let global = registry
        .register_object(&file, key("global"), scalar.clone())
        .unwrap();
    let callee = registry
        .register_function(
            &file,
            key("callee"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar.clone()).unwrap()),
                vec![CParameterType::new(scalar.clone()).unwrap()],
            ),
        )
        .unwrap();
    let parameter = registry
        .register_parameter(&callee, 0, key("arg"), CConstness::Unqualified)
        .unwrap();
    let enumeration = registry.declare_enum(&file, key("Choice")).unwrap();
    let enumerator = registry
        .register_enumerator(&enumeration, key("Zero"), 0)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![enumerator.clone()])
        .unwrap();
    let allocation = registry
        .register_allocation(
            &scope,
            key("allocation"),
            scalar.clone(),
            CAllocatorSource::Default,
        )
        .unwrap();
    let values = CExpressions::new(&registry);
    let null = |ty| {
        values
            .literal(CLiteral::NullPointer(CNullPointer::new(ty).unwrap()))
            .unwrap()
    };
    let pointer_ty = CObjectType::pointer(CPointerTarget::Object(Box::new(scalar.clone())));
    let pointer = null(pointer_ty.clone());
    let void_ty = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
    let zero = || {
        values
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(0)))
            .unwrap()
    };
    let places = [
        values.local(local).unwrap(),
        values.parameter(parameter).unwrap(),
        values.global(global).unwrap(),
        values
            .member(values.local(aggregate).unwrap(), member)
            .unwrap(),
        values.dereference(pointer.clone()).unwrap(),
        values
            .index(
                CIndexBase::Array(Box::new(values.local(array).unwrap())),
                zero(),
            )
            .unwrap(),
        values
            .index(CIndexBase::Pointer(Box::new(pointer.clone())), zero())
            .unwrap(),
    ];
    let mut cases = vec![
        values.literal(CLiteral::Bool(true)).unwrap(),
        int(&values),
        zero(),
        values.literal(CLiteral::CharByte(65)).unwrap(),
        pointer.clone(),
        values.enumerator(enumerator).unwrap(),
        values.function_address(callee.clone()).unwrap(),
        values
            .call_value(values.direct(callee.clone()).unwrap(), vec![int(&values)])
            .unwrap(),
        values
            .call_value(
                values
                    .indirect(values.function_address(callee.clone()).unwrap(), callee)
                    .unwrap(),
                vec![int(&values)],
            )
            .unwrap(),
        values.size_of(scalar.clone()).unwrap(),
        values.align_of(scalar.clone()).unwrap(),
        values
            .conditional(
                values.literal(CLiteral::Bool(false)).unwrap(),
                int(&values),
                int(&values),
            )
            .unwrap(),
        values
            .add_const(
                CObjectType::pointer(CPointerTarget::Object(Box::new(
                    scalar.with_constness(CConstness::Const).unwrap(),
                ))),
                pointer.clone(),
            )
            .unwrap(),
        values
            .object_to_void(void_ty.clone(), pointer.clone())
            .unwrap(),
        values
            .allocation_restore(allocation, null(void_ty))
            .unwrap(),
        values
            .pointer_test(CPointerTest::IsNull(Box::new(pointer.clone())))
            .unwrap(),
        values
            .pointer_test(CPointerTest::IsNonNull(Box::new(pointer.clone())))
            .unwrap(),
    ];
    let slot = null(CObjectType::pointer(CPointerTarget::Object(Box::new(
        pointer_ty,
    ))));
    cases.push(
        values
            .pointer_test(CPointerTest::SameSlot {
                left: Box::new(slot.clone()),
                right: Box::new(slot),
            })
            .unwrap(),
    );
    for place in places {
        let read = values.read(place.clone()).unwrap();
        let ast = CStatements::new(&registry, function.clone()).unwrap();
        let mut bad = read.clone();
        let CValueKind::Read(child) = &mut bad.kind else {
            unreachable!()
        };
        child.ty = CObjectType::scalar(CScalarType::F64);
        let source = package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![ast.discard(bad).unwrap()],
        );
        assert!(registry.check_local_structure(&[source]).is_err());
        cases.push(read);
        cases.push(values.address_of(place).unwrap());
    }
    for scalar in CScalarType::ALL {
        cases.push(values.numeric_conversion(scalar, int(&values)).unwrap());
    }
    for operator in [
        CUnaryOperator::LogicalNot,
        CUnaryOperator::BitNot,
        CUnaryOperator::Negate,
    ] {
        let operand = if operator == CUnaryOperator::LogicalNot {
            values.literal(CLiteral::Bool(true)).unwrap()
        } else {
            int(&values)
        };
        cases.push(values.unary(operator, operand).unwrap());
    }
    use CBinaryOperator as B;
    for operator in [
        B::Add,
        B::Subtract,
        B::Multiply,
        B::Divide,
        B::Remainder,
        B::ShiftLeft,
        B::ShiftRight,
        B::BitAnd,
        B::BitOr,
        B::BitXor,
        B::Equal,
        B::NotEqual,
        B::Less,
        B::LessEqual,
        B::Greater,
        B::GreaterEqual,
        B::LogicalAnd,
        B::LogicalOr,
    ] {
        let operand = if matches!(operator, B::LogicalAnd | B::LogicalOr) {
            values.literal(CLiteral::Bool(true)).unwrap()
        } else {
            int(&values)
        };
        cases.push(values.binary(operator, operand.clone(), operand).unwrap());
    }
    use CKnownConstant as K;
    for constant in [
        K::CharBit,
        K::IntMin,
        K::IntMax,
        K::I32Min,
        K::I32Max,
        K::U32Max,
        K::I64Min,
        K::I64Max,
        K::U64Max,
        K::SizeMax,
        K::FloatRadix,
        K::DoubleMantissaDigits,
        K::DoubleMinExponent,
        K::DoubleMaxExponent,
        K::FloatEvaluationMethod,
        K::EndOfFile,
        K::StandardInput,
        K::StandardOutput,
        K::StandardError,
    ] {
        cases.push(values.known_constant(constant));
    }
    // Local reconstruction only: null dereferences/uninitialized fixtures are
    // deliberately not claimed safe by flow/range/ownership or native proof.
    for value in cases {
        prove(&registry, &file, &function, &scope, value);
    }
}
