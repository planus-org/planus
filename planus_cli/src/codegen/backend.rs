use crate::{ast, intermediate_language::types::*};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
};

#[derive(Copy, Clone, Debug)]
pub enum Scope {
    Global = 0,
    Namespace = 1,
    Declaration = 2,
}

pub struct ReservedNames {
    keywords: HashSet<&'static str>,
    names: [BTreeMap<&'static str, HashSet<String>>; 3],
}

impl ReservedNames {
    pub fn new(keywords: &[&'static str]) -> Self {
        Self {
            keywords: keywords.iter().cloned().collect(),
            names: Default::default(),
        }
    }

    pub fn try_reserve(&mut self, scope: Scope, binding_kind: &'static str, value: &str) -> bool {
        if self.keywords.contains(value) {
            false
        } else {
            let names = self.names[scope as usize].entry(binding_kind).or_default();
            if names.contains(value) {
                false
            } else {
                names.insert(value.to_string());
                true
            }
        }
    }

    pub fn try_reserve_repeat<'a>(
        &mut self,
        scope: Scope,
        binding_kind: &'static str,
        value: Cow<'a, str>,
        padding: char,
    ) -> Cow<'a, str> {
        if self.try_reserve(scope, binding_kind, &value) {
            return value.into();
        }

        let mut value = format!("{}{}", value, padding);
        while !self.try_reserve(scope, binding_kind, &value) {
            value.push(padding);
        }

        value.into()
    }

    pub fn clear(&mut self, scope: Scope) {
        for bindings in self.names[scope as usize].values_mut() {
            bindings.clear();
        }
    }
}

pub enum DeclInfo<'a, B: ?Sized + Backend> {
    Table(B::TableInfo, &'a Table),
    Struct(B::StructInfo, &'a Struct),
    Enum(B::EnumInfo, &'a Enum),
    Union(B::UnionInfo, &'a Union),
    RpcService(B::RpcServiceInfo, &'a RpcService),
}

impl<'a, B: ?Sized + Backend> Clone for DeclInfo<'a, B> {
    fn clone(&self) -> Self {
        match self {
            Self::Table(translated_decl, decl) => {
                Self::Table(translated_decl.clone(), decl.clone())
            }
            Self::Struct(translated_decl, decl) => {
                Self::Struct(translated_decl.clone(), decl.clone())
            }
            Self::Enum(translated_decl, decl) => Self::Enum(translated_decl.clone(), decl.clone()),
            Self::Union(translated_decl, decl) => {
                Self::Union(translated_decl.clone(), decl.clone())
            }
            Self::RpcService(translated_decl, decl) => {
                Self::RpcService(translated_decl.clone(), decl.clone())
            }
        }
    }
}

pub struct RelativeNamespace<'a, B: ?Sized + Backend> {
    pub ascend_count: usize,
    pub path: Vec<&'a B::NamespaceInfo>,
}

impl<'a, B: ?Sized + Backend> RelativeNamespace<'a, B> {
    pub fn new(
        current_namespace: &AbsolutePath,
        other_namespace: &AbsolutePath,
        translated_namespaces: &'a [B::NamespaceInfo],
        declarations: &'a Declarations,
    ) -> RelativeNamespace<'a, B> {
        println!("{:#?}", current_namespace);
        println!("{:#?}", other_namespace);
        let mut current_namespace = current_namespace.0.iter().peekable();
        let mut other_namespace = other_namespace.0.iter().peekable();
        let mut shared = AbsolutePath(Vec::new());

        while current_namespace.peek().is_some()
            && current_namespace.peek() == other_namespace.peek()
        {
            shared.0.push(current_namespace.next().unwrap().clone());
            other_namespace.next();
        }
        println!();
        println!("{:#?}", current_namespace.clone().collect::<Vec<_>>());
        println!("{:#?}", other_namespace.clone().collect::<Vec<_>>());
        println!("{:#?}", shared);

        let index = declarations.namespaces.get_index_of(&shared).unwrap();
        let mut path = vec![&translated_namespaces[index]];

        for remaining in other_namespace {
            shared.0.push(remaining.clone());
            let index = declarations.namespaces.get_index_of(&shared).unwrap();
            path.push(&translated_namespaces[index]);
        }

        RelativeNamespace {
            ascend_count: current_namespace.count(),
            path,
        }
    }
}

struct FormattedRelativeNamespace<'a, B: ?Sized + Backend, F> {
    super_name: &'a str,
    separator: &'a str,
    value: &'a RelativeNamespace<'a, B>,
    output_first: bool,
    name: F,
    trailing_part: &'a str,
}

impl<'a, B: ?Sized + Backend> RelativeNamespace<'a, B> {
    pub fn format(
        &'a self,
        output_first: bool,
        super_name: &'a str,
        separator: &'a str,
        name: impl 'a + Fn(&B::NamespaceInfo) -> &str,
        trailing_part: &'a str,
    ) -> impl 'a + std::fmt::Display {
        FormattedRelativeNamespace {
            super_name,
            separator,
            output_first,
            value: self,
            name,
            trailing_part,
        }
    }
}

impl<'a, B: ?Sized + Backend, F: Fn(&B::NamespaceInfo) -> &str> std::fmt::Display
    for FormattedRelativeNamespace<'a, B, F>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.value.ascend_count {
            write!(f, "{}{}", self.super_name, self.separator)?;
        }
        let skip = if self.output_first { 0 } else { 1 };
        let mut has_output = false;
        for (i, info) in self.value.path.iter().skip(skip).enumerate() {
            if i > 0 {
                write!(f, "{}{}", self.separator, (self.name)(info))?;
            } else {
                write!(f, "{}", (self.name)(info))?;
            }
            has_output = true;
        }
        if !self.trailing_part.is_empty() {
            if has_output {
                write!(f, "{}{}", self.separator, self.trailing_part)?;
            } else {
                write!(f, "{}", self.trailing_part)?;
            }
        }

        Ok(())
    }
}

pub enum ResolvedType<'a, B: ?Sized + Backend> {
    Struct(&'a Struct, &'a B::StructInfo, RelativeNamespace<'a, B>),
    Table(&'a Table, &'a B::TableInfo, RelativeNamespace<'a, B>),
    Enum(&'a Enum, &'a B::EnumInfo, RelativeNamespace<'a, B>),
    Union(&'a Union, &'a B::UnionInfo, RelativeNamespace<'a, B>),
    Vector(Box<ResolvedType<'a, B>>),
    Array(Box<ResolvedType<'a, B>>, u32),
    String,
    Bool,
    Integer(ast::IntegerType),
    Float(ast::FloatType),
}

pub trait Backend {
    type NamespaceInfo: std::fmt::Debug + Clone;
    type TableInfo: std::fmt::Debug + Clone;
    type StructInfo: std::fmt::Debug + Clone;
    type EnumInfo: std::fmt::Debug + Clone;
    type UnionInfo: std::fmt::Debug + Clone;
    type RpcServiceInfo: std::fmt::Debug + Clone;
    type TableFieldInfo: std::fmt::Debug + Clone;
    type StructFieldInfo: std::fmt::Debug + Clone;
    type EnumVariantInfo: std::fmt::Debug + Clone;
    type UnionVariantInfo: std::fmt::Debug + Clone;
    type RpcMethodInfo: std::fmt::Debug + Clone;

    const KEYWORDS: &'static [&'static str];

    fn generate_namespace(
        &mut self,
        reserved_names: &mut ReservedNames,
        namespace_name: &AbsolutePath,
        namespace: &Namespace,
    ) -> Self::NamespaceInfo;

    fn generate_table(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &AbsolutePath,
        decl: &Table,
    ) -> Self::TableInfo;

    fn generate_struct(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &AbsolutePath,
        decl: &Struct,
    ) -> Self::StructInfo;

    fn generate_enum(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &AbsolutePath,
        decl: &Enum,
    ) -> Self::EnumInfo;

    fn generate_union(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &AbsolutePath,
        decl: &Union,
    ) -> Self::UnionInfo;

    fn generate_rpc_service(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &AbsolutePath,
        decl: &RpcService,
    ) -> Self::RpcServiceInfo;

    fn generate_table_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<Self>)],
        parent_info: &Self::TableInfo,
        parent: &Table,
        field_name: &str,
        field: &TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::TableFieldInfo;

    fn generate_struct_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<Self>)],
        parent_info: &Self::StructInfo,
        parent: &Struct,
        field_name: &str,
        field: &StructField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::StructFieldInfo;

    fn generate_enum_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<Self>)],
        parent_info: &Self::EnumInfo,
        parent: &Enum,
        key: &str,
        value: &IntegerLiteral,
    ) -> Self::EnumVariantInfo;

    fn generate_union_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<Self>)],
        parent_info: &Self::UnionInfo,
        parent: &Union,
        key: &str,
        index: u8,
        value: &UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::UnionVariantInfo;

    fn generate_rpc_method(
        &mut self,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[Self::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<Self>)],
        parent_info: &Self::RpcServiceInfo,
        parent: &RpcService,
        method_name: &str,
        method: &RpcMethod,
    ) -> Self::RpcMethodInfo;
}
