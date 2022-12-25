use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
};

use crate::{ast, codegen::backend_translation::BackendVariant, intermediate_language::types::*};

pub type Keywords = HashSet<&'static str>;

pub struct Names<'keywords> {
    keywords: &'keywords Keywords,
    names: BTreeMap<&'static str, HashSet<String>>,
}

pub struct NamespaceNames<'a, 'keywords> {
    pub global_names: &'a mut Names<'keywords>,
    pub namespace_names: &'a mut Names<'keywords>,
}

pub struct DeclarationNames<'a, 'keywords> {
    pub global_names: &'a mut Names<'keywords>,
    pub namespace_names: &'a mut Names<'keywords>,
    pub declaration_names: &'a mut Names<'keywords>,
}

impl<'keywords> Names<'keywords> {
    pub fn new(keywords: &'keywords Keywords) -> Self {
        Self {
            keywords,
            names: Default::default(),
        }
    }

    pub fn try_reserve(&mut self, binding_kind: &'static str, value: &str) -> bool {
        if value.is_empty() || self.keywords.contains(value) {
            false
        } else {
            let names = self.names.entry(binding_kind).or_default();
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
        binding_kind: &'static str,
        value: Cow<'a, str>,
        padding: char,
    ) -> Cow<'a, str> {
        if self.try_reserve(binding_kind, &value) {
            return value;
        }

        let mut value = format!("{value}{padding}");
        while !self.try_reserve(binding_kind, &value) {
            value.push(padding);
        }

        value.into()
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
            Self::Table(translated_decl, decl) => Self::Table(translated_decl.clone(), decl),
            Self::Struct(translated_decl, decl) => Self::Struct(translated_decl.clone(), decl),
            Self::Enum(translated_decl, decl) => Self::Enum(translated_decl.clone(), decl),
            Self::Union(translated_decl, decl) => Self::Union(translated_decl.clone(), decl),
            Self::RpcService(translated_decl, decl) => {
                Self::RpcService(translated_decl.clone(), decl)
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
        let mut current_namespace = current_namespace.0.iter().peekable();
        let mut other_namespace = other_namespace.0.iter().peekable();
        let mut shared = AbsolutePath(Vec::new());

        while current_namespace.peek().is_some()
            && current_namespace.peek() == other_namespace.peek()
        {
            shared.0.push(current_namespace.next().unwrap().clone());
            other_namespace.next();
        }
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
    self_name: Option<&'a str>,
    separator: &'a str,
    value: &'a RelativeNamespace<'a, B>,
    output_shared_ancestor: bool,
    name: F,
    trailing_part: &'a str,
}

impl<'a, B: ?Sized + Backend> RelativeNamespace<'a, B> {
    pub fn format(
        &'a self,
        output_shared_ancestor: bool,
        super_name: &'a str,
        self_name: Option<&'a str>,
        separator: &'a str,
        name: impl 'a + Fn(&B::NamespaceInfo) -> &str,
        trailing_part: &'a str,
    ) -> impl 'a + std::fmt::Display {
        FormattedRelativeNamespace {
            super_name,
            self_name,
            separator,
            output_shared_ancestor,
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
        if self.value.ascend_count == 0 {
            if let Some(self_name) = self.self_name {
                write!(f, "{self_name}")?;
            }
        }
        #[allow(clippy::bool_to_int_with_if)]
        let skip = if self.output_shared_ancestor { 0 } else { 1 };
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
    Enum(
        &'a Enum,
        &'a B::EnumInfo,
        RelativeNamespace<'a, B>,
        &'a [BackendVariant<B::EnumVariantInfo>],
    ),
    Union(&'a Union, &'a B::UnionInfo, RelativeNamespace<'a, B>),
    Vector(Box<ResolvedType<'a, B>>),
    Array(Box<ResolvedType<'a, B>>, u32),
    String,
    Bool,
    Integer(ast::IntegerType),
    Float(ast::FloatType),
}

pub struct DeclarationTranslationContext<'a, 'keywords, B: ?Sized + Backend> {
    pub declaration_names: DeclarationNames<'a, 'keywords>,
    pub translated_namespaces: &'a [B::NamespaceInfo],
    pub translated_decls: &'a [(AbsolutePath, DeclInfo<'a, B>)],
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
        namespace_names: &mut NamespaceNames<'_, '_>,
        namespace_name: &AbsolutePath,
        namespace: &Namespace,
    ) -> Self::NamespaceInfo;

    fn generate_table(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &Table,
    ) -> Self::TableInfo;

    fn generate_struct(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &Struct,
    ) -> Self::StructInfo;

    fn generate_enum(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &Enum,
    ) -> Self::EnumInfo;

    fn generate_union(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &Union,
    ) -> Self::UnionInfo;

    fn generate_rpc_service(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &RpcService,
    ) -> Self::RpcServiceInfo;

    fn generate_table_field(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::TableInfo,
        parent: &Table,
        field_name: &str,
        field: &TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::TableFieldInfo;

    fn generate_struct_field(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::StructInfo,
        parent: &Struct,
        field_name: &str,
        field: &StructField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::StructFieldInfo;

    fn generate_enum_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::EnumInfo,
        parent: &Enum,
        key: &str,
        value: &IntegerLiteral,
    ) -> Self::EnumVariantInfo;

    #[allow(clippy::too_many_arguments)]
    fn generate_union_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::UnionInfo,
        parent: &Union,
        key: &str,
        index: u8,
        value: &UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> Self::UnionVariantInfo;

    #[allow(clippy::too_many_arguments)]
    fn generate_rpc_method(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::RpcServiceInfo,
        parent: &RpcService,
        method_name: &str,
        method: &RpcMethod,
    ) -> Self::RpcMethodInfo;
}
