use std::borrow::Cow;

use planus_types::ast::IntegerType;

use crate::{
    object_info::{DeclarationInfo, ObjectName},
    ArrayObject, BoolObject, ByteIndex, EnumObject, FloatObject, InspectableFlatbuffer,
    IntegerObject, Object, OffsetObject, StringObject, StructObject, TableObject, UnionObject,
    UnionTagObject, VTableObject, VectorObject,
};

type ChildPair<'a> = (Cow<'a, str>, Object<'a>);

pub trait Children<'a> {
    type Iter: Iterator<Item = ChildPair<'a>>;

    fn children(&self, buffer: &InspectableFlatbuffer<'a>) -> Self::Iter;
}

pub trait Byterange {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<usize>>;
}

impl<'a> Children<'a> for Object<'a> {
    type Iter = Box<dyn 'a + Iterator<Item = ChildPair<'a>>>;

    fn children(&self, buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        match self {
            Object::Offset(obj) => Box::new(obj.children(buffer)),
            Object::VTable(obj) => Box::new(obj.children(buffer)),
            Object::Table(obj) => Box::new(obj.children(buffer)),
            Object::Struct(obj) => Box::new(obj.children(buffer)),
            Object::UnionTag(obj) => Box::new(obj.children(buffer)),
            Object::Union(obj) => Box::new(obj.children(buffer)),
            Object::Enum(obj) => Box::new(obj.children(buffer)),
            Object::Vector(obj) => Box::new(obj.children(buffer)),
            Object::Array(obj) => Box::new(obj.children(buffer)),
            Object::Integer(obj) => Box::new(obj.children(buffer)),
            Object::Float(obj) => Box::new(obj.children(buffer)),
            Object::Bool(obj) => Box::new(obj.children(buffer)),
            Object::String(obj) => Box::new(obj.children(buffer)),
        }
    }
}

impl<'a> Children<'a> for OffsetObject<'a> {
    type Iter = std::iter::Once<ChildPair<'a>>;

    fn children(&self, buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::once((Cow::Borrowed("inner"), self.get_inner(buffer).unwrap()))
    }
}

impl<'a> Children<'a> for VTableObject {
    type Iter = Box<dyn 'a + Iterator<Item = ChildPair<'a>>>;

    fn children(&self, buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        let vtable_size = self.get_table_size(buffer).unwrap();

        let iter = (self.offset..self.offset + vtable_size as usize)
            .step_by(2)
            .enumerate()
            .map(|(i, offset)| {
                let object = Object::Integer(IntegerObject {
                    offset: offset,
                    type_: IntegerType::U16,
                });
                match i {
                    0 => (Cow::Borrowed("vtable_size"), object),
                    1 => (Cow::Borrowed("table_size"), object),
                    n => (Cow::Owned(format!("{n}")), object),
                }
            });
        Box::new(iter)
    }
}

impl<'a> Children<'a> for TableObject {
    type Iter = Box<dyn 'a + Iterator<Item = ChildPair<'a>>>;

    fn children(&self, buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        let this = *self;
        let buffer = *buffer;
        let vtable = self.get_vtable(&buffer).unwrap();
        let decl = self.resolve_declaration(&buffer);
        let field_iter = vtable
            .get_offsets(&buffer)
            .unwrap()
            .enumerate()
            .filter(|&(_i, offset)| offset != 0)
            .filter_map(move |(i, _offset)| {
                let (field_name, _field_decl, is_union_tag) =
                    decl.get_field_for_vtable_index(i as u32).unwrap();
                let field_name = if is_union_tag {
                    Cow::Owned(format!("union_key[{}]", field_name))
                } else {
                    Cow::Borrowed(field_name)
                };
                let field_value = this.get_field(&buffer, i).unwrap()?;
                Some((field_name, field_value))
            });

        let iter =
            std::iter::once((Cow::Borrowed("vtable"), Object::VTable(vtable))).chain(field_iter);

        Box::new(iter)
    }
}

impl<'a> Children<'a> for StructObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for UnionTagObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for UnionObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for EnumObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for VectorObject<'a> {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for ArrayObject<'a> {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for IntegerObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for FloatObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for BoolObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Children<'a> for StringObject {
    type Iter = std::iter::Empty<ChildPair<'a>>;

    fn children(&self, _buffer: &InspectableFlatbuffer<'a>) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a> Byterange for Object<'a> {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<usize>> {
        match self {
            Object::Offset(obj) => obj.byterange(buffer),
            Object::VTable(obj) => obj.byterange(buffer),
            Object::Table(obj) => obj.byterange(buffer),
            Object::Struct(obj) => obj.byterange(buffer),
            Object::UnionTag(obj) => obj.byterange(buffer),
            Object::Union(obj) => obj.byterange(buffer),
            Object::Enum(obj) => obj.byterange(buffer),
            Object::Vector(obj) => obj.byterange(buffer),
            Object::Array(obj) => obj.byterange(buffer),
            Object::Integer(obj) => obj.byterange(buffer),
            Object::Float(obj) => obj.byterange(buffer),
            Object::Bool(obj) => obj.byterange(buffer),
            Object::String(obj) => obj.byterange(buffer),
        }
    }
}

impl<'a> Byterange for OffsetObject<'a> {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        Some(self.offset..self.offset + 4)
    }
}

impl Byterange for VTableObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for TableObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        Some(self.offset..self.offset + 4)
    }
}

impl Byterange for StructObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for UnionTagObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for UnionObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for EnumObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl<'a> Byterange for VectorObject<'a> {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl<'a> Byterange for ArrayObject<'a> {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for IntegerObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        Some(self.offset..self.offset + self.type_.byte_size() as usize)
    }
}

impl Byterange for FloatObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for BoolObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}

impl Byterange for StringObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> Option<std::ops::Range<ByteIndex>> {
        None
    }
}
