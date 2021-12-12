use super::backend::{
    Backend, DeclarationNames, DeclarationTranslationContext, NamespaceNames, RelativeNamespace,
    ResolvedType,
};
use crate::{
    ast::{FloatType, IntegerType},
    intermediate_language::types::{AssignMode, Literal},
};
use heck::{CamelCase, SnakeCase};
use std::{borrow::Cow, path::Path, process::Command};

#[derive(Debug, Clone)]
pub struct RustBackend;

#[derive(Clone, Debug)]
pub struct Namespace {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub owned_name: String,
    pub ref_name: String,
}

#[derive(Clone, Debug)]
pub struct TableField {
    pub name: String,
    pub vtable_type: String,
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
    pub owned_name: String,
    pub ref_name: String,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: String,
    pub owned_type: String,
    pub getter_return_type: String,
    pub getter_code: String,
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

const BINDING_KIND_TYPES: &str = "types";

fn reserve_module_name(path: &str, namespace_names: &mut NamespaceNames<'_, '_>) -> String {
    let name = path.to_snake_case().into();
    namespace_names
        .namespace_names
        .try_reserve_repeat(BINDING_KIND_TYPES, name, '_')
        .into()
}

fn reserve_type_name(path: &str, declaration_names: &mut DeclarationNames<'_, '_>) -> String {
    let name = path.to_camel_case().into();
    declaration_names
        .declaration_names
        .try_reserve_repeat(BINDING_KIND_TYPES, name, '_')
        .into()
}

fn reserve_field_name(
    path: &str,
    binding_kind: &'static str,
    declaration_names: &mut DeclarationNames<'_, '_>,
) -> String {
    let name = path.to_snake_case().into();
    declaration_names
        .declaration_names
        .try_reserve_repeat(binding_kind, name, '_')
        .into()
}

fn reserve_rust_enum_variant_name(
    path: &str,
    binding_kind: &'static str,
    declaration_names: &mut DeclarationNames<'_, '_>,
) -> String {
    let name = path.to_camel_case().into();
    declaration_names
        .declaration_names
        .try_reserve_repeat(binding_kind, name, '_')
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
        namespace_names: &mut NamespaceNames<'_, '_>,
        namespace_name: &crate::intermediate_language::types::AbsolutePath,
        _namespace: &crate::intermediate_language::types::Namespace,
    ) -> Namespace {
        let name = namespace_name.0.last().map_or_else(String::new, |name| {
            reserve_module_name(name, namespace_names)
        });
        Namespace { name }
    }

    fn generate_table(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Table,
    ) -> Table {
        let decl_name = decl_name.0.last().unwrap();
        Table {
            owned_name: reserve_type_name(decl_name, declaration_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), declaration_names),
        }
    }

    fn generate_struct(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Struct,
    ) -> Struct {
        let decl_name = decl_name.0.last().unwrap();
        Struct {
            owned_name: reserve_type_name(decl_name, declaration_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), declaration_names),
        }
    }

    fn generate_enum(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        decl: &crate::intermediate_language::types::Enum,
    ) -> Enum {
        let decl_name = decl_name.0.last().unwrap();
        Enum {
            name: reserve_type_name(decl_name, declaration_names),
            repr_type: format!("{:?}", decl.type_).to_lowercase(),
        }
    }

    fn generate_union(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::Union,
    ) -> Union {
        let decl_name = decl_name.0.last().unwrap();
        Union {
            owned_name: reserve_type_name(decl_name, declaration_names),
            ref_name: reserve_type_name(&format!("{}Ref", decl_name), declaration_names),
        }
    }

    fn generate_rpc_service(
        &mut self,
        _declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        _decl_name: &crate::intermediate_language::types::AbsolutePath,
        _decl: &crate::intermediate_language::types::RpcService,
    ) -> RpcService {
        RpcService {}
    }

    fn generate_table_field(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::TableInfo,
        _parent: &crate::intermediate_language::types::Table,
        field_name: &str,
        field: &crate::intermediate_language::types::TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> TableField {
        let name = reserve_field_name(
            field_name,
            "name",
            &mut translation_context.declaration_names,
        );
        let create_name = reserve_field_name(
            field_name,
            "create_name",
            &mut translation_context.declaration_names,
        );
        let read_type;
        let owned_type;
        let vtable_type;
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
                vtable_type =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = vtable_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!("Option<{}>", vtable_type);
                    create_trait = format!("WriteAsOptional<{}>", vtable_type);
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
                let owned_name =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                vtable_type = format!("planus::Offset<{}>", owned_name);
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!("Box<{}>", owned_name);
                    create_trait = format!("WriteAs<{}>", vtable_type);
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!("Option<Box<{}>>", owned_name);
                    create_trait = format!("WriteAsOptional<{}>", vtable_type);
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
                let owned_name =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                vtable_type = format!("planus::Offset<{}>", owned_name);
                if matches!(field.assign_mode, AssignMode::Required) {
                    read_type = format!(
                        "{}<'a>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = owned_name.clone();
                    create_trait = format!("WriteAsUnion<{}>", owned_name);
                } else {
                    read_type = format!(
                        "Option<{}<'a>>",
                        format_relative_namespace(&relative_namespace, ref_name)
                    );
                    owned_type = format!("Option<{}>", owned_name);
                    create_trait = format!("WriteAsOptionalUnion<{}>", owned_name);
                }
            }
            ResolvedType::Enum(_, info, relative_namespace, variants) => {
                vtable_type =
                    format_relative_namespace(&relative_namespace, &info.name).to_string();
                if let AssignMode::HasDefault(Literal::EnumTag { variant_index, .. }) =
                    &field.assign_mode
                {
                    read_type = vtable_type.clone();
                    owned_type = vtable_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);

                    code_for_default =
                        format!("{}::{}", owned_type, variants[*variant_index].name).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!("Option<{}>", vtable_type);
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAsOptional<{}>", vtable_type);
                }
            }
            ResolvedType::Vector(_) => todo!(),
            ResolvedType::Array(_, _) => todo!(),
            ResolvedType::String => {
                vtable_type = "planus::Offset<str>".to_string();
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
                vtable_type = "bool".to_string();
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
                vtable_type = integer_type(&typ).to_string();
                if let AssignMode::HasDefault(Literal::Int(lit)) = &field.assign_mode {
                    read_type = vtable_type.clone();
                    owned_type = vtable_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                    code_for_default = format!("{}", lit).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!("Option<{}>", vtable_type);
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAsOptional<{}>", vtable_type);
                }
            }
            ResolvedType::Float(typ) => {
                vtable_type = float_type(&typ).to_string();
                if let AssignMode::HasDefault(Literal::Float(lit)) = &field.assign_mode {
                    read_type = vtable_type.clone();
                    owned_type = vtable_type.clone();
                    create_trait = format!("WriteAs<{}>", owned_type);
                    code_for_default = format!("{}", lit).into();
                    none_replacement = Some(code_for_default.to_string());
                } else {
                    read_type = format!("Option<{}>", vtable_type);
                    owned_type = read_type.clone();
                    create_trait = format!("WriteAsOptional<{}>", float_type(&typ));
                }
            }
        }
        TableField {
            name,
            vtable_type,
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
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::StructInfo,
        _parent: &crate::intermediate_language::types::Struct,
        field_name: &str,
        _field: &crate::intermediate_language::types::StructField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> StructField {
        let name = reserve_field_name(
            field_name,
            "name",
            &mut translation_context.declaration_names,
        );
        let owned_type;
        let getter_code;
        let getter_return_type;

        match resolved_type {
            ResolvedType::Struct(_, info, relative_namespace) => {
                owned_type =
                    format_relative_namespace(&relative_namespace, &info.owned_name).to_string();
                let ref_name = format_relative_namespace(&relative_namespace, &info.ref_name);
                getter_return_type = format!("{}<'a>", ref_name);
                getter_code = format!("{}(buffer)", ref_name);
            }
            ResolvedType::Enum(decl, info, relative_namespace, _) => {
                owned_type = format_relative_namespace(&relative_namespace, &info.name).to_string();
                getter_return_type = format!("planus::Result<{}>", owned_type);
                getter_code = format!(
                    "{}::from_le_bytes(*buffer.as_array()).try_into()",
                    integer_type(&decl.type_)
                );
            }
            ResolvedType::Bool => {
                owned_type = "bool".to_string();
                getter_return_type = owned_type.clone();
                getter_code = "buffer.as_array()[0] != 0".to_string();
            }
            ResolvedType::Integer(typ) => {
                owned_type = integer_type(&typ).to_string();
                getter_return_type = owned_type.clone();
                getter_code = format!("{}::from_le_bytes(*buffer.as_array())", owned_type);
            }
            ResolvedType::Float(typ) => {
                owned_type = float_type(&typ).to_string();
                getter_return_type = owned_type.clone();
                getter_code = format!("{}::from_le_bytes(*buffer.as_array())", owned_type);
            }
            _ => unreachable!(),
        }
        StructField {
            name,
            owned_type,
            getter_return_type,
            getter_code,
        }
    }

    fn generate_enum_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::EnumInfo,
        _parent: &crate::intermediate_language::types::Enum,
        key: &str,
        value: &crate::intermediate_language::types::IntegerLiteral,
    ) -> EnumVariant {
        let name =
            reserve_rust_enum_variant_name(key, "name", &mut translation_context.declaration_names);

        EnumVariant {
            name,
            value: format!("{}", value),
        }
    }

    fn generate_union_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::UnionInfo,
        _parent: &crate::intermediate_language::types::Union,
        key: &str,
        _index: u8,
        _value: &crate::intermediate_language::types::UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> UnionVariant {
        let create_name = reserve_field_name(
            &format!("create_{}", key),
            "create_function",
            &mut translation_context.declaration_names,
        );
        let enum_name = reserve_rust_enum_variant_name(
            key,
            "variant_name",
            &mut translation_context.declaration_names,
        );
        let create_trait;
        let owned_type;
        let ref_type;

        match resolved_type {
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
                    "WriteAsOffset<{}>",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
            }
            _ => todo!(),
        }
        UnionVariant {
            create_name,
            enum_name,
            create_trait,
            owned_type,
            ref_type,
        }
    }

    fn generate_rpc_method(
        &mut self,
        _translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
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

pub fn format_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let output = Command::new("rustfmt")
        .args(&[path.as_ref().as_os_str()])
        .output()?;

    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8(output.stderr).unwrap());
    }

    Ok(())
}
