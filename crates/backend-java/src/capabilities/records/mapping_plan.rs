//! Independent mapping-owned output checks.
use super::*;
use crate::ast::JavaCallableRef;
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaRecordsInput, JavaRecordsNode);
fn representation(input: &JavaRecordsInput) -> R {
    match input {
        JavaRecordsInput::Declaration(_) => R::Declaration,
        JavaRecordsInput::Type { .. }
        | JavaRecordsInput::Construction { .. }
        | JavaRecordsInput::Field { .. } => R::Direct,
    }
}
fn verify(input: &JavaRecordsInput, output: &JavaRecordsNode) -> bool {
    match input {
        JavaRecordsInput::Type { record } => matches!(output, JavaRecordsNode::Type(a)
            if a == &JavaType::Reference(JavaTypeName::Generated(*record))),
        JavaRecordsInput::Declaration(i) => matches!(output, JavaRecordsNode::Declaration(a)
            if a.declared == Some(i.declared) && a.kind == JavaDeclarationKind::Record
            && a.visibility == java_visibility(i.visibility) && a.modifiers == [JavaModifier::Static]
            && a.name == identifier(&i.name) && a.type_parameters.is_empty() && a.record_components == i.components
            && a.heritage == i.heritage && a.permits.is_empty() && a.members == i.members),
        JavaRecordsInput::Construction {
            owner,
            arguments,
            result,
        } => {
            let JavaRecordsNode::Expression(a) = output else {
                return false;
            };
            if &a.ty != result {
                return false;
            }
            let owner_type = JavaType::Reference(JavaTypeName::Generated(*owner));
            let created = if result == &owner_type {
                a
            } else {
                let JavaExprKind::Cast { target, value } = &a.kind else {
                    return false;
                };
                if target != result {
                    return false;
                }
                value
            };
            created.ty == owner_type
                && matches!(&created.kind,
                JavaExprKind::New { constructor: JavaConstructorRef::Generated { owner: o, parameters }, arguments: args }
                if o == owner && parameters == &arguments.iter().map(|v| v.ty.clone()).collect::<Vec<_>>()
                    && args.len() == arguments.len())
        }
        JavaRecordsInput::Field {
            receiver,
            name,
            result,
            origin,
        } => matches!(output, JavaRecordsNode::Expression(a)
            if &a.ty == result && matches!(&a.kind,
                JavaExprKind::Call { callable: JavaCallableRef::Member { owner, name: n, origin: o, .. }, receiver: Some(_), arguments }
                if owner == &receiver.ty && n == &identifier(name) && o == origin && arguments.is_empty())),
    }
}
