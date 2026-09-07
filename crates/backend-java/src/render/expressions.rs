//! Structural rendering: expressions.

use super::names::{
    render_java_type, resolved_generated_member_name, resolved_name, resolved_type_name,
};
use super::render_diagnostic;
use super::statements::render_block;
use super::syntax::{binary_operator, render_literal, unary_operator};
use crate::ast::{
    JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaFieldRef, JavaResolvedName,
    JavaValueRef,
};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedSymbolId, LinkedFile, TargetSymbolRef};
use portable_diagnostics::Diagnostic;

pub(super) fn render_expr(
    value: &JavaExpr,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    match &value.kind {
        JavaExprKind::Literal(value) => Ok(render_literal(value)),
        JavaExprKind::Value(value) => {
            let value = match value {
                JavaValueRef::Local(value) => value.as_str().to_owned(),
                JavaValueRef::This => "this".to_owned(),
                JavaValueRef::Generated(value) => {
                    resolved_name(names, &TargetSymbolRef::Generated(*value))?
                }
                JavaValueRef::EnumVariant {
                    enumeration,
                    variant,
                } => format!(
                    "{}.{}",
                    resolved_name(
                        names,
                        &TargetSymbolRef::Generated(GeneratedSymbolId::Type(*enumeration)),
                    )?,
                    resolved_generated_member_name(names, GeneratedSymbolId::Value(*variant))?
                ),
                JavaValueRef::KnownField(value) => {
                    format!(
                        "{}.{}",
                        resolved_type_name(names, value.owner())?,
                        value.member().text()
                    )
                }
            };
            Ok(value)
        }
        JavaExprKind::Unary { operator, operand } => {
            let operand = render_expr(operand, names, file)?;
            Ok(format!("({}{operand})", unary_operator(*operator)))
        }
        JavaExprKind::Binary {
            operator,
            left,
            right,
        } => {
            let left = render_expr(left, names, file)?;
            let right = render_expr(right, names, file)?;
            Ok(format!("({left} {} {right})", binary_operator(*operator)))
        }
        JavaExprKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            let condition = render_expr(condition, names, file)?;
            let when_true = render_expr(when_true, names, file)?;
            let when_false = render_expr(when_false, names, file)?;
            Ok(format!("({condition} ? {when_true} : {when_false})"))
        }
        JavaExprKind::Call {
            callable,
            receiver,
            arguments,
        } => {
            let target = match callable {
                JavaCallableRef::Known {
                    callable: value, ..
                } => {
                    format!(
                        "{}.{}",
                        resolved_type_name(names, value.owner())?,
                        value.name()
                    )
                }
                JavaCallableRef::Runtime { callable, .. } => {
                    resolved_name(names, &TargetSymbolRef::RuntimeCallable(*callable))?
                }
                JavaCallableRef::Generated { symbol, .. } => resolved_name(
                    names,
                    &TargetSymbolRef::Generated(GeneratedSymbolId::Callable(*symbol)),
                )?,
                JavaCallableRef::Interface { symbol, .. } => {
                    let receiver = receiver.as_deref().ok_or_else(|| {
                        vec![render_diagnostic(
                            file,
                            "interface call is missing its receiver",
                        )]
                    })?;
                    format!(
                        "{}.{}",
                        render_expr(receiver, names, file)?,
                        resolved_name(
                            names,
                            &TargetSymbolRef::Generated(GeneratedSymbolId::InterfaceMethod(
                                *symbol
                            )),
                        )?
                    )
                }
                JavaCallableRef::Member { name, .. } => {
                    let receiver = receiver.as_deref().ok_or_else(|| {
                        vec![render_diagnostic(
                            file,
                            "member call is missing its receiver",
                        )]
                    })?;
                    format!("{}.{}", render_expr(receiver, names, file)?, name.as_str())
                }
            };
            let arguments = arguments
                .iter()
                .map(|value| render_expr(value, names, file))
                .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?
                .join(", ");
            Ok(format!("{target}({arguments})"))
        }
        JavaExprKind::New {
            constructor,
            arguments,
        } => {
            let target = match constructor {
                JavaConstructorRef::Known {
                    constructor, owner, ..
                } => {
                    let _catalogue_name =
                        resolved_name(names, &TargetSymbolRef::KnownConstructor(*constructor))?;
                    render_java_type(owner, names)?
                }
                JavaConstructorRef::Generated { owner, .. } => resolved_name(
                    names,
                    &TargetSymbolRef::Generated(GeneratedSymbolId::Type(*owner)),
                )?,
            };
            let arguments = arguments
                .iter()
                .map(|value| render_expr(value, names, file))
                .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?
                .join(", ");
            Ok(format!("new {target}({arguments})"))
        }
        JavaExprKind::NewArray { component, length } => {
            let component = render_java_type(component, names)?;
            let length = render_expr(length, names, file)?;
            Ok(format!("new {component}[{length}]"))
        }
        JavaExprKind::ArrayIndex { array, index } => {
            let array = render_expr(array, names, file)?;
            let index = render_expr(index, names, file)?;
            Ok(format!("{array}[{index}]"))
        }
        JavaExprKind::Field { receiver, field } => {
            let receiver = render_expr(receiver, names, file)?;
            let field = match field {
                JavaFieldRef::Known(value) => value.member().text(),
                JavaFieldRef::Structural { name, .. } => name.as_str(),
                JavaFieldRef::Generated { name, .. } => name.as_str(),
            };
            Ok(format!("{receiver}.{field}"))
        }
        JavaExprKind::Cast { target, value }
        | JavaExprKind::InterfaceCoercion {
            target,
            value,
            implementation: _,
        } => {
            let target = render_java_type(target, names)?;
            let value = render_expr(value, names, file)?;
            Ok(format!("(({target}) {value})"))
        }
        JavaExprKind::ArrayOwnershipTransition { value, .. } => render_expr(value, names, file),
        JavaExprKind::InstanceOf {
            value,
            target,
            binding,
        } => {
            let value = render_expr(value, names, file)?;
            let target = render_java_type(target, names)?;
            let binding = binding
                .as_ref()
                .map(|value| format!(" {}", value.as_str()))
                .unwrap_or_default();
            Ok(format!("({value} instanceof {target}{binding})"))
        }
        JavaExprKind::Lambda { parameters, body } => {
            let parameters = parameters
                .iter()
                .map(|value| value.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let body = render_block(body, names, 1, file)?;
            Ok(format!("({parameters}) -> {{\n{body} }}"))
        }
    }
}
