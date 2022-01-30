use std::{collections::BTreeSet, fmt::Display};

use codespan::{FileId, Span};
use indexmap::IndexMap;

use crate::ast;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclarationIndex(pub usize);
impl DeclarationIndex {
    pub const INVALID: DeclarationIndex = DeclarationIndex(usize::MAX);
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
}

#[derive(Debug, Default)]
pub struct Namespace {
    pub child_namespaces: IndexMap<String, NamespaceIndex>,
    pub declaration_ids: IndexMap<String, DeclarationIndex>,
}

#[derive(Debug)]
pub struct Declaration {
    pub definition_span: Span,
    pub file_id: FileId,
    pub namespace_id: NamespaceIndex,
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
    pub object_tag_size: u32,
    pub object_alignment_mask: u32,
    pub object_alignment: u32,
    pub deprecated: bool,
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
    pub vector_stride: u32,
}

#[derive(Debug)]
pub struct StructField {
    pub type_: SimpleType,
    pub offset: u32,
    pub size: u32,
    pub padding_after_field: u32,
}

#[derive(Debug, Clone)]
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

#[derive(Copy, Clone, Debug)]
pub enum FloatLiteral {
    F32(f32),
    F64(f64),
}

impl Display for FloatLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatLiteral::F32(v) => write!(f, "{:?}", v),
            FloatLiteral::F64(v) => write!(f, "{:?}", v),
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
