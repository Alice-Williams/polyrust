//! Per-class aggregation and implicit javac members.

use super::{ClassBudget, Code, Helper, expressions::expression, statements::block, type_pool};
use crate::ast::{
    JavaDeclarationKind, JavaHeritage, JavaMember, JavaModifier, JavaTypeDeclaration,
};

pub(crate) fn report(
    value: &JavaTypeDeclaration,
    members: &[&JavaMember],
    type_count: usize,
) -> Vec<ClassBudget> {
    let mut classes = Vec::new();
    let mut helper = Helper::default();
    declaration(value, members, type_count, None, &mut helper, &mut classes);
    if helper.switches != 0 {
        let mut class = baseline(
            format!("{}$enum-switch-helper", value.name.as_str()),
            type_count,
        );
        class.fields = helper.switches;
        let code = Code {
            bytes: helper
                .switches
                .saturating_mul(128)
                .saturating_add(helper.arms.saturating_mul(64)),
            pool: helper
                .switches
                .saturating_mul(32)
                .saturating_add(helper.arms.saturating_mul(16)),
            locals: 8,
            stack: 16,
            exceptions: helper.arms,
            ..Code::default()
        };
        class.method("<clinit>", code, 0);
        // javac may emit a synthetic default constructor on the helper.
        class.method("<init>", Code::default(), 1);
        classes.push(class);
    }
    classes
}

fn baseline(name: String, type_count: usize) -> ClassBudget {
    ClassBudget {
        name,
        pool: 512usize.saturating_add(type_count.saturating_mul(16)),
        fields: 0,
        methods: Vec::new(),
        metadata: type_count.saturating_add(128),
    }
}

fn declaration(
    value: &JavaTypeDeclaration,
    members: &[&JavaMember],
    type_count: usize,
    enclosing: Option<&str>,
    helper: &mut Helper,
    classes: &mut Vec<ClassBudget>,
) {
    let name = enclosing.map_or_else(
        || value.name.as_str().to_owned(),
        |owner| format!("{owner}${}", value.name.as_str()),
    );
    let mut class = baseline(name, type_count);
    class.pool = class
        .pool
        .saturating_add(value.type_parameters.len().saturating_mul(8));
    if let JavaHeritage::Interfaces(types) = &value.heritage {
        for ty in types {
            class.pool = class.pool.saturating_add(type_pool(ty));
        }
    }
    for ty in &value.permits {
        class.pool = class.pool.saturating_add(type_pool(ty));
    }
    let mut instance = Code::default();
    let mut statics = Code::default();
    let mut constants = 0usize;
    let inner = enclosing.is_some()
        && value.kind == JavaDeclarationKind::FinalClass
        && !value.modifiers.contains(&JavaModifier::Static);
    if inner {
        class.fields = class.fields.saturating_add(1);
        class.pool = class.pool.saturating_add(32);
        instance.bytes = instance.bytes.saturating_add(16);
    }
    // Gather initializers once; their executable contribution is copied into
    // every constructor below, but static initialization stays in one method.
    for member in members {
        match member {
            JavaMember::Field(field) => {
                class.fields = class.fields.saturating_add(1);
                class.pool = class
                    .pool
                    .saturating_add(32)
                    .saturating_add(type_pool(&field.ty));
                if let Some(value) = &field.initializer {
                    let mut code = expression(value);
                    code.bytes = code.bytes.saturating_add(16);
                    if field.modifiers.contains(&JavaModifier::Static) {
                        statics.add(&code);
                    } else {
                        instance.add(&code);
                    }
                }
            }
            JavaMember::CompileFailField(field) => {
                // Certification permits these only in explicitly negative-test
                // artifacts. Account their shape without claiming compilation.
                class.fields = class.fields.saturating_add(1);
                class.pool = class
                    .pool
                    .saturating_add(32)
                    .saturating_add(type_pool(&field.expected_type));
                let mut code = expression(&field.initializer);
                code.bytes = code.bytes.saturating_add(16);
                if field.modifiers.contains(&JavaModifier::Static) {
                    statics.add(&code);
                } else {
                    instance.add(&code);
                }
            }
            JavaMember::EnumConstant(_) => {
                constants = constants.saturating_add(1);
                class.fields = class.fields.saturating_add(1);
                class.pool = class.pool.saturating_add(32);
            }
            JavaMember::Method(_) | JavaMember::Constructor(_) | JavaMember::NestedType(_) => {}
        }
    }
    for member in members {
        match member {
            JavaMember::Method(method) => {
                let mut code = method.body.as_ref().map_or_else(Code::default, block);
                code.ty(&method.return_type);
                code.pool = code
                    .pool
                    .saturating_add(method.type_parameters.len().saturating_mul(8));
                let mut slots = 1usize;
                for parameter in &method.parameters {
                    code.ty(&parameter.ty);
                    code.pool = code.pool.saturating_add(8);
                    slots = slots.saturating_add(super::super::types::slots(&parameter.ty));
                }
                helper.add(&code.helper);
                class.method(method.name.as_str(), code, slots);
            }
            JavaMember::Constructor(constructor) => {
                let mut code = block(&constructor.body);
                code.add(&instance);
                let mut slots = 4usize;
                for parameter in &constructor.parameters {
                    code.ty(&parameter.ty);
                    code.pool = code.pool.saturating_add(8);
                    slots = slots.saturating_add(super::super::types::slots(&parameter.ty));
                }
                helper.add(&code.helper);
                class.method("<init>", code, slots);
            }
            JavaMember::NestedType(child) => declaration(
                child,
                &child.members.iter().collect::<Vec<_>>(),
                type_count,
                Some(&class.name),
                helper,
                classes,
            ),
            JavaMember::Field(_)
            | JavaMember::CompileFailField(_)
            | JavaMember::EnumConstant(_) => {}
        }
    }
    match value.kind {
        JavaDeclarationKind::FinalClass => {
            helper.add(&instance.helper);
            class.method("<init>", instance, 4); // Default constructor reservation, even if explicit.
        }
        JavaDeclarationKind::Record => {
            let mut canonical = instance;
            let mut slots = 1usize;
            for component in &value.record_components {
                class.fields = class.fields.saturating_add(1);
                class.pool = class
                    .pool
                    .saturating_add(32)
                    .saturating_add(type_pool(&component.ty));
                canonical.bytes = canonical.bytes.saturating_add(16);
                canonical.pool = canonical.pool.saturating_add(16);
                slots = slots.saturating_add(super::super::types::slots(&component.ty));
                class.method(component.name.as_str(), Code::default(), 1); // Accessor.
            }
            helper.add(&canonical.helper);
            class.method("<init>", canonical, slots);
            for name in ["equals", "hashCode", "toString"] {
                class.method(
                    name,
                    Code {
                        pool: 64,
                        bootstraps: 1,
                        bootstrap_arguments: value.record_components.len().saturating_add(2),
                        ..Code::default()
                    },
                    2,
                );
            }
        }
        JavaDeclarationKind::Enum | JavaDeclarationKind::UninhabitedEnum(_) => {
            class.fields = class.fields.saturating_add(1); // $VALUES
            class.pool = class.pool.saturating_add(32);
            class.method("<init>", instance, 3);
            class.method("values", Code::default(), 0);
            class.method("valueOf", Code::default(), 1);
            class.method(
                "$values",
                Code {
                    bytes: constants.saturating_mul(16),
                    pool: constants.saturating_mul(8),
                    ..Code::default()
                },
                0,
            ); // $values()
            statics.bytes = statics.bytes.saturating_add(constants.saturating_mul(32));
            statics.pool = statics.pool.saturating_add(constants.saturating_mul(16));
            statics.stack = statics.stack.saturating_add(8);
        }
        JavaDeclarationKind::Interface | JavaDeclarationKind::SealedInterface => {}
    }
    helper.add(&statics.helper);
    class.method("<clinit>", statics, 0); // Conservatively reserved even if absent.
    classes.push(class);
}
