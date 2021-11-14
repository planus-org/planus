use std::fmt::Display;

use codespan::{FileId, Span};
use indexmap::IndexMap;

use crate::ast;

// TODO: docstrings
// TODO: rpc services

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct AbsolutePath(pub Vec<String>);

impl AbsolutePath {
    pub const ROOT_PATH: Self = Self(Vec::new());
    pub fn push<S: Into<String>>(&mut self, value: S) {
        self.0.push(value.into())
    }
    pub fn clone_push<S: AsRef<str>>(&self, value: S) -> Self {
        let mut r = self.clone();
        r.push(value.as_ref());
        r
    }
    pub fn clone_pop(&self) -> Self {
        assert!(!self.0.is_empty());
        Self(self.0[..self.0.len() - 1].to_vec())
    }
    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for part in &self.0 {
            if first {
                first = false;
                write!(f, "{}", part)?;
            } else {
                write!(f, ".{}", part)?;
            }
        }
        Ok(())
    }
}

pub struct RelativePath {
    pub count_until_shared_parent: usize,
    pub remaining: Vec<String>,
}

#[derive(Copy, Clone, Debug)]
pub struct DeclarationIndex(pub usize);
impl DeclarationIndex {
    pub const INVALID: DeclarationIndex = DeclarationIndex(usize::MAX);
}

impl<'a> Into<DeclarationIndex> for &'a DeclarationIndex {
    fn into(self) -> DeclarationIndex {
        *self
    }
}

impl<'a, 'b> Into<DeclarationIndex> for &'b &'a DeclarationIndex {
    fn into(self) -> DeclarationIndex {
        **self
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamespaceIndex(pub usize);
impl NamespaceIndex {
    pub const INVALID: NamespaceIndex = NamespaceIndex(usize::MAX);
}

impl<'a> Into<NamespaceIndex> for &'a NamespaceIndex {
    fn into(self) -> NamespaceIndex {
        *self
    }
}

impl<'a, 'b> Into<NamespaceIndex> for &'b &'a NamespaceIndex {
    fn into(self) -> NamespaceIndex {
        **self
    }
}

#[derive(Debug)]
pub struct Declarations {
    namespaces: IndexMap<AbsolutePath, Namespace>,
    declarations: IndexMap<AbsolutePath, Declaration>,
}

impl Declarations {
    pub fn new(
        namespaces: IndexMap<AbsolutePath, Namespace>,
        declarations: IndexMap<AbsolutePath, Declaration>,
    ) -> Self {
        Self {
            namespaces,
            declarations,
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

    pub fn namespace_count(&self) -> usize {
        self.namespaces.len()
    }

    pub fn declaration_count(&self) -> usize {
        self.declarations.len()
    }

    pub fn iter_declarations(
        &self,
    ) -> impl Iterator<Item = (DeclarationIndex, &AbsolutePath, &Declaration)> {
        self.declarations
            .iter()
            .enumerate()
            .map(|(i, (k, v))| (DeclarationIndex(i), k, v))
    }
}

#[derive(Debug, Default)]
pub struct Namespace {
    // TODO: rpc services
    // TODO: root_type, file_identifier, file_extension
    // TODO: attributes?
    pub child_namespaces: IndexMap<String, NamespaceIndex>,
    pub declaration_ids: IndexMap<String, DeclarationIndex>,
}

#[derive(Debug)]
pub struct Declaration {
    pub definition_span: Span,
    pub file_id: FileId,
    pub kind: DeclarationKind,
}

#[derive(Debug)]
pub enum DeclarationKind {
    Table(Table),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    RpcService(RpcService),
}

#[derive(Debug)]
pub struct Table {
    pub fields: IndexMap<String, TableField>,
    pub alignment_order: Vec<usize>,
    pub max_size: u32,
    pub max_vtable_index: u32,
}

#[derive(Debug)]
pub struct TableField {
    /// The index into the vtable. Not necessarily the same as the index,
    /// into the IndexMap
    pub vtable_index: u32,
    pub type_: Type,
    pub object_value_size: u32,
    pub object_tag_size: u32,
    pub object_alignment_mask: u32,
    pub object_alignment: u32,
    pub assignment: Option<Literal>,
    pub required: bool,
    pub deprecated: bool,
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
    pub assignment: Option<Literal>,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct Enum {
    pub type_: ast::IntegerType,
    pub variants: IndexMap<IntegerLiteral, String>,
    pub alignment: u32,
}

#[derive(Debug)]
pub struct Union {
    pub variants: IndexMap<String, UnionVariant>,
}

#[derive(Debug)]
pub struct UnionVariant {
    pub type_: Type,
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

#[derive(Debug)]
pub struct Type {
    pub span: Span,
    pub kind: TypeKind,
}

#[derive(Debug)]
pub enum TypeKind {
    Table(DeclarationIndex),
    Union(DeclarationIndex),
    Vector(Box<Type>),
    Array(Box<Type>, u32),
    SimpleType(SimpleType),
    String,
}

#[derive(Debug)]
pub enum SimpleType {
    Struct(DeclarationIndex),
    Enum(DeclarationIndex),
    Bool,
    Integer(ast::IntegerType),
    Float(ast::FloatType),
}

#[derive(Debug)]
pub enum Literal {
    Bool(bool),
    String(String),
    Int(IntegerLiteral),
    Float(FloatLiteral),
    Array(Vec<Literal>),
    Vector(Vec<Literal>),
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

impl Display for IntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegerLiteral::U8(v) => write!(f, "{}", v),
            IntegerLiteral::I8(v) => write!(f, "{}", v),
            IntegerLiteral::U16(v) => write!(f, "{}", v),
            IntegerLiteral::I16(v) => write!(f, "{}", v),
            IntegerLiteral::U32(v) => write!(f, "{}", v),
            IntegerLiteral::I32(v) => write!(f, "{}", v),
            IntegerLiteral::U64(v) => write!(f, "{}", v),
            IntegerLiteral::I64(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug)]
pub enum FloatLiteral {
    F32(f32),
    F64(f64),
}
