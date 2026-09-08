//! Native portable-test service certificates.
mod cases;
use super::{JavaPortableTestHarnessInput, JavaPortableTestsInput, JavaPortableTestsNode};
use crate::ast::{
    JavaArrayOwnership, JavaCallableRef, JavaDeclarationKind, JavaExpr, JavaExprKind, JavaHeritage,
    JavaKnownType, JavaLocalFinality, JavaMember, JavaMemberOrigin, JavaMethodDeclaration,
    JavaModifier, JavaPrimitive, JavaStmt, JavaType, JavaTypeDeclaration, JavaVisibility,
};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::lower::{i32_literal, identifier};
java_input_plan!(JavaPortableTestsInput, JavaPortableTestsNode);
fn representation(input: &JavaPortableTestsInput) -> R {
    match input {
        JavaPortableTestsInput::FunctionInvocation(_) => R::Direct,
        JavaPortableTestsInput::MethodInvocation(_) => R::InterfaceDispatch,
        JavaPortableTestsInput::Case(_) => R::StructuredControl,
        JavaPortableTestsInput::Harness(_) => R::Declaration,
    }
}
fn verify(input: &JavaPortableTestsInput, output: &JavaPortableTestsNode) -> bool {
    match input {
        JavaPortableTestsInput::FunctionInvocation(i) => {
            matches!(output, JavaPortableTestsNode::Expression(a)
            if a.ty == i.signature.result && matches!(&a.kind, JavaExprKind::Call {
                callable: JavaCallableRef::Generated { symbol, signature }, receiver: None, arguments,
            } if *symbol == i.symbol && signature == &i.signature && arguments.len() == i.arguments.len()))
        }
        JavaPortableTestsInput::MethodInvocation(i) => {
            matches!(output, JavaPortableTestsNode::Expression(a)
            if a.ty == i.result && matches!(&a.kind, JavaExprKind::Call {
                callable: JavaCallableRef::Member { owner, name, origin: JavaMemberOrigin::GeneratedImplementation(method), .. },
                receiver: Some(_), arguments,
            } if owner == &i.receiver.ty && name == &identifier(&i.method_name) && *method == i.method && arguments.len() == i.arguments.len()))
        }
        JavaPortableTestsInput::Case(i) => {
            matches!(output, JavaPortableTestsNode::Case(a) if cases::verify(i, a))
        }
        JavaPortableTestsInput::Harness(i) => {
            matches!(output, JavaPortableTestsNode::Harness(a) if harness(i, a))
        }
    }
}
fn harness(i: &JavaPortableTestHarnessInput, a: &JavaTypeDeclaration) -> bool {
    if a.declared.is_some()
        || a.kind != JavaDeclarationKind::FinalClass
        || a.visibility != JavaVisibility::Public
        || !a.modifiers.is_empty()
        || a.name != identifier(&i.class_name)
        || !a.type_parameters.is_empty()
        || !a.record_components.is_empty()
        || a.heritage != JavaHeritage::None
        || !a.permits.is_empty()
    {
        return false;
    }
    let [
        JavaMember::Constructor(constructor),
        JavaMember::Method(main),
    ] = a.members.as_slice()
    else {
        return false;
    };
    if constructor.name != identifier(&i.class_name)
        || constructor.modifiers != [JavaModifier::Private]
        || !constructor.parameters.is_empty()
        || !constructor.body.statements.is_empty()
        || main.declared != JavaMethodDeclaration::Structural
        || !main.annotations.is_empty()
        || main.modifiers != [JavaModifier::Public, JavaModifier::Static]
        || !main.type_parameters.is_empty()
        || main.return_type != JavaType::primitive(JavaPrimitive::Void)
        || main.name != identifier("main")
    {
        return false;
    }
    let [parameter] = main.parameters.as_slice() else {
        return false;
    };
    if parameter.name != identifier("arguments")
        || !parameter.final_parameter
        || parameter.ty
            != (JavaType::Array {
                component: Box::new(JavaType::known(JavaKnownType::String)),
                ownership: JavaArrayOwnership::DefensiveCopyBoundary,
            })
    {
        return false;
    }
    let Some(body) = &main.body else {
        return false;
    };
    let [
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty,
            name,
            value: Some(zero),
        },
        rest @ ..,
    ] = body.statements.as_slice()
    else {
        return false;
    };
    if ty != &JavaType::primitive(JavaPrimitive::Int)
        || name != &identifier("completed")
        || zero != &i32_literal(0)
    {
        return false;
    }
    let local = JavaExpr::local(ty.clone(), name.clone());
    let mut rest = rest;
    for case in &i.cases {
        let Some([JavaStmt::Assign { target, value }, tail @ ..]) =
            rest.strip_prefix(case.as_slice())
        else {
            return false;
        };
        if target != &local
            || value.ty != *ty
            || !matches!(&value.kind,
            JavaExprKind::Binary { operator: crate::ast::JavaBinaryOperator::Add, left, right }
                if left.as_ref() == &local && right.as_ref() == &i32_literal(1))
        {
            return false;
        }
        rest = tail;
    }
    let [assertion] = rest else {
        return false;
    };
    let expected_message = format!(
        "portable conformance inventory mismatch: expected {} tests",
        i.expected_test_count,
    );
    let Some(condition) = cases::asserted(assertion, false, &expected_message) else {
        return false;
    };
    condition.ty == JavaType::primitive(JavaPrimitive::Boolean)
        && matches!(&condition.kind,
        JavaExprKind::Binary { operator: crate::ast::JavaBinaryOperator::Equal, left, right }
            if left.as_ref() == &local && right.as_ref() == &i32_literal(i.expected_test_count))
}
