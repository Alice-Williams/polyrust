//! Interface declaration, conformance, and dispatch certificates.
use super::*;
use crate::ast::{JavaConstructorRef, JavaStmt, JavaTypeName, JavaValueRef, JavaVisibility};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
use crate::dialect::JavaKnownConstructor;
use portable_codegen::{GeneratedOrigin, SynthesisReason};
java_input_plan!(JavaInterfacesInput, JavaInterfacesNode);

fn representation(input: &JavaInterfacesInput) -> R {
    match input {
        JavaInterfacesInput::UninhabitedType { .. } | JavaInterfacesInput::Declaration(_) => {
            R::Declaration
        }
        JavaInterfacesInput::Type { .. } | JavaInterfacesInput::SelfValue { .. } => R::Direct,
        JavaInterfacesInput::Conformance(_)
        | JavaInterfacesInput::Coerce { .. }
        | JavaInterfacesInput::ConcreteCall(_)
        | JavaInterfacesInput::InterfaceCall(_) => R::InterfaceDispatch,
    }
}
fn verify(input: &JavaInterfacesInput, output: &JavaInterfacesNode) -> bool {
    match input {
        JavaInterfacesInput::UninhabitedType {
            interface,
            name,
            source,
        } => {
            matches!(output, JavaInterfacesNode::UninhabitedType(a) if a.kind == JavaDeclarationKind::UninhabitedEnum(*interface)
                && &a.name == name && &a.source == source && a.visibility == JavaVisibility::Private
                && a.origin == GeneratedOrigin::Synthesized(SynthesisReason::UninhabitedInterface))
        }
        JavaInterfacesInput::Type { interface } => {
            matches!(output, JavaInterfacesNode::Type(a) if a == &JavaType::Reference(JavaTypeName::Generated(*interface)))
        }
        JavaInterfacesInput::Declaration(i) => {
            let JavaInterfacesNode::Declaration(actual) = output else {
                return false;
            };
            let Some((a, tail)) = actual.split_first() else {
                return false;
            };
            if a.declared != Some(i.declared)
                || a.kind != JavaDeclarationKind::SealedInterface
                || a.visibility != java_visibility(i.visibility)
                || a.modifiers != [JavaModifier::Static]
                || a.name != identifier(&i.name)
                || !a.type_parameters.is_empty()
                || !a.record_components.is_empty()
                || a.heritage != JavaHeritage::None
                || a.permits != i.permits
                || !methods_match(&i.methods, &a.members, MethodKind::Abstract)
            {
                return false;
            }
            match &i.uninhabited {
                None => tail.is_empty(),
                Some(synthetic) => {
                    let [a] = tail else {
                        return false;
                    };
                    a.declared == Some(synthetic.declared)
                        && a.kind == JavaDeclarationKind::UninhabitedEnum(i.declared)
                        && a.visibility == JavaVisibility::Private
                        && a.modifiers.is_empty()
                        && a.name == identifier(&synthetic.name)
                        && a.type_parameters.is_empty()
                        && a.record_components.is_empty()
                        && a.heritage
                            == JavaHeritage::Interfaces(vec![JavaType::Reference(
                                JavaTypeName::Generated(i.declared),
                            )])
                        && a.permits.is_empty()
                        && methods_match(&i.methods, &a.members, MethodKind::Uninhabited)
                }
            }
        }
        JavaInterfacesInput::Conformance(i) => {
            let JavaInterfacesNode::Conformance(a) = output else {
                return false;
            };
            a.heritage
                == JavaHeritage::Interfaces(
                    i.interfaces
                        .iter()
                        .map(|id| JavaType::Reference(JavaTypeName::Generated(*id)))
                        .collect(),
                )
                && a.members.len() == i.methods.len()
                && a.members.iter().zip(&i.methods).all(|(a, i)| {
                    let JavaMember::Method(a) = a else {
                        return false;
                    };
                    a.declared
                        == JavaMethodDeclaration::Implementation {
                            method: i.method,
                            interface: i.interface_method,
                            witness: i.witness,
                        }
                        && a.annotations == [JavaAnnotation::Override]
                        && a.modifiers == [JavaModifier::Public]
                        && a.type_parameters.is_empty()
                        && a.return_type == i.return_type
                        && a.name == identifier(&i.interface_method_name)
                        && a.parameters == i.parameters
                        && a.body.as_ref() == Some(&i.body)
                })
        }
        JavaInterfacesInput::SelfValue { record } => {
            matches!(output, JavaInterfacesNode::Expression(a)
            if a.ty == JavaType::Reference(JavaTypeName::Generated(*record)) && a.kind == JavaExprKind::Value(JavaValueRef::This))
        }
        JavaInterfacesInput::Coerce {
            implementation,
            result,
            ..
        } => matches!(output, JavaInterfacesNode::Expression(a)
            if &a.ty == result && matches!(&a.kind, JavaExprKind::InterfaceCoercion { implementation: w, target, .. }
                if w == implementation && target == result)),
        JavaInterfacesInput::ConcreteCall(i) => matches!(output, JavaInterfacesNode::Expression(a)
            if a.ty == i.result && matches!(&a.kind, JavaExprKind::Call {
                callable: JavaCallableRef::Member { owner, name, origin: JavaMemberOrigin::GeneratedImplementation(method), .. },
                receiver: Some(_), arguments,
            } if owner == &i.receiver.ty && name == &identifier(&i.interface_method_name) && *method == i.method && arguments.len() == i.arguments.len())),
        JavaInterfacesInput::InterfaceCall(i) => matches!(output, JavaInterfacesNode::Expression(a)
            if a.ty == i.result && matches!(&a.kind, JavaExprKind::Call {
                callable: JavaCallableRef::Interface { symbol, signature }, receiver: Some(_), arguments,
            } if *symbol == i.symbol && signature == &i.signature && arguments.len() == i.arguments.len())),
    }
}
#[derive(Clone, Copy)]
enum MethodKind {
    Abstract,
    Uninhabited,
}
fn methods_match(
    expected: &[JavaInterfaceMethodInput],
    actual: &[JavaMember],
    kind: MethodKind,
) -> bool {
    expected.len() == actual.len() && expected.iter().zip(actual).all(|(i, a)| {
        let JavaMember::Method(a) = a else { return false; };
        if a.name != identifier(&i.name) || a.parameters != i.parameters || a.return_type != i.return_type
            || !a.type_parameters.is_empty() { return false; }
        match kind {
            MethodKind::Abstract => a.declared == JavaMethodDeclaration::Interface(i.declared)
                && a.annotations.is_empty() && a.modifiers == [JavaModifier::Public, JavaModifier::Abstract] && a.body.is_none(),
            MethodKind::Uninhabited => {
                if a.declared != JavaMethodDeclaration::UninhabitedImplementation(i.declared)
                    || a.annotations != [JavaAnnotation::Override] || a.modifiers != [JavaModifier::Public] { return false; }
                let Some(body) = &a.body else { return false; };
                matches!(body.statements.as_slice(), [JavaStmt::Throw(value)] if value.ty == JavaType::known(crate::ast::JavaKnownType::AssertionError) &&
                    matches!(&value.kind, JavaExprKind::New {
                        constructor: JavaConstructorRef::Known { constructor: JavaKnownConstructor::AssertionErrorString, owner, parameters }, arguments,
                    } if owner == &value.ty && parameters == &[JavaType::known(crate::ast::JavaKnownType::String)]
                        && arguments == &[crate::lower::string_literal("uninhabited interface")]))
            }
        }
    })
}
