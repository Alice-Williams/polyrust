//! Lazy metadata discovery: no vector of all fields or sibling declarations.
use crate::ast::{
    JavaFileItem, JavaMember, JavaRecordComponent, JavaRecordComponentOrigin, JavaTypeDeclaration,
};
use crate::dialect::JavaDialect;
use portable_codegen::{RustSourceOrigin, TargetAstPackage};

pub(crate) fn origins(
    package: &TargetAstPackage<JavaDialect>,
) -> impl Iterator<Item = &RustSourceOrigin> {
    package
        .files()
        .flat_map(|file| file.items())
        .flat_map(Fields::new)
}

enum Frame<'a> {
    Components(&'a [JavaRecordComponent]),
    Members(&'a [JavaMember]),
}

struct Fields<'a> {
    // Only ancestors are retained. Width and total field count do not allocate
    // pending work; the shared checker can stop after its first rejected origin.
    pending: Vec<Frame<'a>>,
}

impl<'a> Fields<'a> {
    fn new(item: &'a JavaFileItem) -> Self {
        let mut fields = Self {
            pending: Vec::new(),
        };
        match item {
            JavaFileItem::Type { declaration, .. } => fields.enter(declaration),
            JavaFileItem::RuntimeMembers { members, .. } => {
                fields.pending.push(Frame::Members(members))
            }
        }
        fields
    }

    fn enter(&mut self, declaration: &'a JavaTypeDeclaration) {
        self.pending.push(Frame::Members(&declaration.members));
        self.pending
            .push(Frame::Components(&declaration.record_components));
    }
}

impl<'a> Iterator for Fields<'a> {
    type Item = &'a RustSourceOrigin;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(frame) = self.pending.last_mut() {
            match frame {
                Frame::Components(remaining) => {
                    let Some((component, tail)) = remaining.split_first() else {
                        self.pending.pop();
                        continue;
                    };
                    *remaining = tail;
                    if let JavaRecordComponentOrigin::RustSource(field) = &component.origin {
                        return Some(field.origin.as_ref());
                    }
                }
                Frame::Members(remaining) => {
                    let Some((member, tail)) = remaining.split_first() else {
                        self.pending.pop();
                        continue;
                    };
                    *remaining = tail;
                    if let JavaMember::NestedType(child) = member {
                        self.enter(child);
                    }
                }
            }
        }
        None
    }
}
