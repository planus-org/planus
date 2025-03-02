use std::{borrow::Cow, collections::BTreeSet, fmt::Display};

use codespan::{FileId, Span};
use indexmap::IndexMap;

use crate::ast::{Docstrings, FloatType, IntegerType};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct AbsolutePath(pub Vec<String>);

impl AbsolutePath {
    pub const ROOT_PATH: Self = Self(Vec::new());

    pub fn push<S: Into<String>>(&mut self, value: S) {
        self.0.push(value.into())
    }

    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }

    #[must_use]
    pub fn clone_push<S: AsRef<str>>(&self, value: S) -> Self {
        let mut r = self.clone();
        r.push(value.as_ref());
        r
    }

    #[must_use]
    pub fn clone_pop(&self) -> Self {
        assert!(!self.0.is_empty());
        Self(self.0[..self.0.len() - 1].to_vec())
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for part in &self.0 {
            if first {
                first = false;
                write!(f, "{part}")?;
            } else {
                write!(f, ".{part}")?;
            }
        }
        Ok(())
    }
}

pub struct RelativePath {
    pub count_until_shared_parent: usize,
    pub remaining: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationIndex(pub usize);
impl DeclarationIndex {
    pub const INVALID: DeclarationIndex = DeclarationIndex(usize::MAX);
}

impl Display for DeclarationIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a DeclarationIndex> for DeclarationIndex {
    fn from(decl: &'a DeclarationIndex) -> Self {
        *decl
    }
}

impl<'a, 'b> From<&'b &'a DeclarationIndex> for DeclarationIndex {
    fn from(decl: &'b &'a DeclarationIndex) -> Self {
        **decl
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamespaceIndex(pub usize);
impl NamespaceIndex {
    pub const INVALID: NamespaceIndex = NamespaceIndex(usize::MAX);
}

impl<'a> From<&'a NamespaceIndex> for NamespaceIndex {
    fn from(namespace: &'a NamespaceIndex) -> Self {
        *namespace
    }
}

impl<'a, 'b> From<&'b &'a NamespaceIndex> for NamespaceIndex {
    fn from(namespace: &'b &'a NamespaceIndex) -> Self {
        **namespace
    }
}

#[derive(Debug)]
pub struct Declarations {
    pub namespaces: IndexMap<AbsolutePath, Namespace>,
    pub declarations: IndexMap<AbsolutePath, Declaration>,
    pub children: Vec<Vec<DeclarationIndex>>,
    pub parents: Vec<Vec<DeclarationIndex>>,
}

impl Declarations {
    pub fn new(
        namespaces: IndexMap<AbsolutePath, Namespace>,
        declarations: IndexMap<AbsolutePath, Declaration>,
    ) -> Self {
        let children = declarations
            .values()
            .map(|decl| match &decl.kind {
                DeclarationKind::Table(decl) => decl.children(),
                DeclarationKind::Struct(decl) => decl.children(),
                DeclarationKind::Enum(_) => Vec::new(),
                DeclarationKind::Union(decl) => decl.children(),
                DeclarationKind::RpcService(decl) => decl.children(),
            })
            .collect::<Vec<_>>();

        let mut parents = (0..declarations.len())
            .map(|_| Vec::new())
            .collect::<Vec<_>>();
        for (parent_decl_id, children) in children.iter().enumerate() {
            for child_decl_id in children {
                parents[child_decl_id.0].push(DeclarationIndex(parent_decl_id));
            }
        }

        Self {
            namespaces,
            declarations,
            children,
            parents,
        }
    }

    pub fn get_namespace(&self, index: NamespaceIndex) -> (&AbsolutePath, &Namespace) {
        self.namespaces.get_index(index.0).unwrap()
    }

    pub fn get_root_namespace(&self) -> (NamespaceIndex, &Namespace) {
        let (index, _, namespace) = self.namespaces.get_full(&AbsolutePath::ROOT_PATH).unwrap();
        (NamespaceIndex(index), namespace)
    }

    pub fn get_declaration(&self, index: DeclarationIndex) -> (&AbsolutePath, &Declaration) {
        self.declarations.get_index(index.0).unwrap()
    }

    pub fn iter_declarations(
        &self,
    ) -> impl Iterator<Item = (DeclarationIndex, &AbsolutePath, &Declaration)> {
        self.declarations
            .iter()
            .enumerate()
            .map(|(i, (k, v))| (DeclarationIndex(i), k, v))
    }

    pub fn format_type_kind(&self, type_: &TypeKind) -> Cow<'static, str> {
        match type_ {
            TypeKind::Table(index) => {
                Cow::Owned(format!("table {}", self.get_declaration(*index).0))
            }
            TypeKind::Union(index) => {
                Cow::Owned(format!("union {}", self.get_declaration(*index).0))
            }
            TypeKind::Vector(type_) => {
                Cow::Owned(format!("[{}]", self.format_type_kind(&type_.kind)))
            }
            TypeKind::Array(type_, size) => {
                Cow::Owned(format!("[{}; {size}]", self.format_type_kind(&type_.kind)))
            }
            TypeKind::SimpleType(type_) => self.format_simple_type(type_),
            TypeKind::String => Cow::Borrowed("string"),
        }
    }

    pub fn format_simple_type(&self, type_: &SimpleType) -> Cow<'static, str> {
        match type_ {
            SimpleType::Struct(index) => {
                Cow::Owned(format!("struct {}", self.get_declaration(*index).0))
            }
            SimpleType::Enum(index) => {
                Cow::Owned(format!("enum {}", self.get_declaration(*index).0))
            }
            SimpleType::Bool => Cow::Borrowed("bool"),
            SimpleType::Integer(type_) => Cow::Borrowed(type_.flatbuffer_name()),
            SimpleType::Float(type_) => Cow::Borrowed(type_.flatbuffer_name()),
        }
    }
}

#[derive(Debug)]
pub struct Namespace {
    /// The span is where the namespace path is defined
    pub spans: Vec<(FileId, Option<Span>)>,
    pub docstrings: Docstrings,
    pub child_namespaces: IndexMap<String, NamespaceIndex>,
    pub declaration_ids: IndexMap<String, DeclarationIndex>,
}

impl Default for Namespace {
    fn default() -> Self {
        Self {
            spans: Default::default(),
            docstrings: Docstrings::new(None),
            child_namespaces: Default::default(),
            declaration_ids: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Declaration {
    pub definition_span: Span,
    pub file_id: FileId,
    pub namespace_id: NamespaceIndex,
    pub kind: DeclarationKind,
    pub docstrings: Docstrings,
}

#[derive(Debug)]
pub enum DeclarationKind {
    Table(Table),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    RpcService(RpcService),
}

impl DeclarationKind {
    pub fn kind_as_str(&self) -> &'static str {
        match self {
            DeclarationKind::Table(_) => "table",
            DeclarationKind::Struct(_) => "struct",
            DeclarationKind::Enum(_) => "enum",
            DeclarationKind::Union(_) => "union",
            DeclarationKind::RpcService(_) => "rpc_service",
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub fields: IndexMap<String, TableField>,
    pub alignment_order: Vec<usize>,
    pub max_size: u32,
    pub max_vtable_size: u32,
    pub max_alignment: u32,
}

#[derive(Debug)]
pub struct TableField {
    /// The index into the vtable.
    /// Not necessarily the same as the index into the IndexMap.
    pub vtable_index: u32,
    pub span: Span,
    pub type_: Type,
    pub assign_mode: AssignMode,
    pub object_value_size: u32,
    pub object_tag_kind: TableFieldTagKind,
    pub object_alignment_mask: u32,
    pub object_alignment: u32,
    pub deprecated: bool,
    pub docstrings: Docstrings,
}

#[derive(Copy, Clone, Debug)]
/// Indicates whether a [`TableField`] has a preceding tag. Used by unions and union vectors.
pub enum TableFieldTagKind {
    None,
    UnionTag,
    UnionTagVector,
}

impl TableFieldTagKind {
    /// The size of the preceding tag or 0 if not present.
    pub fn size(self) -> u32 {
        match self {
            TableFieldTagKind::None => 0,
            TableFieldTagKind::UnionTag => 1,
            TableFieldTagKind::UnionTagVector => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AssignMode {
    Required,
    Optional,
    HasDefault(Literal),
}

#[derive(Debug)]
pub struct Struct {
    pub fields: IndexMap<String, StructField>,
    pub size: u32,
    pub alignment: u32,
}

#[derive(Debug)]
pub struct StructField {
    pub type_: SimpleType,
    pub offset: u32,
    pub size: u32,
    pub padding_after_field: u32,
    pub docstrings: Docstrings,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub type_: IntegerType,
    pub variants: IndexMap<IntegerLiteral, EnumVariant>,
    pub alignment: u32,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub span: Span,
    pub name: String,
    pub docstrings: Docstrings,
}

#[derive(Debug)]
pub struct Union {
    pub variants: IndexMap<String, UnionVariant>,
}

#[derive(Debug)]
pub struct UnionVariant {
    pub type_: Type,
    pub docstrings: Docstrings,
}

#[derive(Debug)]
pub struct RpcService {
    pub methods: IndexMap<String, RpcMethod>,
}

#[derive(Debug)]
pub struct RpcMethod {
    pub argument_type: Type,
    pub return_type: Type,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Type {
    pub span: Span,
    pub kind: TypeKind,
}

impl TypeKind {
    pub fn is_scalar(&self) -> bool {
        match self {
            TypeKind::Table(_)
            | TypeKind::Union(_)
            | TypeKind::Vector(_)
            | TypeKind::Array(_, _)
            | TypeKind::String => false,
            TypeKind::SimpleType(type_) => type_.is_scalar(),
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, &TypeKind::SimpleType(SimpleType::Enum(..)))
    }

    pub fn is_type_with_tag(&self) -> bool {
        match self {
            TypeKind::Union(_) => true,
            TypeKind::Vector(inner) => matches!(inner.kind, TypeKind::Union(_)),
            _ => false,
        }
    }

    fn add_children(&self, children: &mut BTreeSet<DeclarationIndex>) {
        match self {
            TypeKind::Table(decl_id) | TypeKind::Union(decl_id) => {
                children.insert(*decl_id);
            }
            TypeKind::Vector(type_) | TypeKind::Array(type_, _) => {
                type_.kind.add_children(children)
            }
            TypeKind::SimpleType(kind) => {
                kind.add_children(children);
            }
            TypeKind::String => (),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Table(DeclarationIndex),
    Union(DeclarationIndex),
    Vector(Box<Type>),
    Array(Box<Type>, u32),
    SimpleType(SimpleType),
    String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SimpleType {
    Struct(DeclarationIndex),
    Enum(DeclarationIndex),
    Bool,
    Integer(IntegerType),
    Float(FloatType),
}

impl SimpleType {
    pub fn is_scalar(&self) -> bool {
        match self {
            SimpleType::Struct(_) => false,
            SimpleType::Enum(_)
            | SimpleType::Bool
            | SimpleType::Integer(_)
            | SimpleType::Float(_) => true,
        }
    }

    fn add_children(&self, children: &mut BTreeSet<DeclarationIndex>) {
        match self {
            SimpleType::Struct(decl_id) | SimpleType::Enum(decl_id) => {
                children.insert(*decl_id);
            }
            SimpleType::Bool | SimpleType::Integer(_) | SimpleType::Float(_) => (),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Literal {
    Bool(bool),
    String(String),
    Int(IntegerLiteral),
    Float(FloatLiteral),
    Array(Vec<Literal>),
    Vector(Vec<Literal>),
    EnumTag {
        variant_index: usize,
        value: IntegerLiteral,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntegerLiteral {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
}

impl IntegerLiteral {
    pub fn is_zero(&self) -> bool {
        match self {
            IntegerLiteral::U8(n) => *n == 0,
            IntegerLiteral::I8(n) => *n == 0,
            IntegerLiteral::U16(n) => *n == 0,
            IntegerLiteral::I16(n) => *n == 0,
            IntegerLiteral::U32(n) => *n == 0,
            IntegerLiteral::I32(n) => *n == 0,
            IntegerLiteral::U64(n) => *n == 0,
            IntegerLiteral::I64(n) => *n == 0,
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            IntegerLiteral::U8(v) => *v as u64,
            IntegerLiteral::I8(v) => *v as u64,
            IntegerLiteral::U16(v) => *v as u64,
            IntegerLiteral::I16(v) => *v as u64,
            IntegerLiteral::U32(v) => *v as u64,
            IntegerLiteral::I32(v) => *v as u64,
            IntegerLiteral::U64(v) => *v,
            IntegerLiteral::I64(v) => *v as u64,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(v) => write!(f, "{v}"),
            Literal::String(v) => write!(f, "{v}"),
            Literal::Int(v) => write!(f, "{v}"),
            Literal::Float(v) => write!(f, "{v}"),
            Literal::Array(vs) | Literal::Vector(vs) => {
                write!(f, "[")?;
                let mut first = true;
                for v in vs {
                    if !first {
                        write!(f, ", {v}")?;
                    } else {
                        first = false;
                        write!(f, "{v}")?;
                    }
                }
                write!(f, "]")
            }
            Literal::EnumTag { value, .. } => write!(f, "{value}"),
        }
    }
}

impl Display for IntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegerLiteral::U8(v) => write!(f, "{v}"),
            IntegerLiteral::I8(v) => write!(f, "{v}"),
            IntegerLiteral::U16(v) => write!(f, "{v}"),
            IntegerLiteral::I16(v) => write!(f, "{v}"),
            IntegerLiteral::U32(v) => write!(f, "{v}"),
            IntegerLiteral::I32(v) => write!(f, "{v}"),
            IntegerLiteral::U64(v) => write!(f, "{v}"),
            IntegerLiteral::I64(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FloatLiteral {
    F32(f32),
    F64(f64),
}

impl Display for FloatLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatLiteral::F32(v) => write!(f, "{v:?}"),
            FloatLiteral::F64(v) => write!(f, "{v:?}"),
        }
    }
}
impl Table {
    fn children(&self) -> Vec<DeclarationIndex> {
        let mut children = BTreeSet::new();
        for field in self.fields.values() {
            field.type_.kind.add_children(&mut children);
        }
        children.into_iter().collect()
    }

    pub fn get_field_for_vtable_index(
        &self,
        vtable_index: u32,
    ) -> Option<(&str, &TableField, bool)> {
        for (field_name, field) in &self.fields {
            if vtable_index == field.vtable_index {
                return Some((field_name, field, field.type_.kind.is_type_with_tag()));
            }
            if vtable_index == field.vtable_index + 1 && field.type_.kind.is_type_with_tag() {
                return Some((field_name, field, false));
            }
        }
        None
    }
}

impl Struct {
    fn children(&self) -> Vec<DeclarationIndex> {
        let mut children = BTreeSet::new();
        for field in self.fields.values() {
            field.type_.add_children(&mut children);
        }
        children.into_iter().collect()
    }
}

impl Union {
    fn children(&self) -> Vec<DeclarationIndex> {
        let mut children = BTreeSet::new();
        for variant in self.variants.values() {
            variant.type_.kind.add_children(&mut children);
        }
        children.into_iter().collect()
    }
}

impl RpcService {
    fn children(&self) -> Vec<DeclarationIndex> {
        let mut children = BTreeSet::new();
        for method in self.methods.values() {
            method.argument_type.kind.add_children(&mut children);
            method.return_type.kind.add_children(&mut children);
        }
        children.into_iter().collect()
    }
}

impl IntegerLiteral {
    pub fn default_value_from_type(type_: &crate::ast::IntegerType) -> Self {
        match type_ {
            crate::ast::IntegerType::U8 => Self::U8(0),
            crate::ast::IntegerType::U16 => Self::U16(0),
            crate::ast::IntegerType::U32 => Self::U32(0),
            crate::ast::IntegerType::U64 => Self::U64(0),
            crate::ast::IntegerType::I8 => Self::I8(0),
            crate::ast::IntegerType::I16 => Self::I16(0),
            crate::ast::IntegerType::I32 => Self::I32(0),
            crate::ast::IntegerType::I64 => Self::I64(0),
        }
    }

    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::U8(n) => Self::U8(n.wrapping_add(1)),
            Self::I8(n) => Self::I8(n.wrapping_add(1)),
            Self::U16(n) => Self::U16(n.wrapping_add(1)),
            Self::I16(n) => Self::I16(n.wrapping_add(1)),
            Self::U32(n) => Self::U32(n.wrapping_add(1)),
            Self::I32(n) => Self::I32(n.wrapping_add(1)),
            Self::U64(n) => Self::U64(n.wrapping_add(1)),
            Self::I64(n) => Self::I64(n.wrapping_add(1)),
        }
    }
}

impl From<&crate::ast::BuiltinType> for TypeKind {
    fn from(value: &crate::ast::BuiltinType) -> TypeKind {
        match value {
            crate::ast::BuiltinType::Bool => TypeKind::SimpleType(SimpleType::Bool),
            crate::ast::BuiltinType::Integer(typ) => {
                TypeKind::SimpleType(SimpleType::Integer(*typ))
            }
            crate::ast::BuiltinType::Float(typ) => TypeKind::SimpleType(SimpleType::Float(*typ)),
            crate::ast::BuiltinType::String => TypeKind::String,
        }
    }
}
