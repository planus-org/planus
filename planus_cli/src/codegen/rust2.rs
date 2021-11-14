use super::backend::{Backend, RelativeNamespace, ReservedNames, ResolvedType, Scope};
use crate::{
    ast::{FloatType, IntegerType},
    intermediate_language::types::{AbsolutePath, AssignMode, Literal},
};
use heck::{CamelCase, SnakeCase};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct RustBackend;

#[derive(Clone, Debug)]
pub struct Namespace {
    name: String,
}

#[derive(Clone, Debug)]
pub struct Table {
    owned_name: String,
    ref_name: String,
}

#[derive(Clone, Debug)]
pub struct TableField {
    pub name: String,
    pub owned_type: String,
    pub read_type: String,
    pub create_name: String,
    pub create_trait: String,
    pub required: bool,
    pub code_for_default: Cow<'static, str>,
    pub none_replacement: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Struct {
    owned_name: String,
    ref_name: String,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: String,
    pub owned_type: String,
    pub read_type: String,
    pub read_type_no_lifetime: String,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: String,
    pub repr_type: String,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct Union {
    pub owned_name: String,
    pub ref_name: String,
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub index: u8,
    pub create_name: String,
    pub create_trait: String,
    pub enum_name: String,
    pub owned_type: String,
    pub ref_type: String,
}

#[derive(Clone, Debug)]
pub struct RpcService {
    // TODO
}

#[derive(Clone, Debug)]
pub struct RpcMethod {
    // TODO
}

const BINDING_KIND_TYPES: &'static str = "types";

fn reserve_module_name(path: &str, reserved_names: &mut ReservedNames) -> String {
    let name = path.to_snake_case().into();
    reserved_names
        .try_reserve_repeat(Scope::Namespace, BINDING_KIND_TYPES, name, '_')
        .into()
}

fn reserve_type_name(path: &str, reserved_names: &mut ReservedNames) -> String {
    let name = path.to_camel_case().into();
    reserved_names
        .try_reserve_repeat(Scope::Namespace, BINDING_KIND_TYPES, name, '_')
        .into()
}

fn reserve_field_name(
    path: &str,
    binding_kind: &'static str,
    reserved_names: &mut ReservedNames,
) -> String {
    let name = path.to_snake_case().into();
    reserved_names
        .try_reserve_repeat(Scope::Declaration, binding_kind, name, '_')
        .into()
}

fn reserve_rust_enum_variant_name(
    path: &str,
    binding_kind: &'static str,
    reserved_names: &mut ReservedNames,
) -> String {
    let name = path.to_camel_case().into();
    reserved_names
        .try_reserve_repeat(Scope::Declaration, binding_kind, name, '_')
        .into()
}

fn format_relative_namespace<'a>(
    relative_namespace: &'a RelativeNamespace<'a, RustBackend>,
    trailing_part: &'a str,
) -> impl 'a + std::fmt::Display {
    relative_namespace.format(false, "super", "::", |info| &info.name, trailing_part)
}

impl Backend for RustBackend {
    type NamespaceInfo = Namespace;
    type TableInfo = Table;
    type TableFieldInfo = TableField;
    type StructInfo = Struct;
    type StructFieldInfo = StructField;
    type EnumInfo = Enum;
    type EnumVariantInfo = EnumVariant;
    type UnionInfo = Union;
    type UnionVariantInfo = UnionVariant;
    type RpcServiceInfo = RpcService;
    type RpcMethodInfo = RpcMethod;

    const KEYWORDS: &'static [&'static str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "Self", "self", "static", "struct", "super", "trait",
        "true", "type", "union", "unsafe", "use", "where", "while", "abstract", "become", "box",
        "do", "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
    ];

    fn generate_namespace(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        namespace_name: &crate::intermediate_language::types::AbsolutePath,
        _namespace: &crate::intermediate_language::types::Namespace,
    ) -> Namespace {
        let name = namespace_name.0.last().map_or_else(String::new, |name| {
            reserve_module_name(name, reserved_names)
        });
        Namespace { name }
    }

    fn generate_table(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Table,
    ) -> Table {
        let decl_name = decl_name.0.last().unwrap();
        Table {
            owned_name: reserve_type_name(decl_name, reserved_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), reserved_names),
        }
    }

    fn generate_struct(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Struct,
    ) -> Struct {
        let decl_name = decl_name.0.last().unwrap();
        Struct {
            owned_name: reserve_type_name(decl_name, reserved_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), reserved_names),
        }
    }

    fn generate_enum(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        decl: &crate::intermediate_language::types::Enum,
    ) -> Enum {
        let decl_name = decl_name.0.last().unwrap();
        Enum {
            name: reserve_type_name(decl_name, reserved_names),
            repr_type: format!("{:?}", decl.type_).to_lowercase().into(),
        }
    }

    fn generate_union(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Union,
    ) -> Union {
        let decl_name = decl_name.0.last().unwrap();
        Union {
            owned_name: reserve_type_name(decl_name, reserved_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), reserved_names),
        }
    }

    fn generate_rpc_service(
        &mut self,
        _reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::RpcService,
    ) -> RpcService {
        RpcService {}
    }

    fn generate_table_field(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _translated_decls: &[(AbsolutePath, super::backend::DeclInfo<Self>)],
        _parent_info: &Self::TableInfo,
        _parent: &crate::intermediate_language::types::Table,
        field_name: &str,
        field: &crate::intermediate_language::types::TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> TableField {
        let name = reserve_field_name(field_name, "name", reserved_names);
        let create_name = reserve_field_name(field_name, "create_name", reserved_names);
        let read_type;
        let owned_type;
        let create_trait;
        let mut code_for_default = Cow::Borrowed("Default::default()");
        let mut none_replacement: Option<String> = None;
        match resolved_type {
            ResolvedType::Struct(
                _,
                Struct {
                    owned_name,
                    ref_name,
                },
                relative_namespace,
            ) => {
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type =
                        format_relative_namespace(&relative_namespace, owned_name).to_string();
                    create_trait = format!("WriteAs<{}>", owned_type);
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!(
                        "Option<{}>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                    create_trait = format!("WriteAsOptional<{}>", owned_type);
                }
            }
            ResolvedType::Table(
                _,
                Table {
                    owned_name,
                    ref_name,
                },
                relative_namespace,
            ) => {
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!(
                        "Box<{}>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                    create_trait = format!(
                        "WriteAs<Offset<{}>>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!(
                        "Option<Box<{}>>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                    create_trait = format!(
                        "WriteAsOptional<Offset<{}>>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                }
            }
            ResolvedType::Union(
                _,
                Union {
                    owned_name,
                    ref_name,
                },
                relative_namespace,
            ) => {
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type =
                        format_relative_namespace(&relative_namespace, owned_name).to_string();
                    create_trait = format!(
                        "WriteAsUnion<{}>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!(
                        "Option<{}>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                    create_trait = format!(
                        "WriteAsOptionalUnion<{}>",
                        format_relative_namespace(&relative_namespace, owned_name)
                    );
                }
            }
            ResolvedType::Enum(_, info, relative_namespace) => {
                if let AssignMode::HasDefault(Literal::EnumTag { name, .. }) = &field.assign_mode {
                    // TODO: get the renamed variant name
                    read_type =
                        format_relative_namespace(&relative_namespace, &info.name).to_string();
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                    code_for_default = format!("{}::{}", owned_type, name).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!(
                        "Option<{}>",
                        format_relative_namespace(&relative_namespace, &info.name)
                    );
                    owned_type = read_type.clone();
                    create_trait = format!(
                        "WriteAsOptional<{}>",
                        format_relative_namespace(&relative_namespace, &info.name)
                    );
                }
            }
            ResolvedType::Vector(_) => todo!(),
            ResolvedType::Array(_, _) => todo!(),
            ResolvedType::String => {
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = "&'a str".to_string();
                    owned_type = "String".to_string();
                    create_trait = "WriteAs<Offset<str>>".to_string();
                } else {
                    read_type = "Option<&'a str>".to_string();
                    owned_type = "Option<String>".to_string();
                    create_trait = "WriteAsOptional<Offset<str>>".to_string();
                }
            }
            ResolvedType::Bool => {
                if let AssignMode::HasDefault(Literal::Bool(lit)) = &field.assign_mode {
                    read_type = "bool".to_string();
                    owned_type = "bool".to_string();
                    create_trait = "WriteAs<bool>".to_string();
                    code_for_default = format!("{}", lit).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = "Option<bool>".to_string();
                    owned_type = "Option<bool>".to_string();
                    create_trait = "WriteAsOptional<bool>".to_string();
                }
            }
            ResolvedType::Integer(typ) => {
                if let AssignMode::HasDefault(Literal::Int(lit)) = &field.assign_mode {
                    read_type = integer_type(&typ).to_string();
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                    code_for_default = format!("{}", lit).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!("Option<{}>", integer_type(&typ));
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAsOptional<{}>", integer_type(&typ));
                }
            }
            ResolvedType::Float(typ) => {
                if let AssignMode::HasDefault(Literal::Float(lit)) = &field.assign_mode {
                    read_type = float_type(&typ).to_string();
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                    code_for_default = format!("{}", lit).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!("Option<{}>", float_type(&typ));
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAsOptional<{}>", float_type(&typ));
                }
            }
        }
        TableField {
            name,
            owned_type,
            read_type,
            create_name,
            create_trait,
            required: matches!(field.assign_mode, AssignMode::Required),
            code_for_default,
            none_replacement,
        }
    }

    fn generate_struct_field(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _translated_decls: &[(AbsolutePath, super::backend::DeclInfo<Self>)],
        _parent_info: &Self::StructInfo,
        _parent: &crate::intermediate_language::types::Struct,
        field_name: &str,
        _field: &crate::intermediate_language::types::StructField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> StructField {
        let name = reserve_field_name(field_name, "name", reserved_names);
        let owned_type;
        let read_type;
        let read_type_no_lifetime;

        match resolved_type {
            ResolvedType::Struct(_, info, relative_namespace) => {
                owned_type =
                    format_relative_namespace(&relative_namespace, &info.owned_name).to_string();
                read_type_no_lifetime =
                    format_relative_namespace(&relative_namespace, &info.ref_name).to_string();
                read_type = format!("{}<'a>", read_type_no_lifetime);
            }
            ResolvedType::Enum(_, info, relative_namespace) => {
                owned_type = format_relative_namespace(&relative_namespace, &info.name).to_string();
                read_type_no_lifetime = owned_type.clone();
                read_type = owned_type.clone();
            }
            ResolvedType::Bool => {
                owned_type = "bool".to_string();
                read_type_no_lifetime = owned_type.clone();
                read_type = owned_type.clone();
            }
            ResolvedType::Integer(typ) => {
                owned_type = integer_type(&typ).to_string();
                read_type_no_lifetime = owned_type.clone();
                read_type = owned_type.clone();
            }
            ResolvedType::Float(typ) => {
                owned_type = float_type(&typ).to_string();
                read_type_no_lifetime = owned_type.clone();
                read_type = owned_type.clone();
            }
            _ => unreachable!(),
        }
        StructField {
            name,
            owned_type,
            read_type,
            read_type_no_lifetime,
        }
    }

    fn generate_enum_variant(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _translated_decls: &[(AbsolutePath, super::backend::DeclInfo<Self>)],
        _parent_info: &Self::EnumInfo,
        _parent: &crate::intermediate_language::types::Enum,
        key: &str,
        value: &crate::intermediate_language::types::IntegerLiteral,
    ) -> EnumVariant {
        let name = reserve_rust_enum_variant_name(key, "name", reserved_names);

        EnumVariant {
            name,
            value: format!("{}", value),
        }
    }

    fn generate_union_variant(
        &mut self,
        reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _translated_decls: &[(AbsolutePath, super::backend::DeclInfo<Self>)],
        _parent_info: &Self::UnionInfo,
        _parent: &crate::intermediate_language::types::Union,
        key: &str,
        index: u8,
        _value: &crate::intermediate_language::types::UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> UnionVariant {
        let create_name = reserve_field_name(
            &format!("create_{}", key),
            "create_function",
            reserved_names,
        );
        let enum_name = reserve_rust_enum_variant_name(key, "variant_name", reserved_names);
        let create_trait;
        let owned_type;
        let ref_type;

        match resolved_type {
            ResolvedType::Struct(_, info, relative_namespace) => {
                owned_type = format!(
                    "{}",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
                ref_type = format!(
                    "{}<'a>",
                    format_relative_namespace(&relative_namespace, &info.ref_name)
                );
                create_trait = format!(
                    "WriteAs<{}>",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
            }
            ResolvedType::Table(_, info, relative_namespace) => {
                owned_type = format!(
                    "Box<{}>",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
                ref_type = format!(
                    "{}<'a>",
                    format_relative_namespace(&relative_namespace, &info.ref_name)
                );
                create_trait = format!(
                    "WriteAs<Offset<{}>>",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
            }
            _ => todo!(),
        }
        UnionVariant {
            index,
            create_name,
            enum_name,
            create_trait,
            owned_type,
            ref_type,
        }
    }

    fn generate_rpc_method(
        &mut self,
        _reserved_names: &mut super::backend::ReservedNames,
        _translated_namespaces: &[Self::NamespaceInfo],
        _translated_decls: &[(AbsolutePath, super::backend::DeclInfo<Self>)],
        _parent_info: &Self::RpcServiceInfo,
        _parent: &crate::intermediate_language::types::RpcService,
        _method_name: &str,
        _method: &crate::intermediate_language::types::RpcMethod,
    ) -> RpcMethod {
        todo!()
    }
}

fn integer_type(type_: &IntegerType) -> &'static str {
    match &type_ {
        IntegerType::U8 => "u8",
        IntegerType::I8 => "i8",
        IntegerType::U16 => "u16",
        IntegerType::I16 => "i16",
        IntegerType::U32 => "u32",
        IntegerType::I32 => "i32",
        IntegerType::U64 => "u64",
        IntegerType::I64 => "i64",
    }
}

fn float_type(type_: &FloatType) -> &'static str {
    match &type_ {
        FloatType::F32 => "f32",
        FloatType::F64 => "f64",
    }
}
