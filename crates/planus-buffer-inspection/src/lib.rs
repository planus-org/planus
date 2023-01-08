use planus_types::{
    ast::{FloatType, IntegerType},
    intermediate::{DeclarationIndex, Declarations, Type, TypeKind},
};

use crate::object_info::{DeclarationInfo, ObjectName};

pub mod children;
pub mod object_info;
pub mod object_mapping;

pub type ByteIndex = usize;

// TODO
#[derive(Debug)]
pub struct Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct InspectableFlatbuffer<'a> {
    pub declarations: &'a Declarations,
    pub buffer: &'a [u8],
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Object<'a> {
    /// 4 bytes of offset inside a table
    Offset(OffsetObject<'a>),

    /// 0 actual bytes, all bytes are sub-objects of u16 types
    VTable(VTableObject),

    /// 4 bytes of offset to a vtable. Fields are their own objects
    Table(TableObject),

    /// 0 bytes, actual bytes are in child objects
    Struct(StructObject),

    /// 1 byte of union tag
    UnionTag(UnionTagObject),

    /// 4 bytes of offset to the inner object
    Union(UnionObject),

    /// 1, 2, 4 or 8 bytes of enum tag
    Enum(EnumObject),

    /// 4 bytes length field. The actual elements are their own values
    Vector(VectorObject<'a>),

    /// 0 bytes, actual elements are their own values
    Array(ArrayObject<'a>),

    /// 1, 2, 4 or 8 bytes of integer data
    Integer(IntegerObject),

    /// 4 or 8 bytes of float data
    Float(FloatObject),

    /// 1 byte of bool data
    Bool(BoolObject),

    /// 4+n bytes of length and strings
    String(StringObject),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct OffsetObject<'a> {
    pub offset: ByteIndex,
    pub kind: OffsetObjectKind<'a>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum OffsetObjectKind<'a> {
    Table(DeclarationIndex),
    Vector(&'a Type),
    String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct VTableObject {
    pub declaration: DeclarationIndex,
    pub offset: ByteIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TableOffsetObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TableObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StructObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UnionTagObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UnionObject {
    pub tag: u8,
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EnumObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct VectorObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArrayObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
    pub size: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct IntegerObject {
    pub offset: ByteIndex,
    pub type_: IntegerType,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FloatObject {
    pub offset: ByteIndex,
    pub type_: FloatType,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoolObject {
    pub offset: ByteIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringObject {
    pub offset: ByteIndex,
}

impl<'a> OffsetObject<'a> {
    pub fn get_inner(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<Object<'a>> {
        let slice: &[u8; 4] = buffer.buffer[self.offset..self.offset + 4]
            .try_into()
            .unwrap();
        let offset = self.offset + u32::from_le_bytes(*slice) as usize;
        match self.kind {
            OffsetObjectKind::Table(declaration) => Ok(Object::Table(TableObject {
                offset,
                declaration,
            })),
            OffsetObjectKind::Vector(type_) => Ok(Object::Vector(VectorObject { offset, type_ })),
            OffsetObjectKind::String => Ok(Object::String(StringObject { offset })),
        }
    }
}

impl VTableObject {
    pub fn get_vtable_size(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u16> {
        let slice: &[u8; 2] = buffer.buffer[self.offset..self.offset + 2]
            .try_into()
            .unwrap();
        Ok(u16::from_le_bytes(*slice))
    }

    pub fn get_table_size(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u16> {
        let slice: &[u8; 2] = buffer.buffer[self.offset + 2..self.offset + 4]
            .try_into()
            .unwrap();
        Ok(u16::from_le_bytes(*slice))
    }

    pub fn get_offset(&self, i: usize, buffer: &InspectableFlatbuffer<'_>) -> Result<Option<u16>> {
        let slice: &[u8; 2] = buffer.buffer[self.offset + 4 + 2 * i..self.offset + 4 + 2 * i + 2]
            .try_into()
            .unwrap();
        let value = u16::from_le_bytes(*slice);
        let value = (value != 0).then_some(value);
        Ok(value)
    }

    pub fn get_offsets<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<impl 'a + ExactSizeIterator + DoubleEndedIterator + Iterator<Item = u16>> {
        let size = self.get_vtable_size(buffer)?;
        let slice: &[u8] = &buffer.buffer[self.offset + 4..self.offset + size as usize];
        Ok(slice.chunks_exact(2).map(move |chunk| {
            let slice: &[u8; 2] = chunk.try_into().unwrap();
            u16::from_le_bytes(*slice)
        }))
    }
}

impl TableObject {
    pub fn get_vtable(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<VTableObject> {
        let slice: &[u8; 4] = buffer.buffer[self.offset..self.offset + 4]
            .try_into()
            .unwrap();
        let offset = i32::from_le_bytes(*slice);
        let vtable_offset = self.offset.checked_add_signed(-offset as isize).unwrap();

        Ok(VTableObject {
            declaration: self.declaration,
            offset: vtable_offset,
        })
    }

    pub fn get_field<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        field_index: usize,
    ) -> Result<Option<Object<'a>>> {
        let decl = self.resolve_declaration(buffer);
        let Some(offset) = self.get_vtable(buffer)?.get_offset(field_index, buffer)?
        else {
            return Ok(None)
        };

        let offset = self.offset + offset as usize;
        let (_field_name, field_decl, is_union_tag) =
            decl.get_field_for_vtable_index(field_index as u32).unwrap();
        let object = match field_decl.type_.kind {
            TypeKind::Table(declaration) => Object::Offset(OffsetObject {
                offset,
                kind: OffsetObjectKind::Table(declaration),
            }),
            TypeKind::Union(declaration) if is_union_tag => Object::UnionTag(UnionTagObject {
                offset,
                declaration,
            }),
            TypeKind::Union(_declaration) => todo!(),
            TypeKind::Vector(ref _type_) => todo!(),
            TypeKind::Array(ref _type_, _size) => todo!(),
            TypeKind::SimpleType(_type_) => todo!(),
            TypeKind::String => todo!(),
        };
        Ok(Some(object))
    }
}

impl StructObject {
    pub fn get_field<'a>(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _field_index: usize,
    ) -> Result<Option<Object<'a>>> {
        todo!()
    }
}

impl UnionObject {
    pub fn get_variant<'a>(&self, _buffer: &InspectableFlatbuffer<'a>) -> Result<Object<'a>> {
        todo!()
    }
}

impl EnumObject {
    pub fn tag<'a>(&self, _buffer: &InspectableFlatbuffer<'a>) -> Result<IntegerObject> {
        todo!()
    }
}
