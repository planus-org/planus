use planus_types::intermediate::{
    AbsolutePath, DeclarationIndex, DeclarationKind, Enum, SimpleType, Struct, Table, TypeKind,
    Union,
};

use crate::{
    ArrayObject, BoolObject, EnumObject, FloatObject, InspectableFlatbuffer, IntegerObject, Object,
    OffsetObject, StringObject, StructObject, TableObject, UnionObject, UnionTagObject,
    UnionVectorTagsObject, UnionVectorValuesObject, VTableObject, VectorObject,
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
            fn print_object(&self, buffer: &InspectableFlatbuffer<'_>) -> String {
                format!("{}", self.resolve_path(buffer).0.last().unwrap())
            }
        }
    };
}

pub trait ObjectName<'a> {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String;
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

impl DeclarationInfo for VTableObject {
    type Declaration = Table;
    const KIND: &'static str = "vtable";

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

impl<'a> ObjectName<'a> for VTableObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let path = buffer.declarations.get_declaration(self.declaration).0;
        let path = path.0.last().unwrap();
        format!("vtable {path}")
    }
}

impl<'a> ObjectName<'a> for OffsetObject<'a> {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        self.follow_offset(buffer).map_or_else(
            |_| "invalid offset".to_string(),
            |inner| inner.print_object(buffer),
        )
    }
}

impl<'a> ObjectName<'a> for VectorObject<'a> {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let len = if let Ok(len) = self.len(buffer) {
            len.to_string()
        } else {
            "?".to_string()
        };

        if let TypeKind::Table(declaration_index)
        | TypeKind::Union(declaration_index)
        | TypeKind::SimpleType(SimpleType::Enum(declaration_index))
        | TypeKind::SimpleType(SimpleType::Struct(declaration_index)) = self.type_.kind
        {
            let (path, _) = buffer.declarations.get_declaration(declaration_index);
            let path = path.0.last().unwrap();
            format!("[{path}; {len}]",)
        } else {
            format!("[{:?}; {len}]", self.type_.kind)
        }
    }
}

impl<'a> ObjectName<'a> for UnionVectorTagsObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let len = if let Ok(len) = self.len(buffer) {
            len.to_string()
        } else {
            "?".to_string()
        };

        let (path, _) = buffer.declarations.get_declaration(self.declaration);
        let path = path.0.last().unwrap();
        format!("[{path}; {len}]",)
    }
}

impl<'a> ObjectName<'a> for UnionVectorValuesObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let len = if let Ok(len) = self.len(buffer) {
            len.to_string()
        } else {
            "?".to_string()
        };

        let (path, _) = buffer.declarations.get_declaration(self.declaration);
        let path = path.0.last().unwrap();
        format!("[{path}; {len}]",)
    }
}

impl<'a> ObjectName<'a> for ArrayObject<'a> {
    fn print_object(&self, _buffer: &InspectableFlatbuffer<'a>) -> String {
        "ARRAY".to_string() // TODO
    }
}

impl<'a> ObjectName<'a> for IntegerObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("{}", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for FloatObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("{}", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for BoolObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("{}", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for StringObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "{:?} (len={})",
            String::from_utf8_lossy(self.bytes(buffer).unwrap()),
            self.len(buffer).unwrap()
        )
    }
}

impl<'a> ObjectName<'a> for EnumObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let tag = self.tag(buffer).unwrap();
        let (path, decl) = buffer
            .declarations
            .get_declaration(self.declaration_index());
        if let DeclarationKind::Enum(e) = &decl.kind {
            let tag = tag.read(buffer).unwrap();
            let path = path.0.last().unwrap();
            if let Some(variant) = e.variants.get(&tag) {
                format!("{path}::{}", variant.name)
            } else {
                format!("{path}::UnknownTag({tag})")
            }
        } else {
            panic!("Inconsistent declarations");
        }
    }
}

impl_object_name!(StructObject);
impl_object_name!(TableObject);

impl<'a> ObjectName<'a> for UnionTagObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let path = buffer.declarations.get_declaration(self.declaration).0;
        let path = path.0.last().unwrap();
        match self.tag_variant(buffer) {
            Ok(Some((variant_name, _variant))) => {
                format!("{path}::{variant_name}")
            }
            Ok(None) => {
                format!(
                    "{path}::UnknownVariant({})",
                    self.tag_value(buffer).unwrap()
                )
            }
            Err(_) => {
                format!("{path}::ERROR")
            }
        }
    }
}

impl<'a> ObjectName<'a> for UnionObject {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        let path = buffer.declarations.get_declaration(self.declaration).0;
        let path = path.0.last().unwrap();
        match self.tag_variant(buffer) {
            Ok(Some((variant_name, _variant))) => {
                format!("{path}::{variant_name}")
            }
            Ok(None) => {
                format!("{path}::UnknownVariant({})", self.tag)
            }
            Err(_) => {
                format!("{path}::ERROR")
            }
        }
    }
}

impl<'a> ObjectName<'a> for Object<'a> {
    fn print_object(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        match self {
            Object::Offset(obj) => obj.print_object(buffer),
            Object::VTable(obj) => obj.print_object(buffer),
            Object::Table(obj) => obj.print_object(buffer),
            Object::Struct(obj) => obj.print_object(buffer),
            Object::UnionTag(obj) => obj.print_object(buffer),
            Object::Union(obj) => obj.print_object(buffer),
            Object::Enum(obj) => obj.print_object(buffer),
            Object::Vector(obj) => obj.print_object(buffer),
            Object::Array(obj) => obj.print_object(buffer),
            Object::Integer(obj) => obj.print_object(buffer),
            Object::Float(obj) => obj.print_object(buffer),
            Object::Bool(obj) => obj.print_object(buffer),
            Object::String(obj) => obj.print_object(buffer),
            Object::UnionVectorTags(obj) => obj.print_object(buffer),
            Object::UnionVectorValues(obj) => obj.print_object(buffer),
        }
    }
}
