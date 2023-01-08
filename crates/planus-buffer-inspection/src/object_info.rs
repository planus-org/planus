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

pub trait ObjectName<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String;
}

impl<'a, T: DeclarationInfo> ObjectName<'a> for T {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'_>) -> String {
        format!("{}[{}]", Self::KIND, self.resolve_path(buffer))
    }
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
            "vtable[{}]",
            buffer.declarations.get_declaration(self.declaration).0
        )
    }
}

impl<'a> ObjectName<'a> for OffsetObject<'a> {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        match self.kind {
            crate::OffsetObjectKind::VTable(declaration) => format!(
                "offset to vtable[{}]",
                buffer.declarations.get_declaration(declaration).0
            ),
            crate::OffsetObjectKind::Table(declaration) => {
                format!(
                    "offset to table[{}]",
                    buffer.declarations.get_declaration(declaration).0
                )
            }
            crate::OffsetObjectKind::Vector(_) => format!("offset to vector"),
            crate::OffsetObjectKind::String => format!("offset to string"),
        }
    }
}

impl<'a> ObjectName<'a> for VectorObject<'a> {
    fn resolve_name(&self, _buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("VECTOR") // TODO
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
            "{} = {}",
            self.type_.flatbuffer_name(),
            self.read(buffer).unwrap()
        )
    }
}

impl<'a> ObjectName<'a> for FloatObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("float = {}", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for BoolObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!("bool = {}", self.read(buffer).unwrap())
    }
}

impl<'a> ObjectName<'a> for StringObject {
    fn resolve_name(&self, buffer: &InspectableFlatbuffer<'a>) -> String {
        format!(
            "string[{}] = {}",
            self.len(buffer).unwrap(),
            String::from_utf8_lossy(self.bytes(buffer).unwrap())
        )
    }
}

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
