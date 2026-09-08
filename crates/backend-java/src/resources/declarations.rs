//! Exact declaration descriptors and parameter-slot capacity.

use super::{Names, limit, types};
use crate::ast::{
    JavaDeclarationKind, JavaHeritage, JavaIdentifier, JavaMember, JavaModifier, JavaParameter,
    JavaPrimitive, JavaType, JavaTypeDeclaration,
};
use portable_diagnostics::Diagnostic;

pub(super) struct Checker<'a> {
    names: &'a Names,
    path: &'a str,
    errors: &'a mut Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
enum ReceiverLayout {
    Static,
    Instance,
    Inner(usize),
    Enum,
}

impl ReceiverLayout {
    fn slots(self) -> usize {
        match self {
            Self::Static => 0,
            Self::Instance => 1,
            Self::Inner(_) => 2,
            Self::Enum => 3,
        }
    }
    fn descriptor(self) -> usize {
        match self {
            Self::Static | Self::Instance => 0,
            Self::Inner(owner) => owner.saturating_add(2),
            Self::Enum => "Ljava/lang/String;I".len(),
        }
    }
}

impl<'a> Checker<'a> {
    pub fn new(names: &'a Names, path: &'a str, errors: &'a mut Vec<Diagnostic>) -> Self {
        Self {
            names,
            path,
            errors,
        }
    }

    pub fn declaration(
        &mut self,
        value: &JavaTypeDeclaration,
        members: &[&JavaMember],
        prefix: usize,
        enclosing: Option<usize>,
    ) {
        self.name(&value.name);
        let binary_name = value
            .declared
            .and_then(|id| self.names.get(&id).copied())
            .unwrap_or_else(|| prefix.saturating_add(value.name.as_str().len()));
        limit(
            self.errors,
            self.path,
            "binary class name bytes",
            binary_name,
            types::MAX_UTF8,
        );
        let generic = types::parameters(&value.type_parameters, self.path, self.errors);
        let mut signature = generic.saturating_add(match value.kind {
            JavaDeclarationKind::Enum | JavaDeclarationKind::UninhabitedEnum(_) => {
                "Ljava/lang/Enum<>;"
                    .len()
                    .saturating_add(binary_name)
                    .saturating_add(2)
            }
            JavaDeclarationKind::FinalClass
            | JavaDeclarationKind::Interface
            | JavaDeclarationKind::SealedInterface => "Ljava/lang/Object;".len(),
            JavaDeclarationKind::Record => "Ljava/lang/Record;".len(),
        });
        if let JavaHeritage::Interfaces(interfaces) = &value.heritage {
            limit(
                self.errors,
                self.path,
                "direct interface count",
                interfaces.len(),
                65_535,
            );
            for ty in interfaces {
                signature = signature.saturating_add(self.ty(ty).signature);
            }
        }
        limit(
            self.errors,
            self.path,
            "class generic signature bytes",
            signature,
            types::MAX_UTF8,
        );
        limit(
            self.errors,
            self.path,
            "permitted subclass count",
            value.permits.len(),
            65_535,
        );
        for ty in &value.permits {
            self.ty(ty);
        }
        if value.kind == JavaDeclarationKind::Record {
            let mut slots = 1usize;
            let mut descriptor = 3usize; // '(' + ')' + void return
            let mut signature = 3usize;
            let mut recipe = value.record_components.len().saturating_sub(1);
            for component in &value.record_components {
                recipe = recipe.saturating_add(component.name.as_str().len());
                self.name(&component.name);
                let ty = self.ty(&component.ty);
                slots = slots.saturating_add(types::slots(&component.ty));
                descriptor = descriptor.saturating_add(ty.descriptor);
                signature = signature.saturating_add(ty.signature);
                limit(
                    self.errors,
                    self.path,
                    "record accessor descriptor bytes",
                    ty.descriptor.saturating_add(2),
                    types::MAX_UTF8,
                );
            }
            self.callable_limits(slots, descriptor, signature);
            limit(
                self.errors,
                self.path,
                "record component-name recipe bytes",
                recipe,
                types::MAX_UTF8,
            );
        }
        for member in members {
            match member {
                JavaMember::Field(field) => {
                    self.name(&field.name);
                    self.ty(&field.ty);
                    if let Some(value) = &field.initializer {
                        self.expression(value);
                    }
                }
                JavaMember::CompileFailField(field) => {
                    self.name(&field.name);
                    self.ty(&field.expected_type);
                    self.expression(&field.initializer);
                }
                JavaMember::EnumConstant(constant) => self.name(&constant.name),
                JavaMember::Method(method) => {
                    if let Some(body) = &method.body {
                        self.block(body);
                    }
                    self.name(&method.name);
                    let implicit = if method.modifiers.contains(&JavaModifier::Static) {
                        ReceiverLayout::Static
                    } else {
                        ReceiverLayout::Instance
                    };
                    self.callable(
                        &method.parameters,
                        &method.return_type,
                        &method.type_parameters,
                        implicit,
                    );
                }
                JavaMember::Constructor(constructor) => {
                    self.block(&constructor.body);
                    self.name(&constructor.name);
                    let implicit = match value.kind {
                        JavaDeclarationKind::Enum | JavaDeclarationKind::UninhabitedEnum(_) => {
                            ReceiverLayout::Enum
                        }
                        JavaDeclarationKind::FinalClass
                        | JavaDeclarationKind::Record
                        | JavaDeclarationKind::Interface
                        | JavaDeclarationKind::SealedInterface => {
                            if value.kind == JavaDeclarationKind::FinalClass
                                && !value.modifiers.contains(&JavaModifier::Static)
                            {
                                enclosing.map_or(ReceiverLayout::Instance, ReceiverLayout::Inner)
                            } else {
                                ReceiverLayout::Instance
                            }
                        }
                    };
                    self.callable(
                        &constructor.parameters,
                        &JavaType::Primitive(JavaPrimitive::Void),
                        &[],
                        implicit,
                    );
                }
                JavaMember::NestedType(child) => self.declaration(
                    child,
                    &child.members.iter().collect::<Vec<_>>(),
                    binary_name.saturating_add(1),
                    Some(binary_name),
                ),
            }
        }
    }

    fn callable(
        &mut self,
        parameters: &[JavaParameter],
        result: &JavaType,
        generics: &[JavaIdentifier],
        implicit: ReceiverLayout,
    ) {
        let result = self.ty(result);
        let mut descriptor = result
            .descriptor
            .saturating_add(2)
            .saturating_add(implicit.descriptor());
        let mut signature = result
            .signature
            .saturating_add(2)
            .saturating_add(types::parameters(generics, self.path, self.errors));
        let mut slots = implicit.slots();
        for parameter in parameters {
            self.name(&parameter.name);
            let ty = self.ty(&parameter.ty);
            descriptor = descriptor.saturating_add(ty.descriptor);
            signature = signature.saturating_add(ty.signature);
            slots = slots.saturating_add(types::slots(&parameter.ty));
        }
        self.callable_limits(slots, descriptor, signature);
    }

    fn callable_limits(&mut self, slots: usize, descriptor: usize, signature: usize) {
        limit(self.errors, self.path, "method parameter slots", slots, 255);
        limit(
            self.errors,
            self.path,
            "method descriptor bytes",
            descriptor,
            types::MAX_UTF8,
        );
        limit(
            self.errors,
            self.path,
            "method generic signature bytes",
            signature,
            types::MAX_UTF8,
        );
    }

    pub(super) fn name(&mut self, name: &JavaIdentifier) {
        limit(
            self.errors,
            self.path,
            "identifier bytes",
            name.as_str().len(),
            types::MAX_UTF8,
        );
    }

    pub(super) fn ty(&mut self, ty: &JavaType) -> types::Encoding {
        types::encoding(ty, self.names, self.path, self.errors)
    }
}
