use std::borrow::Cow;

use planus_types::{ast::IntegerType, intermediate::TypeKind};

use crate::{
    object_info::DeclarationInfo, ArrayObject, BoolObject, ByteIndex, EnumObject, FloatObject,
    InspectableFlatbuffer, IntegerObject, Object, OffsetObject, StringObject, StructObject,
    TableObject, UnionObject, UnionTagObject, VTableObject, VectorObject,
};

pub trait Children<'a> {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    );
}

pub trait Byterange {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex);
}

impl<'a> Children<'a> for Object<'a> {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        match self {
            Object::Offset(_) => (),
            Object::VTable(obj) => obj.children(buffer, callback),
            Object::Table(obj) => obj.children(buffer, callback),
            Object::Struct(obj) => obj.children(buffer, callback),
            Object::UnionTag(obj) => obj.children(buffer, callback),
            Object::Union(obj) => obj.children(buffer, callback),
            Object::Enum(obj) => obj.children(buffer, callback),
            Object::Vector(obj) => obj.children(buffer, callback),
            Object::Array(obj) => obj.children(buffer, callback),
            Object::Integer(obj) => obj.children(buffer, callback),
            Object::Float(obj) => obj.children(buffer, callback),
            Object::Bool(obj) => obj.children(buffer, callback),
            Object::String(obj) => obj.children(buffer, callback),
        }
    }
}

impl<'a> Children<'a> for VTableObject {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        let vtable_size = self.get_vtable_size(buffer).unwrap();
        let decl = self.resolve_declaration(buffer);

        for (i, offset) in (self.offset..self.offset + vtable_size as u32)
            .step_by(2)
            .enumerate()
        {
            let object = Object::Integer(IntegerObject {
                offset: offset,
                type_: IntegerType::U16,
            });
            match i {
                0 => callback(Some(Cow::Borrowed("#vtable_size")), object),
                1 => callback(Some(Cow::Borrowed("#table_size")), object),
                n => {
                    let n = n - 2;
                    if let Some((k, v)) = decl.fields.get_index(n) {
                        if matches!(v.type_.kind, TypeKind::Union(_)) {
                            callback(Some(Cow::Owned(format!("offset[union_tag[{k}]]"))), object)
                        } else {
                            callback(Some(Cow::Owned(format!("offset[{k}]"))), object)
                        }
                    } else if let Some((k, _v)) = decl
                        .fields
                        .get_index(n - 1)
                        .filter(|(_k, v)| matches!(v.type_.kind, TypeKind::Union(_)))
                    {
                        callback(Some(Cow::Owned(format!("offset[union[{k}]]"))), object)
                    } else {
                        callback(Some(Cow::Owned(format!("offset[{n}]"))), object)
                    }
                }
            }
        }
    }
}

impl<'a> Children<'a> for TableObject {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        callback(
            Some(Cow::Borrowed("#vtable")),
            Object::Offset(OffsetObject {
                offset: self.offset,
                kind: crate::OffsetObjectKind::VTable(self.declaration),
            }),
        );

        let vtable = self.get_vtable(buffer).unwrap();
        let decl = self.resolve_declaration(buffer);
        for (i, offset) in vtable.get_offsets(buffer).unwrap().enumerate() {
            if offset == 0 {
                continue;
            }
            let (field_name, _field_decl, is_union_tag) =
                decl.get_field_for_vtable_index(i as u32).unwrap();
            let field_name = if is_union_tag {
                Cow::Owned(format!("union_key[{}]", field_name))
            } else {
                Cow::Borrowed(field_name)
            };
            if let Some(field_value) = self.get_field(&buffer, i as u32).unwrap() {
                callback(Some(field_name), field_value);
            }
        }
    }
}

impl<'a> Children<'a> for StructObject {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        let this = *self;
        let buffer = *buffer;
        let decl = self.resolve_declaration(&buffer);
        for (i, field_name) in decl.fields.keys().enumerate() {
            if let Ok(field) = this.get_field(&buffer, i) {
                callback(Some(Cow::Borrowed(field_name.as_str())), field);
            }
        }
    }
}

impl<'a> Children<'a> for UnionTagObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for UnionObject {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        match self.inner_offset(buffer) {
            Ok(Some(inner_offset)) => callback(None, Object::Offset(inner_offset)),
            Ok(None) => callback(
                Some(Cow::Borrowed("unknown offset")),
                Object::Integer(IntegerObject {
                    offset: self.offset,
                    type_: IntegerType::U32,
                }),
            ),
            Err(_) => callback(
                Some(Cow::Borrowed("error offset")),
                Object::Integer(IntegerObject {
                    offset: self.offset,
                    type_: IntegerType::U32,
                }),
            ),
        }
    }
}

impl<'a> Children<'a> for EnumObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for VectorObject<'a> {
    fn children(
        &self,
        buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        callback(
            Some(Cow::Borrowed("length")),
            Object::Integer(IntegerObject {
                offset: self.offset,
                type_: IntegerType::U32,
            }),
        );
        for i in 0..self.len(&buffer).unwrap_or(0) {
            if let Ok(Some(value)) = self.read(i, buffer) {
                callback(Some(Cow::Owned(i.to_string())), value);
            }
        }
    }
}

impl<'a> Children<'a> for ArrayObject<'a> {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for IntegerObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for FloatObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for BoolObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        _callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
    }
}

impl<'a> Children<'a> for StringObject {
    fn children(
        &self,
        _buffer: &InspectableFlatbuffer<'a>,
        mut callback: impl FnMut(Option<Cow<'a, str>>, Object<'a>),
    ) {
        callback(
            Some(Cow::Borrowed("length")),
            Object::Integer(IntegerObject {
                offset: self.offset,
                type_: IntegerType::U32,
            }),
        );
    }
}

impl<'a> Byterange for Object<'a> {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
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
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + 4)
    }
}

impl Byterange for VTableObject {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        let size = self.get_vtable_size(buffer).unwrap();
        (self.offset, self.offset + size as ByteIndex)
    }
}

impl Byterange for TableObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + 4)
    }
}

impl Byterange for StructObject {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        let decl = self.resolve_declaration(buffer);
        (self.offset, self.offset + decl.size)
    }
}

impl Byterange for UnionTagObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + 1)
    }
}

impl Byterange for UnionObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + 4)
    }
}

impl Byterange for EnumObject {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        let decl = self.resolve_declaration(buffer);
        (self.offset, self.offset + decl.type_.byte_size())
    }
}

impl<'a> Byterange for VectorObject<'a> {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset)
    }
}

impl<'a> Byterange for ArrayObject<'a> {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset)
    }
}

impl Byterange for IntegerObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + self.type_.byte_size())
    }
}

impl Byterange for FloatObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + self.type_.byte_size())
    }
}

impl Byterange for BoolObject {
    fn byterange(&self, _buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (self.offset, self.offset + 1)
    }
}

impl Byterange for StringObject {
    fn byterange(&self, buffer: &InspectableFlatbuffer<'_>) -> (ByteIndex, ByteIndex) {
        (
            self.offset,
            self.offset + 4 + self.len(buffer).unwrap() as ByteIndex,
        )
    }
}
