//! Structural rendering: statements.

use super::*;

pub(super) fn render_block(
    value: &JavaBlock,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let mut statements = String::new();
    for statement in &value.statements {
        statements.push_str(&render_stmt(statement, names, depth, file)?);
    }
    Ok(statements)
}

pub(super) fn render_stmt(
    value: &JavaStmt,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
    depth: usize,
    file: &LinkedFile<JavaDialect>,
) -> Result<String, Vec<Diagnostic>> {
    let indentation = indent(depth);
    match value {
        JavaStmt::Local {
            finality,
            ty,
            name,
            value,
        } => {
            let ty = render_java_type(ty, names)?;
            let initializer = match value {
                Some(value) => format!(" = {}", render_expr(value, names, file)?),
                None => String::new(),
            };
            let finality = if *finality == JavaLocalFinality::Final {
                "final "
            } else {
                ""
            };
            Ok(format!(
                "{indentation}{finality}{ty} {}{initializer};\n",
                name.as_str()
            ))
        }
        JavaStmt::Assign { target, value } => {
            let target = render_expr(target, names, file)?;
            let value = render_expr(value, names, file)?;
            Ok(format!("{indentation}{target} = {value};\n"))
        }
        JavaStmt::Expression(value) => {
            let value = render_expr(value, names, file)?;
            Ok(format!("{indentation}{value};\n"))
        }
        JavaStmt::Return(value) => {
            let value = match value {
                Some(value) => format!(" {}", render_expr(value, names, file)?),
                None => String::new(),
            };
            Ok(format!("{indentation}return{value};\n"))
        }
        JavaStmt::If {
            condition,
            then_block,
            else_block,
        } => {
            let condition = render_expr(condition, names, file)?;
            let then_body = render_block(then_block, names, depth + 1, file)?;
            let else_body = match else_block {
                Some(value) => format!(
                    " else {{\n{}{}}}",
                    render_block(value, names, depth + 1, file)?,
                    indentation
                ),
                None => String::new(),
            };
            Ok(format!(
                "{indentation}if ({condition}) {{\n{then_body}{indentation} }}{else_body}\n"
            ))
        }
        JavaStmt::ForEach {
            binding_type,
            binding,
            iterable,
            body,
        } => {
            let ty = render_java_type(binding_type, names)?;
            let iterable = render_expr(iterable, names, file)?;
            let body = render_block(body, names, depth + 1, file)?;
            Ok(format!(
                "{indentation}for ({ty} {} : {iterable}) {{\n{body}{indentation}}}\n",
                binding.as_str()
            ))
        }
        JavaStmt::While { condition, body } => {
            let condition = render_expr(condition, names, file)?;
            let body = render_block(body, names, depth + 1, file)?;
            Ok(format!(
                "{indentation}while ({condition}) {{\n{body}{indentation}}}\n"
            ))
        }
        JavaStmt::Switch { value, arms } => {
            let value = render_expr(value, names, file)?;
            let mut rendered_arms = String::new();
            for arm in arms {
                let pattern = render_switch_pattern(&arm.pattern, names)?;
                let body = render_block(&arm.body, names, depth + 2, file)?;
                let arm_indent = indent(depth + 1);
                rendered_arms.push_str(&format!(
                    "{arm_indent}{pattern} -> {{\n{body}{arm_indent}}}\n"
                ));
            }
            Ok(format!(
                "{indentation}switch ({value}) {{\n{rendered_arms}{indentation}}}\n"
            ))
        }
        JavaStmt::TryCatch { try_block, catches } => {
            let body = render_block(try_block, names, depth + 1, file)?;
            let mut rendered_catches = String::new();
            for catch in catches {
                let exception_type = render_java_type(&catch.exception_type, names)?;
                let catch_body = render_block(&catch.body, names, depth + 1, file)?;
                rendered_catches.push_str(&format!(
                    " catch ({exception_type} {}) {{\n{catch_body}{indentation} }}",
                    catch.binding.as_str()
                ));
            }
            Ok(format!(
                "{indentation}try {{\n{body}{indentation} }}{rendered_catches}\n"
            ))
        }
        JavaStmt::ThrowAssertion(message) => {
            let message = render_expr(message, names, file)?;
            Ok(format!(
                "{indentation}throw new AssertionError({message});\n"
            ))
        }
        JavaStmt::Throw(value) => {
            let value = render_expr(value, names, file)?;
            Ok(format!("{indentation}throw {value};\n"))
        }
        JavaStmt::Break => Ok(format!("{indentation}break;\n")),
        JavaStmt::Continue => Ok(format!("{indentation}continue;\n")),
    }
}

pub(super) fn render_switch_pattern(
    pattern: &JavaPattern,
    names: &std::collections::BTreeMap<TargetSymbolRef<JavaDialect>, JavaResolvedName>,
) -> Result<String, Vec<Diagnostic>> {
    match pattern {
        JavaPattern::Default => Ok("default".to_owned()),
        JavaPattern::Literal(value) => Ok(format!("case {}", render_literal(value))),
        JavaPattern::EnumVariant { variant, .. } => Ok(format!(
            "case {}",
            resolved_generated_member_name(names, GeneratedSymbolId::Value(*variant))?
        )),
        JavaPattern::Type { ty, binding } => Ok(format!(
            "case {} {}",
            render_java_type(ty, names)?,
            binding.as_str()
        )),
    }
}
