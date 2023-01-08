use planus_types::{
    ast::{FloatType, IntegerType},
    intermediate::{DeclarationIndex, Declarations, FloatLiteral, IntegerLiteral, Type, TypeKind},
};

use crate::object_info::DeclarationInfo;

pub mod allocations;
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

impl<'a> InspectableFlatbuffer<'a> {
    pub fn read_u8(&self, offset: usize) -> Result<u8> {
        Ok(self.buffer[offset])
    }

    pub fn read_u16(&self, offset: usize) -> Result<u16> {
        let slice: &[u8; 2] = self.buffer[offset..offset + 2].try_into().unwrap();
        Ok(u16::from_le_bytes(*slice))
    }

    pub fn read_u32(&self, offset: usize) -> Result<u32> {
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(u32::from_le_bytes(*slice))
    }

    pub fn read_u64(&self, offset: usize) -> Result<u64> {
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(u64::from_le_bytes(*slice))
    }

    pub fn read_i8(&self, offset: usize) -> Result<i8> {
        Ok(self.buffer[offset] as i8)
    }

    pub fn read_i16(&self, offset: usize) -> Result<i16> {
        let slice: &[u8; 2] = self.buffer[offset..offset + 2].try_into().unwrap();
        Ok(i16::from_le_bytes(*slice))
    }

    pub fn read_i32(&self, offset: usize) -> Result<i32> {
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(i32::from_le_bytes(*slice))
    }

    pub fn read_i64(&self, offset: usize) -> Result<i64> {
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(i64::from_le_bytes(*slice))
    }

    pub fn read_f32(&self, offset: usize) -> Result<f32> {
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(f32::from_le_bytes(*slice))
    }

    pub fn read_f64(&self, offset: usize) -> Result<f64> {
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(f64::from_le_bytes(*slice))
    }

    pub fn read_slice(&self, offset: usize, size: usize) -> Result<&'a [u8]> {
        Ok(&self.buffer[offset..offset + size])
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct OffsetObject<'a> {
    pub offset: ByteIndex,
    pub kind: OffsetObjectKind<'a>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum OffsetObjectKind<'a> {
    VTable,
    Table(DeclarationIndex),
    Vector(&'a Type),
    String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VTableObject {
    pub offset: ByteIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TableOffsetObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TableObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StructObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnionTagObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnionObject {
    pub tag: u8,
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct EnumObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VectorObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ArrayObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
    pub size: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct IntegerObject {
    pub offset: ByteIndex,
    pub type_: IntegerType,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FloatObject {
    pub offset: ByteIndex,
    pub type_: FloatType,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct BoolObject {
    pub offset: ByteIndex,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StringObject {
    pub offset: ByteIndex,
}

impl<'a> OffsetObject<'a> {
    pub fn get_inner(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<Object<'a>> {
        let offset = self.offset + buffer.read_u32(self.offset)? as usize;
        match self.kind {
            OffsetObjectKind::VTable => {
                let offset = self
                    .offset
                    .checked_add_signed(-buffer.read_i32(self.offset)? as isize)
                    .unwrap();
                Ok(Object::VTable(VTableObject { offset }))
            }
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
        buffer.read_u16(self.offset)
    }

    pub fn get_table_size(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u16> {
        buffer.read_u16(self.offset + 2)
    }

    pub fn get_offset(&self, i: usize, buffer: &InspectableFlatbuffer<'_>) -> Result<Option<u16>> {
        let value = buffer.read_u16(self.offset + 4 + 2 * i)?;
        let value = (value != 0).then_some(value);
        Ok(value)
    }

    pub fn get_offsets<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<impl 'a + ExactSizeIterator + DoubleEndedIterator + Iterator<Item = u16>> {
        let size = self.get_vtable_size(buffer)?;
        let slice: &[u8] = buffer.read_slice(self.offset, size as usize)?;
        Ok(slice[4..].chunks_exact(2).map(move |chunk| {
            let slice: &[u8; 2] = chunk.try_into().unwrap();
            u16::from_le_bytes(*slice)
        }))
    }
}

impl TableObject {
    pub fn get_vtable(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<VTableObject> {
        let vtable_offset = self
            .offset
            .checked_add_signed(-buffer.read_i32(self.offset)? as isize)
            .unwrap();

        Ok(VTableObject {
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

impl StringObject {
    pub fn len(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u32> {
        buffer.read_u32(self.offset)
    }

    pub fn bytes<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<&'a [u8]> {
        Ok(&buffer.buffer[self.offset..self.offset + self.len(buffer)? as usize])
    }
}

impl BoolObject {
    pub fn read(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<bool> {
        Ok(buffer.buffer[self.offset] != 0)
    }
}

impl FloatObject {
    pub fn read(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<FloatLiteral> {
        match self.type_ {
            FloatType::F32 => Ok(FloatLiteral::F32(buffer.read_f32(self.offset)?)),
            FloatType::F64 => Ok(FloatLiteral::F64(buffer.read_f64(self.offset)?)),
        }
    }
}

impl IntegerObject {
    pub fn read(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<IntegerLiteral> {
        let literal = match self.type_ {
            IntegerType::U8 => IntegerLiteral::U8(buffer.read_u8(self.offset)?),
            IntegerType::U16 => IntegerLiteral::U16(buffer.read_u16(self.offset)?),
            IntegerType::U32 => IntegerLiteral::U32(buffer.read_u32(self.offset)?),
            IntegerType::U64 => IntegerLiteral::U64(buffer.read_u64(self.offset)?),
            IntegerType::I8 => IntegerLiteral::I8(buffer.read_i8(self.offset)?),
            IntegerType::I16 => IntegerLiteral::I16(buffer.read_i16(self.offset)?),
            IntegerType::I32 => IntegerLiteral::I32(buffer.read_i32(self.offset)?),
            IntegerType::I64 => IntegerLiteral::I64(buffer.read_i64(self.offset)?),
        };
        Ok(literal)
    }
}
