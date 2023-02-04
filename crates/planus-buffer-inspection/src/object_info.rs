use planus_types::intermediate::{
    AbsolutePath, DeclarationIndex, DeclarationKind, Enum, Struct, Table, Union,
};

use crate::{
    ArrayObject, BoolObject, EnumObject, FloatObject, InspectableFlatbuffer, IntegerObject, Object,
    OffsetObject, StringObject, StructObject, TableObject, UnionObject, UnionTagObject,
    VTableObject, VectorObject,
};

pub trait DeclarationInfo {
    type Declaration;
    const KIND: &'static str;

    fn declaration_index(&self) -> DeclarationIndex;
    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Self::Declaration;
    fn resolve_path<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a AbsolutePath {
        buffer
            .declarations
            .get_declaration(self.declaration_index())
            .0
    }
}

macro_rules! impl_object_name {
    ($typ:ty) => {
        impl<'a> ObjectName<'a> for $typ {
            fn resolve_name(&self, buffer: &InspectableFlatbuffer<'_>) -> String {
                format!("{} {}", Self::KIND, self.resolve_path(buffer))
            }
        }
    };
}

pub trait ObjectName<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String;
}

impl DeclarationInfo for TableObject {
    type Declaration = Table;
    const KIND: &'static str = "table";

    fn declaration_index(&self) -> DeclarationIndex {
        self.declaration
    }

    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Table {
        if let DeclarationKind::Table(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl DeclarationInfo for StructObject {
    type Declaration = Struct;
    const KIND: &'static str = "struct";

    fn declaration_index(&self) -> DeclarationIndex {
        self.declaration
    }

    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Struct {
        if let DeclarationKind::Struct(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl DeclarationInfo for UnionTagObject {
    type Declaration = Union;
    const KIND: &'static str = "union_key";

    fn declaration_index(&self) -> DeclarationIndex {
        self.declaration
    }

    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Union {
        if let DeclarationKind::Union(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl DeclarationInfo for UnionObject {
    type Declaration = Union;
    const KIND: &'static str = "union";

    fn declaration_index(&self) -> DeclarationIndex {
        self.declaration
    }

    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Union {
        if let DeclarationKind::Union(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl DeclarationInfo for EnumObject {
    type Declaration = Enum;
    const KIND: &'static str = "enum";

    fn declaration_index(&self) -> DeclarationIndex {
        self.declaration
    }

    fn resolve_declaration<'a>(&self, buffer: &InspectableFlatbuffer<'a>) -> &'a Enum {
        if let DeclarationKind::Enum(decl) =
            &buffer.declarations.get_declaration(self.declaration).1.kind
        {
            decl
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl<'a> ObjectName<'a> for VTableObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "vtable {}",
            buffer.declarations.get_declaration(self.declaration).0
        )
    }
}

impl<'a> ObjectName<'a> for OffsetObject<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "offset({})",
            self.get_inner(buffer)
                .as_ref()
                .map(|inner| inner.resolve_name(buffer))
                .as_deref()
                .unwrap_or("invalid")
        )
    }
}

impl<'a> ObjectName<'a> for VectorObject<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let len = self.len(buffer).map(|n| n.to_string());
        let len = len.as_deref().unwrap_or("invalid");
        format!("vector({}, {:?})", len, self.type_.kind)
    }
}

impl<'a> ObjectName<'a> for ArrayObject<'a> {
    fn resolve_name(&self, _buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("ARRAY") // TODO
    }
}

impl<'a> ObjectName<'a> for IntegerObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "{}({})",
            self.type_.flatbuffer_name(),
            self.read(buffer).unwrap()
        )
    }
}

impl<'a> ObjectName<'a> for FloatObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "{}({})",
            self.type_.flatbuffer_name(),
            self.read(buffer).unwrap()
        )
    }
}

impl<'a> ObjectName<'a> for BoolObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("bool({})", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for StringObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "string({}, {:?})",
            self.len(buffer).unwrap(),
            String::from_utf8_lossy(self.bytes(buffer).unwrap())
        )
    }
}

impl<'a> ObjectName<'a> for EnumObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let tag = self.tag(buffer).unwrap();
        let (path, decl) = buffer
            .declarations
            .get_declaration(self.declaration_index());
        if let DeclarationKind::Enum(e) = &decl.kind {
            let tag = tag.read(buffer).unwrap();
            if let Some(variant) = e.variants.get(&tag) {
                format!("enum {path}({}, {})", tag, variant.name)
            } else {
                format!("enum {path}({tag}, ?)")
            }
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl_object_name!(StructObject);
impl_object_name!(TableObject);
impl_object_name!(UnionObject);
impl_object_name!(UnionTagObject);

impl<'a> ObjectName<'a> for Object<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        match self {
            Object::Offset(obj) => obj.resolve_name(buffer),
            Object::VTable(obj) => obj.resolve_name(buffer),
            Object::Table(obj) => obj.resolve_name(buffer),
            Object::Struct(obj) => obj.resolve_name(buffer),
            Object::UnionTag(obj) => obj.resolve_name(buffer),
            Object::Union(obj) => obj.resolve_name(buffer),
            Object::Enum(obj) => obj.resolve_name(buffer),
            Object::Vector(obj) => obj.resolve_name(buffer),
            Object::Array(obj) => obj.resolve_name(buffer),
            Object::Integer(obj) => obj.resolve_name(buffer),
            Object::Float(obj) => obj.resolve_name(buffer),
            Object::Bool(obj) => obj.resolve_name(buffer),
            Object::String(obj) => obj.resolve_name(buffer),
        }
    }
}
