use planus_types::{
    ast::{FloatType, IntegerType},
    intermediate::{
        AbsolutePath, Declaration, DeclarationIndex, DeclarationKind, Declarations, SimpleType,
        Table, Type, TypeKind,
    },
};

pub type ByteIndex = usize;

pub struct Error;
type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone)]
pub struct InspectableFlatbuffer<'a> {
    pub declarations: &'a Declarations,
    pub buffer: &'a [u8],
}

#[derive(Copy, Clone)]
pub struct Object<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
}

#[derive(Copy, Clone)]
pub enum ObjectEnum<'a> {
    VTable(VTableObject),
    Table(TableObject),
    Struct(StructObject),
    Union(UnionObject),
    Enum(EnumObject),
    Vector(VectorObject<'a>),
    Array(ArrayObject<'a>),
    Integer(IntegerObject),
    Float(FloatObject),
    Bool(BoolObject),
    String(StringObject),
}

#[derive(Copy, Clone)]
pub struct VTableObject {
    pub offset: ByteIndex,
}

#[derive(Copy, Clone)]
pub struct TableObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone)]
pub struct StructObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone)]
pub struct UnionObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone)]
pub struct EnumObject {
    pub offset: ByteIndex,
    pub declaration: DeclarationIndex,
}

#[derive(Copy, Clone)]
pub struct VectorObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
}

#[derive(Copy, Clone)]
pub struct ArrayObject<'a> {
    pub offset: ByteIndex,
    pub type_: &'a Type,
    pub size: u32,
}

#[derive(Copy, Clone)]
pub struct IntegerObject {
    pub offset: ByteIndex,
    pub type_: IntegerType,
}

#[derive(Copy, Clone)]
pub struct FloatObject {
    pub offset: ByteIndex,
    pub type_: FloatType,
}

#[derive(Copy, Clone)]
pub struct BoolObject {
    pub offset: ByteIndex,
}

#[derive(Copy, Clone)]
pub struct StringObject {
    pub offset: ByteIndex,
}

impl<'a> From<Object<'a>> for ObjectEnum<'a> {
    fn from(object: Object<'a>) -> Self {
        let offset = object.offset;
        match object.type_.kind {
            TypeKind::Table(declaration) => ObjectEnum::Table(TableObject {
                offset,
                declaration,
            }),
            TypeKind::Union(declaration) => ObjectEnum::Union(UnionObject {
                offset,
                declaration,
            }),
            TypeKind::Vector(ref type_) => ObjectEnum::Vector(VectorObject {
                offset,
                type_: &type_,
            }),
            TypeKind::Array(ref type_, size) => ObjectEnum::Array(ArrayObject {
                offset,
                type_,
                size: size,
            }),
            TypeKind::SimpleType(SimpleType::Struct(declaration)) => {
                ObjectEnum::Struct(StructObject {
                    offset,
                    declaration,
                })
            }
            TypeKind::SimpleType(SimpleType::Enum(declaration)) => ObjectEnum::Enum(EnumObject {
                offset,
                declaration,
            }),
            TypeKind::SimpleType(SimpleType::Bool) => ObjectEnum::Bool(BoolObject { offset }),
            TypeKind::SimpleType(SimpleType::Integer(type_)) => {
                ObjectEnum::Integer(IntegerObject { offset, type_ })
            }
            TypeKind::SimpleType(SimpleType::Float(type_)) => {
                ObjectEnum::Float(FloatObject { offset, type_ })
            }
            TypeKind::String => ObjectEnum::String(StringObject { offset }),
        }
    }
}

impl VTableObject {
    pub fn get_vtable_size(buffer: &InspectableFlatbuffer<'_>) -> Result<u16> {
        todo!()
    }

    pub fn get_table_size(buffer: &InspectableFlatbuffer<'_>) -> Result<u16> {
        todo!()
    }

    pub fn get_offsets<'a>(
        buffer: &InspectableFlatbuffer<'a>,
    ) -> Result<impl ExactSizeIterator + DoubleEndedIterator + Iterator<Item = u16>> {
        todo!();
        Ok(std::iter::empty())
    }
}

impl TableObject {
    pub fn resolve_decl<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Table {
        if let DeclarationKind::Table(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }

    pub fn resolve_name<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a AbsolutePath {
        buffer.declarations.get_declaration(self.declaration).0
    }

    pub fn get_vtable(&self, buffer: &InspectableFlatbuffer<'_>) -> VTableObject {
        todo!()
    }

    pub fn get_field<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        field_index: usize,
    ) -> Result<Option<Object<'a>>> {
        todo!()
    }

    pub fn get_fields<'a>(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
    ) -> impl Iterator<Item = (usize, Result<Object<'a>>)> {
        todo!();
        std::iter::empty()
    }
}

trait ObjectInfo<'a> {
    type Iterator: Iterator<Item = (String, Object<'a>)>;

    fn get_representation(&self, buffer: &InspectableFlatbuffer<'_>) -> String;
    fn get_children(&self, buffer: &InspectableFlatbuffer<'_>) -> String;
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> std::ops::Range<usize>;
}
