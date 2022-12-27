use std::collections::BTreeMap;

use planus_types::intermediate::{DeclarationIndex, Declarations, Type, TypeKind};

type ObjectIndex = usize;
type ByteIndex = usize;

struct TreeState<T> {
    data: T,
    unfolded: bool,
    children: Option<Vec<TreeState<T>>>,
}

struct ViewState<'a> {
    all_objects: Vec<Object<'a>>,
    current_gui_root_object: ObjectIndex,
    byte_mapping: BTreeMap<ByteIndex, Vec<ObjectIndex>>,
    parents: BTreeMap<ObjectIndex, ObjectIndex>,
}

#[derive(Copy, Clone)]
struct ObjectMetadata<'a> {
    declarations: &'a Declarations,
    buffer: &'a [u8],
    offset: usize,
}

#[derive(Copy, Clone)]
struct Object<'a> {
    metadata: ObjectMetadata<'a>,
    type_: &'a Type,
}

#[derive(Copy, Clone)]
struct TableObject<'a> {
    metadata: ObjectMetadata<'a>,
    declaration_index: DeclarationIndex,
}

impl<'a> Object<'a> {
    pub fn new(declarations: &'a Declarations, buffer: &'a [u8], root_type_index: usize) -> Self {
        todo!()
    }

    pub fn as_table(self) -> Option<TableObject<'a>> {
        if let TypeKind::Table(declaration_index) = self.type_.kind {
            Some(TableObject {
                metadata: self.metadata,
                declaration_index,
            })
        } else {
            None
        }
    }
}

impl<'a> TableObject<'a> {
    pub fn get_field(&self, name: &str) -> Result<Option<Object<'a>> {
        let declaration = &self.metadata.declarations.declarations[self.declaration_index];
    }
}
