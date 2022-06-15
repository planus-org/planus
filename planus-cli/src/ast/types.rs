use std::{cell::Cell, collections::HashMap};

use codespan::{FileId, Span};
use indexmap::IndexMap;

use crate::{ctx::Ctx, error::ErrorKind};

pub type RawIdentifier = string_interner::DefaultSymbol;
pub type Interner = string_interner::StringInterner;

#[derive(Copy, Clone, Debug)]
pub struct Identifier {
    pub span: Span,
    pub value: RawIdentifier,
}

pub struct Schema {
    pub file_id: FileId,

    // the spans are where the include definitions were
    pub native_includes: Vec<StringLiteral>,
    pub includes: Vec<StringLiteral>,

    pub namespace: Option<(Span, NamespacePath)>,
    pub root_type: Option<(Span, Type)>,
    pub file_extension: Option<(Span, StringLiteral)>,
    pub file_identifier: Option<(Span, StringLiteral)>,

    pub attributes: Vec<Attribute>,
    pub type_declarations: IndexMap<RawIdentifier, Declaration>,

    pub errors_seen: Cell<ErrorKind>,
}

impl Schema {
    pub fn new(file_id: FileId) -> Self {
        Self {
            file_id,
            namespace: Default::default(),
            native_includes: Default::default(),
            includes: Default::default(),
            root_type: Default::default(),
            file_extension: Default::default(),
            file_identifier: Default::default(),
            attributes: Default::default(),
            type_declarations: Default::default(),
            errors_seen: Default::default(),
        }
    }
}

pub struct Attribute {
    pub span: Span,
    pub kind: AttributeKind,
}

pub enum AttributeKind {
    // Potentially add more as we begin caring about them
    Identifier(RawIdentifier),
    String(String),
}

#[derive(Clone)]
pub struct Declaration {
    pub file_id: FileId,
    pub full_span: Span,
    pub definition_span: Span,
    pub identifier: Identifier,
    pub kind: TypeDeclarationKind,
}

#[derive(Clone)]
pub enum TypeDeclarationKind {
    Table(Struct),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    RpcService(RpcService),
}

#[derive(Clone, Debug, Default)]
pub struct MetadataMap {
    pub seen: HashMap<MetadataValueKindKey, Span>,
    pub values: Vec<MetadataValue>,
}

#[derive(Clone)]
pub struct Struct {
    pub metadata: MetadataMap,
    pub fields: IndexMap<RawIdentifier, StructField>,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub span: Span,
    pub ident: Identifier,
    pub type_: Type,
    pub assignment: Option<Literal>,
    pub metadata: MetadataMap,
}

#[derive(Clone)]
pub struct Enum {
    pub metadata: MetadataMap,
    pub type_: IntegerType,
    pub type_span: Span,
    pub variants: IndexMap<RawIdentifier, EnumVariant>,
}

#[derive(Clone)]
pub struct EnumVariant {
    pub span: Span,
    pub ident: Identifier,
    pub value: Option<IntegerLiteral>,
}

#[derive(Clone)]
pub struct Union {
    pub metadata: MetadataMap,
    pub variants: IndexMap<UnionKey, UnionVariant>,
}

#[derive(Clone, Debug)]
pub enum UnionKey {
    Identifier(RawIdentifier),
    Type(Type),
}

impl PartialEq for UnionKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UnionKey::Identifier(i0), UnionKey::Identifier(i1)) => i0 == i1,
            (UnionKey::Type(t0), UnionKey::Type(t1)) => t0.eq_unspanned(t1),
            _ => false,
        }
    }
}

impl Eq for UnionKey {}
impl std::hash::Hash for UnionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&std::mem::discriminant(self), state);
        match self {
            UnionKey::Identifier(i) => i.hash(state),
            UnionKey::Type(t) => t.hash_unspanned(state),
        }
    }
}

#[derive(Clone)]
pub struct UnionVariant {
    pub span: Span,
    pub ident: Option<Identifier>,
    pub type_: Type,
}

#[derive(Clone)]
pub struct RpcService {
    pub methods: IndexMap<RawIdentifier, RpcMethod>,
}

#[derive(Clone)]
pub struct RpcMethod {
    pub span: Span,
    pub ident: Identifier,
    pub argument_type: Type,
    pub return_type: Type,
    pub metadata: MetadataMap,
}

#[derive(Clone, Debug)]
pub struct MetadataValue {
    pub span: Span,
    pub kind: MetadataValueKind,
}

#[derive(Clone, Debug)]
pub enum MetadataValueKind {
    ForceAlign(IntegerLiteral),
    BitFlags,
    CsharpPartial,
    Private,
    NativeType(StringLiteral),
    NativeTypePackName(StringLiteral),
    OriginalOrder,

    Required,
    Deprecated,
    Key,
    Shared,
    NestedFlatbuffer(StringLiteral),
    Id(IntegerLiteral),
    Hash(StringLiteral),
    CppType(StringLiteral),
    CppPtrType(StringLiteral),
    CppPtrTypeGet(StringLiteral),
    CppStrType(StringLiteral),
    CppStrFlexCtor,
    NativeInline,
    NativeDefault(StringLiteral),
    Flexbuffer,

    Streaming(StringLiteral),
    Idempotent,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum MetadataValueKindKey {
    ForceAlign,
    BitFlags,
    CsharpPartial,
    Private,
    NativeType,
    NativeTypePackName,
    OriginalOrder,

    Required,
    Deprecated,
    Key,
    Shared,
    NestedFlatbuffer,
    Id,
    Hash,
    CppType,
    CppPtrType,
    CppPtrTypeGet,
    CppStrType,
    CppStrFlexCtor,
    NativeInline,
    NativeDefault,
    Flexbuffer,

    Streaming,
    Idempotent,
}

impl MetadataValueKindKey {
    pub fn parse(s: &str) -> Option<MetadataValueKindKey> {
        match s {
            "force_align" => Some(Self::ForceAlign),
            "bit_flags" => Some(Self::BitFlags),
            "csharp_partial" => Some(Self::CsharpPartial),
            "private" => Some(Self::Private),
            "native_type" => Some(Self::NativeType),
            "native_type_pack_name" => Some(Self::NativeTypePackName),
            "original_order" => Some(Self::OriginalOrder),
            "required" => Some(Self::Required),
            "deprecated" => Some(Self::Deprecated),
            "key" => Some(Self::Key),
            "shared" => Some(Self::Shared),
            "nested_flatbuffer" => Some(Self::NestedFlatbuffer),
            "id" => Some(Self::Id),
            "hash" => Some(Self::Hash),
            "cpp_type" => Some(Self::CppType),
            "cpp_ptr_type" => Some(Self::CppPtrType),
            "cpp_ptr_type_get" => Some(Self::CppPtrTypeGet),
            "cpp_str_type" => Some(Self::CppStrType),
            "cpp_str_flex_ctor" => Some(Self::CppStrFlexCtor),
            "native_inline" => Some(Self::NativeInline),
            "native_default" => Some(Self::NativeDefault),
            "flexbuffer" => Some(Self::Flexbuffer),
            "streaming" => Some(Self::Streaming),
            "idempotent" => Some(Self::Idempotent),
            _ => None,
        }
    }

    pub fn requirement(&self) -> &'static str {
        match self {
            Self::BitFlags
            | Self::CsharpPartial
            | Self::Private
            | Self::OriginalOrder
            | Self::Required
            | Self::Deprecated
            | Self::Key
            | Self::Shared
            | Self::CppStrFlexCtor
            | Self::NativeInline
            | Self::Flexbuffer
            | Self::Idempotent => "should not have an argument",
            Self::ForceAlign | Self::Id => "should have an integer argument",
            Self::NativeType
            | Self::NativeTypePackName
            | Self::NestedFlatbuffer
            | Self::Hash
            | Self::CppType
            | Self::CppPtrType
            | Self::CppPtrTypeGet
            | Self::CppStrType
            | Self::NativeDefault
            | Self::Streaming => "should have a string argument",
        }
    }
}

impl MetadataValueKind {
    // Does the attribute have at least partial support?
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            Self::ForceAlign(_) | Self::Required | Self::Deprecated | Self::Id(_)
        )
    }

    pub fn accepted_on_enums(&self) -> bool {
        matches!(
            self,
            Self::ForceAlign(_) | Self::BitFlags | Self::CsharpPartial | Self::Private,
        )
    }

    pub fn accepted_on_structs(&self) -> bool {
        matches!(
            self,
            Self::ForceAlign(_)
                | Self::CsharpPartial
                | Self::Private
                | Self::NativeType(_)
                | Self::NativeTypePackName(_)
        )
    }

    pub fn accepted_on_tables(&self) -> bool {
        matches!(
            self,
            Self::CsharpPartial
                | Self::Private
                | Self::NativeType(_)
                | Self::NativeTypePackName(_)
                | Self::OriginalOrder
        )
    }

    pub fn accepted_on_unions(&self) -> bool {
        matches!(
            self,
            Self::CsharpPartial | Self::Private | Self::NativeType(_) | Self::NativeTypePackName(_)
        )
    }

    pub fn accepted_on_rpc_services(&self) -> bool {
        matches!(self, Self::Private)
    }

    pub fn accepted_on_struct_fields(&self) -> bool {
        matches!(
            self,
            Self::Key
                | Self::Hash(_)
                | Self::CppType(_)
                | Self::CppPtrType(_)
                | Self::CppPtrTypeGet(_)
                | Self::NativeInline
                | Self::NativeDefault(_)
        )
    }

    pub fn accepted_on_table_fields(&self) -> bool {
        matches!(
            self,
            Self::ForceAlign(_)
                | Self::Required
                | Self::Deprecated
                | Self::Key
                | Self::Shared
                | Self::NestedFlatbuffer(_)
                | Self::Id(_)
                | Self::Hash(_)
                | Self::CppType(_)
                | Self::CppPtrType(_)
                | Self::CppPtrTypeGet(_)
                | Self::CppStrType(_)
                | Self::CppStrFlexCtor
                | Self::NativeInline
                | Self::NativeDefault(_)
                | Self::Flexbuffer
        )
    }

    pub fn accepted_on_rpc_methods(&self) -> bool {
        matches!(self, Self::Streaming(_) | Self::Idempotent)
    }
}

#[derive(Clone, Debug)]
pub struct NamespacePath {
    pub span: Span,
    pub parts: Vec<RawIdentifier>,
}

#[derive(Clone, Debug)]
pub struct Type {
    pub span: Span,
    pub kind: TypeKind,
}

impl Type {
    pub fn hash_unspanned<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.kind.hash_unspanned(state);
    }

    pub fn eq_unspanned(&self, other: &Self) -> bool {
        self.kind.eq_unspanned(&other.kind)
    }
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Builtin(BuiltinType),
    Vector { inner_type: Box<Type> },
    Array { inner_type: Box<Type>, size: u32 },
    Path(NamespacePath),
    Invalid,
}

impl TypeKind {
    pub fn hash_unspanned<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        std::hash::Hash::hash(&std::mem::discriminant(self), state);
        match self {
            TypeKind::Builtin(t) => std::hash::Hash::hash(t, state),
            TypeKind::Vector { inner_type } => inner_type.hash_unspanned(state),
            TypeKind::Array { inner_type, size } => {
                inner_type.hash_unspanned(state);
                std::hash::Hash::hash(size, state);
            }
            TypeKind::Path(path) => {
                std::hash::Hash::hash(&path.parts, state);
            }
            TypeKind::Invalid => (),
        }
    }

    pub fn eq_unspanned(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeKind::Builtin(t0), TypeKind::Builtin(t1)) => t0 == t1,
            (TypeKind::Vector { inner_type: t0 }, TypeKind::Vector { inner_type: t1 }) => {
                t0.eq_unspanned(t1)
            }
            (
                TypeKind::Array {
                    inner_type: t0,
                    size: s0,
                },
                TypeKind::Array {
                    inner_type: t1,
                    size: s1,
                },
            ) => s0 == s1 && t0.eq_unspanned(t1),
            (TypeKind::Path(p0), TypeKind::Path(p1)) => p0.parts == p1.parts,
            (TypeKind::Invalid, TypeKind::Invalid) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntegerType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl IntegerType {
    pub fn byte_size(&self) -> u32 {
        match self {
            IntegerType::U8 => 1,
            IntegerType::I8 => 1,
            IntegerType::U16 => 2,
            IntegerType::I16 => 2,
            IntegerType::U32 => 4,
            IntegerType::I32 => 4,
            IntegerType::U64 => 8,
            IntegerType::I64 => 8,
        }
    }

    pub fn flatbuffer_name(&self) -> &'static str {
        match self {
            IntegerType::U8 => "uint8",
            IntegerType::U16 => "uint16",
            IntegerType::U32 => "uint32",
            IntegerType::U64 => "uint64",
            IntegerType::I8 => "int8",
            IntegerType::I16 => "int16",
            IntegerType::I32 => "int32",
            IntegerType::I64 => "int64",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FloatType {
    F32,
    F64,
}

impl FloatType {
    pub fn byte_size(&self) -> u32 {
        match self {
            FloatType::F32 => 4,
            FloatType::F64 => 8,
        }
    }

    pub fn flatbuffer_name(&self) -> &'static str {
        match self {
            FloatType::F32 => "float32",
            FloatType::F64 => "float64",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Literal {
    pub span: Span,
    pub kind: LiteralKind,
}

#[derive(Clone, Debug)]
pub enum LiteralKind {
    Bool(bool),
    Integer { is_negative: bool, value: String },
    Float { is_negative: bool, value: String },
    String(String),
    List(Vec<Literal>),
    Null,
    Constant(String),
}

#[derive(Clone, Debug)]
pub struct IntegerLiteral {
    pub span: Span,
    pub is_negative: bool,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub value: String,
}

impl Type {
    pub fn to_string(&self, ctx: &Ctx) -> String {
        pub struct Fmt<F>(pub F)
        where
            F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result;

        impl<F> std::fmt::Debug for Fmt<F>
        where
            F: Fn(&mut std::fmt::Formatter) -> std::fmt::Result,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                (self.0)(f)
            }
        }

        format!("{:?}", Fmt(|f| self.fmt(ctx, f)))
    }

    fn fmt(&self, ctx: &Ctx, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TypeKind::Builtin(BuiltinType::Bool) => write!(f, "bool"),
            TypeKind::Builtin(BuiltinType::String) => write!(f, "string"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::U8)) => write!(f, "uint8"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::U16)) => write!(f, "uint16"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::U32)) => write!(f, "uint32"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::U64)) => write!(f, "uint64"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::I8)) => write!(f, "int8"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::I16)) => write!(f, "int16"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::I32)) => write!(f, "int32"),
            TypeKind::Builtin(BuiltinType::Integer(IntegerType::I64)) => write!(f, "int64"),
            TypeKind::Builtin(BuiltinType::Float(FloatType::F32)) => write!(f, "float32"),
            TypeKind::Builtin(BuiltinType::Float(FloatType::F64)) => write!(f, "float64"),
            TypeKind::Vector { inner_type } => {
                write!(f, "[")?;
                inner_type.fmt(ctx, f)?;
                write!(f, "]")?;
                Ok(())
            }
            TypeKind::Array { inner_type, size } => {
                write!(f, "[")?;
                inner_type.fmt(ctx, f)?;
                write!(f, ": {}]", size)?;
                Ok(())
            }
            TypeKind::Path(path) => path.fmt(ctx, f),
            TypeKind::Invalid => write!(f, "!"),
        }
    }
}

impl NamespacePath {
    fn fmt(&self, ctx: &Ctx, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for &part in &self.parts {
            if first {
                write!(f, "{}", ctx.resolve_identifier(part))?;
            } else {
                write!(f, ".{}", ctx.resolve_identifier(part))?;
            }
            first = false
        }
        Ok(())
    }
}
