use super::{BTreeSet, Checker, Declaration, NodeMeta, TypePosition, TypeRef};

impl Checker<'_> {
    pub(super) fn type_contains_interface(&mut self, ty: &TypeRef, node: &NodeMeta) -> bool {
        let mut pending = vec![(ty.clone(), node.clone())];
        let mut visited = BTreeSet::new();
        while let Some((ty, node)) = pending.pop() {
            // Alias normalization keeps its own cycle diagnostic. Aggregate
            // recursion is valid here and must not restart an unbounded walk.
            let Some(ty) = self.normalize_type(&ty, &node, TypePosition::General, &mut Vec::new())
            else {
                continue;
            };
            match ty {
                TypeRef::Interface(_) => return true,
                TypeRef::List(inner) | TypeRef::Option(inner) => pending.push((*inner, node)),
                TypeRef::Result { ok, error } => {
                    pending.push((*ok, node.clone()));
                    pending.push((*error, node));
                }
                TypeRef::Named(id) if visited.insert(id) => match self.index.declaration(id) {
                    Some(Declaration::Record(record)) => {
                        pending.extend(
                            record
                                .fields
                                .iter()
                                .map(|field| (field.ty.clone(), field.header.node.clone())),
                        );
                    }
                    Some(Declaration::Enum(enumeration)) => {
                        pending.extend(
                            enumeration
                                .variants
                                .iter()
                                .flat_map(|variant| &variant.fields)
                                .map(|field| (field.ty.clone(), field.header.node.clone())),
                        );
                    }
                    _ => {}
                },
                TypeRef::Named(_)
                | TypeRef::Unit
                | TypeRef::Bool
                | TypeRef::I32
                | TypeRef::I64
                | TypeRef::F64
                | TypeRef::Char
                | TypeRef::String
                | TypeRef::Bytes => {}
            }
        }
        false
    }
}
