use planus_types::intermediate::{DeclarationIndex, Declarations, Type, TypeKind};

pub type ObjectIndex = usize;
pub type ByteIndex = usize;

pub struct Error;

pub struct InspectableBuffer<'a> {
    pub declarations: &'a Declarations,
    pub buffer: &'a [u8],
}

#[derive(Copy, Clone)]
pub struct Object<'a> {
    pub offset: usize,
    pub type_: &'a Type,
}

#[derive(Copy, Clone)]
pub struct TableObject {
    pub offset: usize,
    pub declaration_index: DeclarationIndex,
}

impl<'a> InspectableBuffer<'a> {
    pub fn get_root_object(&self, root_type_index: DeclarationIndex) -> Result<Object<'a>, Error> {
        todo!()
    }
}

impl<'a> Object<'a> {
    pub fn as_table(self) -> Option<TableObject> {
        if let TypeKind::Table(declaration_index) = self.type_.kind {
            Some(TableObject {
                offset: self.offset,
                declaration_index,
            })
        } else {
            None
        }
    }
}
