use std::borrow::Cow;

use planus_types::{
    ast::{FloatType, IntegerType},
    intermediate::{
        DeclarationIndex, DeclarationKind, Declarations, FloatLiteral, IntegerLiteral, SimpleType,
        StructField, Type, TypeKind, UnionVariant,
    },
};

use crate::object_info::DeclarationInfo;

pub mod children;
pub mod object_info;
pub mod object_mapping;

pub type ByteIndex = u32;

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
    pub fn read_u8(&self, offset: ByteIndex) -> Result<u8> {
        let offset = offset as usize;
        Ok(self.buffer[offset])
    }

    pub fn read_u16(&self, offset: ByteIndex) -> Result<u16> {
        let offset = offset as usize;
        let slice: &[u8; 2] = self.buffer[offset..offset + 2].try_into().unwrap();
        Ok(u16::from_le_bytes(*slice))
    }

    pub fn read_u32(&self, offset: ByteIndex) -> Result<u32> {
        let offset = offset as usize;
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(u32::from_le_bytes(*slice))
    }

    pub fn read_u64(&self, offset: ByteIndex) -> Result<u64> {
        let offset = offset as usize;
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(u64::from_le_bytes(*slice))
    }

    pub fn read_i8(&self, offset: ByteIndex) -> Result<i8> {
        let offset = offset as usize;
        Ok(self.buffer[offset] as i8)
    }

    pub fn read_i16(&self, offset: ByteIndex) -> Result<i16> {
        let offset = offset as usize;
        let slice: &[u8; 2] = self.buffer[offset..offset + 2].try_into().unwrap();
        Ok(i16::from_le_bytes(*slice))
    }

    pub fn read_i32(&self, offset: ByteIndex) -> Result<i32> {
        let offset = offset as usize;
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(i32::from_le_bytes(*slice))
    }

    pub fn read_i64(&self, offset: ByteIndex) -> Result<i64> {
        let offset = offset as usize;
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(i64::from_le_bytes(*slice))
    }

    pub fn read_f32(&self, offset: ByteIndex) -> Result<f32> {
        let offset = offset as usize;
        let slice: &[u8; 4] = self.buffer[offset..offset + 4].try_into().unwrap();
        Ok(f32::from_le_bytes(*slice))
    }

    pub fn read_f64(&self, offset: ByteIndex) -> Result<f64> {
        let offset = offset as usize;
        let slice: &[u8; 8] = self.buffer[offset..offset + 8].try_into().unwrap();
        Ok(f64::from_le_bytes(*slice))
    }

    pub fn read_slice(&self, offset: ByteIndex, size: usize) -> Result<&'a [u8]> {
        let offset = offset as usize;
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

impl<'a> Object<'a> {
    pub fn offset(&self) -> ByteIndex {
        match self {
            Object::Offset(inner) => inner.offset,
            Object::VTable(inner) => inner.offset,
            Object::Table(inner) => inner.offset,
            Object::Struct(inner) => inner.offset,
            Object::UnionTag(inner) => inner.offset,
            Object::Union(inner) => inner.offset,
            Object::Enum(inner) => inner.offset,
            Object::Vector(inner) => inner.offset,
            Object::Array(inner) => inner.offset,
            Object::Integer(inner) => inner.offset,
            Object::Float(inner) => inner.offset,
            Object::Bool(inner) => inner.offset,
            Object::String(inner) => inner.offset,
        }
    }

    pub fn have_braces(&self) -> bool {
        match self {
            Object::Offset(_) => false,
            Object::VTable(_) => true,
            Object::Table(_) => true,
            Object::Struct(_) => true,
            Object::UnionTag(_) => false,
            Object::Union(_) => false,
            Object::Enum(_) => false,
            Object::Vector(_) => true,
            Object::Array(_) => true,
            Object::Integer(_) => false,
            Object::Float(_) => false,
            Object::Bool(_) => false,
            Object::String(_) => false,
        }
    }

    pub fn type_name(&self, declarations: &Declarations) -> Cow<'static, str> {
        match self {
            Object::Offset(inner) => inner.type_name(declarations),
            Object::VTable(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Table(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Struct(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::UnionTag(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Union(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Enum(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Vector(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Array(inner) => Cow::Owned(inner.type_name(declarations)),
            Object::Integer(inner) => Cow::Borrowed(inner.type_.flatbuffer_name()),
            Object::Float(inner) => Cow::Borrowed(inner.type_.flatbuffer_name()),
            Object::Bool(_) => Cow::Borrowed("bool"),
            Object::String(_) => Cow::Borrowed("string"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct OffsetObject<'a> {
    pub offset: ByteIndex,
    pub kind: OffsetObjectKind<'a>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum OffsetObjectKind<'a> {
    VTable(DeclarationIndex),
    Table(DeclarationIndex),
    Vector(&'a Type),
    String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VTableObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
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

impl UnionTagObject {
    fn type_name(&self, declarations: &Declarations) -> String {
        format!(
            "union tag {}",
            declarations.get_declaration(self.declaration).0
        )
    }

    pub fn tag_value(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u8> {
        buffer.read_u8(self.offset)
    }

    pub fn tag_variant<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<Option<(&'a str, &'a UnionVariant)>> {
        let decl = self.resolve_declaration(buffer);
        if let Some((k, v)) = decl
            .variants
            .get_index(self.tag_value(buffer)? as usize - 1)
        {
            Ok(Some((k, v)))
        } else {
            Ok(None)
        }
    }
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
impl<'a> ArrayObject<'a> {
    pub fn type_name(&self, declarations: &Declarations) -> String {
        format!(
            "[{}; {}]",
            declarations.format_type_kind(&self.type_.kind),
            self.size
        )
    }
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
    pub fn get_byte_index(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<ByteIndex> {
        let res = if matches!(self.kind, OffsetObjectKind::VTable(_)) {
            self.offset
                .checked_add_signed(-buffer.read_i32(self.offset)?)
                .ok_or(Error)?
        } else {
            self.offset + buffer.read_u32(self.offset)?
        };
        Ok(res)
    }

    pub fn follow_offset(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<Object<'a>> {
        let offset = self.get_byte_index(buffer)?;
        match self.kind {
            OffsetObjectKind::VTable(declaration) => Ok(Object::VTable(VTableObject {
                declaration,
                offset,
            })),
            OffsetObjectKind::Table(declaration) => Ok(Object::Table(TableObject {
                offset,
                declaration,
            })),
            OffsetObjectKind::Vector(type_) => Ok(Object::Vector(VectorObject { offset, type_ })),
            OffsetObjectKind::String => Ok(Object::String(StringObject { offset })),
        }
    }

    fn type_name(&self, declarations: &Declarations) -> Cow<'static, str> {
        match self.kind {
            OffsetObjectKind::VTable(index) => {
                Cow::Owned(format!("&vtable {}", declarations.get_declaration(index).0))
            }
            OffsetObjectKind::Table(index) => {
                Cow::Owned(format!("&table {}", declarations.get_declaration(index).0))
            }
            OffsetObjectKind::Vector(type_) => {
                Cow::Owned(format!("&[{}]", declarations.format_type_kind(&type_.kind)))
            }
            OffsetObjectKind::String => Cow::Borrowed("&string"),
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

    pub fn get_offset(&self, i: u32, buffer: &InspectableFlatbuffer<'_>) -> Result<Option<u16>> {
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

    fn type_name(&self, declarations: &Declarations) -> String {
        format!(
            "vtable for {}",
            declarations.get_declaration(self.declaration).0
        )
    }
}

impl TableObject {
    pub fn get_vtable(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<VTableObject> {
        let vtable_offset = self
            .offset
            .checked_add_signed(-buffer.read_i32(self.offset)?)
            .unwrap();

        Ok(VTableObject {
            declaration: self.declaration,
            offset: vtable_offset,
        })
    }

    pub fn get_field<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        field_index: u32,
    ) -> Result<Option<Object<'a>>> {
        let decl = self.resolve_declaration(buffer);
        let Some(object_offset) = self.get_vtable(buffer)?.get_offset(field_index, buffer)? else {
            return Ok(None);
        };

        let offset = self.offset + object_offset as u32;
        let (_field_name, field_decl, is_union_tag) =
            decl.get_field_for_vtable_index(field_index).unwrap();
        let object = match field_decl.type_.kind {
            TypeKind::Table(declaration) => Object::Offset(OffsetObject {
                offset,
                kind: OffsetObjectKind::Table(declaration),
            }),
            TypeKind::Union(declaration) if is_union_tag => Object::UnionTag(UnionTagObject {
                offset,
                declaration,
            }),
            TypeKind::Union(declaration) => {
                let Some(tag_offset) = self
                    .get_vtable(buffer)?
                    .get_offset(field_index - 1, buffer)?
                else {
                    return Ok(None);
                };

                let tag_offset = self.offset + tag_offset as u32;
                if let Ok(tag) = buffer.read_u8(tag_offset) {
                    Object::Union(UnionObject {
                        tag,
                        offset,
                        declaration,
                    })
                } else {
                    return Ok(None);
                }
            }
            TypeKind::Vector(ref type_) => Object::Offset(OffsetObject {
                offset,
                kind: OffsetObjectKind::Vector(type_),
            }),
            TypeKind::Array(ref _type_, _size) => todo!(),
            TypeKind::SimpleType(type_) => match type_ {
                SimpleType::Struct(declaration) => Object::Struct(StructObject {
                    offset,
                    declaration,
                }),
                SimpleType::Enum(declaration) => Object::Enum(EnumObject {
                    offset,
                    declaration,
                }),
                SimpleType::Bool => Object::Bool(BoolObject { offset }),
                SimpleType::Integer(type_) => Object::Integer(IntegerObject { offset, type_ }),
                SimpleType::Float(type_) => Object::Float(FloatObject { offset, type_ }),
            },
            TypeKind::String => Object::Offset(OffsetObject {
                offset,
                kind: OffsetObjectKind::String,
            }),
        };
        Ok(Some(object))
    }

    fn type_name(&self, declarations: &Declarations) -> String {
        format!("table {}", declarations.get_declaration(self.declaration).0)
    }
}

impl StructObject {
    pub fn get_field_info<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        field_index: usize,
    ) -> Option<(&'a str, &'a StructField)> {
        let decl = self.resolve_declaration(buffer);
        let (field_name, field) = decl.fields.get_index(field_index)?;
        Some((field_name.as_str(), field))
    }

    pub fn get_field<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        field_index: usize,
    ) -> Result<Object<'a>> {
        let Some((_field_name, field)) = self.get_field_info(buffer, field_index) else {
            return Err(Error);
        };

        let offset = self.offset + field.offset;
        let object = match field.type_ {
            SimpleType::Struct(declaration) => Object::Struct(StructObject {
                offset,
                declaration,
            }),
            SimpleType::Enum(declaration) => Object::Enum(EnumObject {
                offset,
                declaration,
            }),
            SimpleType::Bool => Object::Bool(BoolObject { offset }),
            SimpleType::Integer(type_) => Object::Integer(IntegerObject { offset, type_ }),
            SimpleType::Float(type_) => Object::Float(FloatObject { offset, type_ }),
        };
        Ok(object)
    }

    fn type_name(&self, declarations: &Declarations) -> String {
        format!(
            "struct {}",
            declarations.get_declaration(self.declaration).0
        )
    }
}

impl UnionObject {
    pub fn inner_offset<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<Option<OffsetObject<'a>>> {
        if let Some((_name, variant)) = self.tag_variant(buffer)? {
            Ok(Some(OffsetObject {
                offset: self.offset,
                kind: match &variant.type_.kind {
                    TypeKind::Table(index) => OffsetObjectKind::Table(*index),
                    TypeKind::Vector(type_) => OffsetObjectKind::Vector(type_),
                    TypeKind::String => OffsetObjectKind::String,
                    TypeKind::Array(_, _) | TypeKind::SimpleType(_) | TypeKind::Union(_) => {
                        panic!("Inconsistent declarations")
                    }
                },
            }))
        } else {
            Ok(None)
        }
    }

    fn type_name(&self, declarations: &Declarations) -> String {
        format!(
            "&union {}",
            declarations.get_declaration(self.declaration).0
        )
    }

    pub fn tag_variant<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<Option<(&'a str, &'a UnionVariant)>> {
        let decl = self.resolve_declaration(buffer);
        if let Some((k, v)) = decl.variants.get_index(self.tag as usize - 1) {
            Ok(Some((k, v)))
        } else {
            Ok(None)
        }
    }
}

impl EnumObject {
    pub fn tag(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<IntegerObject> {
        let (_, decl) = buffer.declarations.get_declaration(self.declaration);

        if let DeclarationKind::Enum(e) = &decl.kind {
            Ok(IntegerObject {
                offset: self.offset,
                type_: e.type_,
            })
        } else {
            Err(Error)
        }
    }

    pub fn read(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u64> {
        let tag = self.tag(buffer)?;
        let val = tag.read(buffer)?;
        Ok(val.to_u64())
    }

    fn type_name(&self, declarations: &Declarations) -> String {
        format!("enum {}", declarations.get_declaration(self.declaration).0)
    }
}

impl StringObject {
    pub fn len(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u32> {
        buffer.read_u32(self.offset)
    }

    pub fn bytes<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> Result<&'a [u8]> {
        let offset = self.offset as usize;
        Ok(&buffer.buffer[offset + 4..offset + 4 + self.len(buffer)? as usize])
    }
}

impl BoolObject {
    pub fn read(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<bool> {
        Ok(buffer.buffer[self.offset as usize] != 0)
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

impl<'a> VectorObject<'a> {
    pub fn len(&self, buffer: &InspectableFlatbuffer<'_>) -> Result<u32> {
        buffer.read_u32(self.offset)
    }

    pub fn read(
        &self,
        index: u32,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<Option<Object<'a>>> {
        if index >= self.len(buffer)? {
            return Ok(None);
        }

        let offset = self.offset + 4;
        let object = match &self.type_.kind {
            TypeKind::Union(_) => return Ok(None),
            TypeKind::Array(_, _) => return Ok(None),
            TypeKind::Table(declaration_index) => Object::Offset(OffsetObject {
                offset: offset + index * 4,
                kind: OffsetObjectKind::Table(*declaration_index),
            }),
            TypeKind::Vector(type_) => Object::Offset(OffsetObject {
                offset: offset + index * 4,
                kind: OffsetObjectKind::Vector(type_),
            }),

            TypeKind::String => Object::Offset(OffsetObject {
                offset: offset + index * 4,
                kind: OffsetObjectKind::String,
            }),

            TypeKind::SimpleType(type_) => match type_ {
                SimpleType::Struct(declaration_index) => {
                    if let DeclarationKind::Struct(decl) = &buffer
                        .declarations
                        .get_declaration(*declaration_index)
                        .1
                        .kind
                    {
                        Object::Struct(StructObject {
                            offset: offset + index * decl.size,
                            declaration: *declaration_index,
                        })
                    } else {
                        panic!("Inconsistent declarations");
                    }
                }
                SimpleType::Enum(declaration_index) => {
                    if let DeclarationKind::Enum(decl) = &buffer
                        .declarations
                        .get_declaration(*declaration_index)
                        .1
                        .kind
                    {
                        Object::Enum(EnumObject {
                            offset: offset + index * decl.type_.byte_size(),
                            declaration: *declaration_index,
                        })
                    } else {
                        panic!("Inconsistent declarations");
                    }
                }
                SimpleType::Bool => Object::Bool(BoolObject {
                    offset: offset + index,
                }),
                SimpleType::Integer(type_) => Object::Integer(IntegerObject {
                    offset: offset + index * type_.byte_size(),
                    type_: *type_,
                }),
                SimpleType::Float(type_) => Object::Float(FloatObject {
                    offset: offset + index * type_.byte_size(),
                    type_: *type_,
                }),
            },
        };
        Ok(Some(object))
    }

    fn type_name(&self, declarations: &Declarations) -> String {
        format!("[{}]", declarations.format_type_kind(&self.type_.kind))
    }
}
