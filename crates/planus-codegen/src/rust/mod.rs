pub mod analysis;

use std::{
    borrow::Cow,
    io::Write,
    process::{Command, Stdio},
};

use eyre::Context;
use heck::{ToSnakeCase, ToUpperCamelCase};
use planus_types::{
    ast::{FloatType, IntegerType},
    intermediate::{self, AbsolutePath, AssignMode, DeclarationIndex, Literal, TableFieldTagKind},
};

use super::backend::{
    Backend, DeclarationNames, DeclarationTranslationContext, NamespaceNames, RelativeNamespace,
    ResolvedType,
};

#[derive(Debug, Clone)]
pub struct RustBackend {
    pub default_analysis: Vec<bool>,
    pub eq_analysis: Vec<bool>,
    pub infallible_analysis: Vec<bool>,
}

#[derive(Clone, Debug)]
pub struct Namespace {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub owned_name: String,
    pub ref_name: String,
    pub builder_name: String,
    pub should_do_default: bool,
    pub should_do_eq: bool,
}

#[derive(Clone, Debug)]
pub struct TableField {
    pub name: String,
    pub name_with_as: String,
    pub primitive_size: u32,
    pub vtable_type: String,
    pub owned_type: String,
    pub read_type: String,
    pub create_name: String,
    pub create_trait: String,
    pub required: bool,
    pub optional: bool,
    pub has_default: bool,
    pub impl_default_code: Cow<'static, str>,
    pub serialize_default: Option<Cow<'static, str>>,
    pub deserialize_default: Option<Cow<'static, str>>,
    pub try_from_code: String,
    pub is_copy: bool,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub owned_name: String,
    pub ref_name: String,
    pub should_do_default: bool,
    pub should_do_eq: bool,
    pub should_do_infallible_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: String,
    pub owned_type: String,
    pub getter_return_type: String,
    pub getter_code: String,
    pub can_do_infallible_conversion: bool,
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
    pub builder_name: String,
    pub ref_name: String,
    pub ref_name_with_lifetime: String,
    pub should_do_eq: bool,
    pub should_do_infallible_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub create_name: String,
    pub create_trait: String,
    pub builder_name: String,
    pub enum_name: String,
    pub owned_type: String,
    pub ref_type: String,
    pub is_struct: bool,
    pub can_do_infallible_conversion: bool,
}

#[derive(Clone, Debug)]
pub struct RpcService {}

#[derive(Clone, Debug)]
pub struct RpcMethod {}

const BINDING_KIND_TYPES: &str = "types";

fn reserve_module_name(path: &str, namespace_names: &mut NamespaceNames<'_, '_>) -> String {
    let name = path.to_snake_case().into();
    namespace_names
        .namespace_names
        .try_reserve_repeat(BINDING_KIND_TYPES, name, '_')
        .into()
}

fn reserve_type_name(path: &str, declaration_names: &mut DeclarationNames<'_, '_>) -> String {
    let name = path.to_upper_camel_case().into();
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
    let name = path.to_upper_camel_case().into();
    declaration_names
        .declaration_names
        .try_reserve_repeat(binding_kind, name, '_')
        .into()
}

fn format_relative_namespace<'a>(
    relative_namespace: &'a RelativeNamespace<'a, RustBackend>,
    trailing_part: &'a str,
) -> impl 'a + std::fmt::Display {
    relative_namespace.format(
        false,
        "super",
        Some("self::"),
        "::",
        |info| &info.name,
        trailing_part,
    )
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
        "_",
    ];

    fn generate_namespace(
        &mut self,
        namespace_names: &mut NamespaceNames<'_, '_>,
        namespace_name: &AbsolutePath,
        _namespace: &intermediate::Namespace,
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
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        _decl: &intermediate::Table,
    ) -> Table {
        let decl_name = decl_name.0.last().unwrap();
        Table {
            owned_name: reserve_type_name(decl_name, declaration_names),
            ref_name: reserve_type_name(&format!("{decl_name}Ref"), declaration_names),
            builder_name: reserve_type_name(&format!("{decl_name}Builder"), declaration_names),
            should_do_default: self.default_analysis[decl_id.0],
            should_do_eq: self.eq_analysis[decl_id.0],
        }
    }

    fn generate_struct(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        _decl: &intermediate::Struct,
    ) -> Struct {
        let decl_name = decl_name.0.last().unwrap();
        Struct {
            owned_name: reserve_type_name(decl_name, declaration_names),
            ref_name: reserve_type_name(&format!("{decl_name}Ref"), declaration_names),
            should_do_default: self.default_analysis[decl_id.0],
            should_do_eq: self.eq_analysis[decl_id.0],
            should_do_infallible_conversion: self.infallible_analysis[decl_id.0],
        }
    }

    fn generate_enum(
        &mut self,
        declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        _decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &intermediate::Enum,
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
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &intermediate::Union,
    ) -> Union {
        let decl_name = decl_name.0.last().unwrap();
        let ref_name = reserve_type_name(&format!("{decl_name}Ref"), declaration_names);
        let builder_name = reserve_type_name(&format!("{decl_name}Builder"), declaration_names);
        Union {
            owned_name: reserve_type_name(decl_name, declaration_names),
            builder_name,
            ref_name_with_lifetime: if decl.variants.is_empty() {
                ref_name.clone()
            } else {
                format!("{ref_name}<'a>")
            },
            ref_name,
            should_do_eq: self.eq_analysis[decl_id.0],
            should_do_infallible_conversion: self.infallible_analysis[decl_id.0],
        }
    }

    fn generate_table_field(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::TableInfo,
        _parent: &intermediate::Table,
        field_name: &str,
        field: &intermediate::TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> TableField {
        let name = reserve_field_name(
            field_name,
            "name",
            &mut translation_context.declaration_names,
        );
        let mut name_with_as = format!("{field_name}_as").to_snake_case();
        if name_with_as == "as" {
            name_with_as = format!("{field_name}_as");
        }
        let create_name = reserve_field_name(
            field_name,
            "create_name",
            &mut translation_context.declaration_names,
        );
        let read_type;
        let owned_type;
        let vtable_type;
        let create_trait;
        let is_copy;
        let primitive_size;
        let mut serialize_default: Option<Cow<'static, str>> = None;
        let mut deserialize_default: Option<Cow<'static, str>> = None;
        let mut impl_default_code: Cow<'static, str> = "::core::default::Default::default()".into();
        let mut try_from_code = if matches!(field.assign_mode, AssignMode::Optional) {
            format!(
                r#"
                    if let ::core::option::Option::Some({name}) = value.{name}()? {{
                        ::core::option::Option::Some(::core::convert::TryInto::try_into({name})?)
                    }} else {{
                        ::core::option::Option::None
                    }}
                "#
            )
        } else {
            format!("::core::convert::TryInto::try_into(value.{name}()?)?")
        };

        match resolved_type {
            ResolvedType::Struct(
                field_decl_id,
                decl,
                Struct {
                    owned_name,
                    ref_name,
                    ..
                },
                relative_namespace,
            ) => {
                vtable_type =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                is_copy = true;
                primitive_size = decl.size;
                match &field.assign_mode {
                    AssignMode::Required => {
                        read_type = format!(
                            "{}<'a>",
                            format_relative_namespace(&relative_namespace, ref_name)
                        );
                        owned_type = vtable_type.clone();
                        create_trait = format!("WriteAs<{owned_type}>");
                        if self.infallible_analysis[field_decl_id.0] {
                            try_from_code = format!("::core::convert::Into::into(value.{name}()?)");
                        }
                    }
                    AssignMode::Optional => {
                        read_type = format!(
                            "::core::option::Option<{}<'a>>",
                            format_relative_namespace(&relative_namespace, ref_name)
                        );
                        owned_type = format!("::core::option::Option<{vtable_type}>");
                        create_trait = format!("WriteAsOptional<{vtable_type}>");
                        if self.infallible_analysis[field_decl_id.0] {
                            try_from_code = format!(
                                r#"
                                    value.{name}()?.map(::core::convert::Into::into)
                                "#
                            );
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ResolvedType::Table(
                _,
                _,
                Table {
                    owned_name,
                    ref_name,
                    ..
                },
                relative_namespace,
            ) => {
                let owned_name =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                is_copy = false;
                primitive_size = 4;
                vtable_type = format!("::planus::Offset<{owned_name}>");
                match &field.assign_mode {
                    AssignMode::Required => {
                        read_type = format!(
                            "{}<'a>",
                            format_relative_namespace(&relative_namespace, ref_name)
                        );
                        owned_type = format!("::planus::alloc::boxed::Box<{owned_name}>");
                        create_trait = format!("WriteAs<{vtable_type}>");
                        try_from_code = format!(
                            "::planus::alloc::boxed::Box::new(::core::convert::TryInto::try_into(value.{name}()?)?)"
                        );
                    }
                    AssignMode::Optional => {
                        read_type = format!(
                            "::core::option::Option<{}<'a>>",
                            format_relative_namespace(&relative_namespace, ref_name)
                        );
                        owned_type = format!(
                            "::core::option::Option<::planus::alloc::boxed::Box<{owned_name}>>"
                        );
                        create_trait = format!("WriteAsOptional<{vtable_type}>");
                        try_from_code = format!(
                            r#"
                                if let ::core::option::Option::Some({name}) = value.{name}()? {{
                                    ::core::option::Option::Some(::planus::alloc::boxed::Box::new(::core::convert::TryInto::try_into({name})?))
                                }} else {{
                                    ::core::option::Option::None
                                }}
                            "#
                        );
                    }
                    AssignMode::HasDefault(..) => unreachable!(),
                }
            }
            ResolvedType::Union(
                field_decl_id,
                _,
                Union {
                    owned_name,
                    ref_name_with_lifetime,
                    ..
                },
                relative_namespace,
            ) => {
                let owned_name =
                    format_relative_namespace(&relative_namespace, owned_name).to_string();
                is_copy = false;
                primitive_size = 4;
                vtable_type = format!("::planus::Offset<{owned_name}>");
                match &field.assign_mode {
                    AssignMode::Required => {
                        read_type =
                            format_relative_namespace(&relative_namespace, ref_name_with_lifetime)
                                .to_string();
                        owned_type = owned_name.clone();
                        create_trait = format!("WriteAsUnion<{owned_name}>");
                        if self.infallible_analysis[field_decl_id.0] {
                            try_from_code = format!("::core::convert::Into::into(value.{name}()?)");
                        }
                    }
                    AssignMode::Optional => {
                        read_type = format!(
                            "::core::option::Option<{}>",
                            format_relative_namespace(&relative_namespace, ref_name_with_lifetime)
                        );
                        owned_type = format!("::core::option::Option<{owned_name}>");
                        create_trait = format!("WriteAsOptionalUnion<{owned_name}>");
                        if self.infallible_analysis[field_decl_id.0] {
                            try_from_code = format!(
                                r#"
                                    value.{name}()?.map(::core::convert::Into::into)
                                "#
                            );
                        }
                    }
                    AssignMode::HasDefault(..) => unreachable!(),
                }
            }
            ResolvedType::Enum(_, decl, info, relative_namespace, variants) => {
                vtable_type =
                    format_relative_namespace(&relative_namespace, &info.name).to_string();
                is_copy = true;
                primitive_size = decl.type_.byte_size();
                match &field.assign_mode {
                    AssignMode::HasDefault(Literal::EnumTag { variant_index, .. }) => {
                        read_type = vtable_type.clone();
                        owned_type = vtable_type.clone();
                        create_trait = format!("WriteAsDefault<{owned_type}, {owned_type}>");

                        impl_default_code =
                            format!("{}::{}", owned_type, variants[*variant_index].name).into();
                        serialize_default = Some(
                            format!("&{}::{}", owned_type, variants[*variant_index].name).into(),
                        );
                        deserialize_default = Some(impl_default_code.clone());
                    }
                    AssignMode::Optional => {
                        read_type = format!("::core::option::Option<{vtable_type}>");
                        owned_type = read_type.clone();
                        create_trait = format!("WriteAsOptional<{vtable_type}>");
                    }
                    AssignMode::HasDefault(..) => todo!(),
                    AssignMode::Required => todo!(),
                }
            }
            ResolvedType::Vector(type_) => {
                fn vector_offset_type<'a>(type_: &ResolvedType<'a, RustBackend>) -> Cow<'a, str> {
                    match type_ {
                        ResolvedType::Struct(_, _, info, relative_namespace) => {
                            format_relative_namespace(relative_namespace, &info.owned_name)
                                .to_string()
                                .into()
                        }
                        ResolvedType::Table(_, _, info, relative_namespace) => format!(
                            "::planus::Offset<{}>",
                            format_relative_namespace(relative_namespace, &info.owned_name)
                        )
                        .into(),
                        ResolvedType::Enum(_, _, info, relative_namespace, _) => {
                            format_relative_namespace(relative_namespace, &info.name)
                                .to_string()
                                .into()
                        }
                        ResolvedType::Union(_, _, info, relative_namespace) => format!(
                            "::planus::Offset<{}>",
                            format_relative_namespace(relative_namespace, &info.owned_name)
                        )
                        .into(),
                        ResolvedType::Vector(_) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::Array(_, _) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::String => "::planus::Offset<str>".into(),
                        ResolvedType::Bool => "bool".into(),
                        ResolvedType::Integer(type_) => integer_type(type_).into(),
                        ResolvedType::Float(type_) => float_type(type_).into(),
                    }
                }
                fn vector_owned_type<'a>(type_: &ResolvedType<'a, RustBackend>) -> Cow<'a, str> {
                    match type_ {
                        ResolvedType::Table(_, _, info, relative_namespace) => {
                            format_relative_namespace(relative_namespace, &info.owned_name)
                                .to_string()
                                .into()
                        }
                        ResolvedType::Struct(_, _, info, relative_namespace) => {
                            format_relative_namespace(relative_namespace, &info.owned_name)
                                .to_string()
                                .into()
                        }

                        ResolvedType::Enum(_, _, info, relative_namespace, _) => {
                            format_relative_namespace(relative_namespace, &info.name)
                                .to_string()
                                .into()
                        }
                        ResolvedType::Union(_, _, info, relative_namespace) => {
                            format_relative_namespace(relative_namespace, &info.owned_name)
                                .to_string()
                                .into()
                        }
                        ResolvedType::Vector(_) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::Array(_, _) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::String => "::planus::alloc::string::String".into(),
                        ResolvedType::Bool => "bool".into(),
                        ResolvedType::Integer(type_) => integer_type(type_).into(),
                        ResolvedType::Float(type_) => float_type(type_).into(),
                    }
                }
                fn vector_read_type(type_: &ResolvedType<'_, RustBackend>) -> String {
                    match type_ {
                        ResolvedType::Struct(_, _, info, relative_namespace) => format!(
                            "::planus::Vector<'a, {}<'a>>",
                            format_relative_namespace(relative_namespace, &info.ref_name)
                        ),
                        ResolvedType::Table(_, _, info, relative_namespace) => format!(
                            "::planus::Vector<'a, ::planus::Result<{}<'a>>>",
                            format_relative_namespace(relative_namespace, &info.ref_name)
                        ),
                        ResolvedType::Enum(_, _, info, relative_namespace, _) => format!(
                            "::planus::Vector<'a, ::core::result::Result<{}, ::planus::errors::UnknownEnumTag>>",
                            format_relative_namespace(relative_namespace, &info.name)
                        ),
                        ResolvedType::Union(_, _, info, relative_namespace) => format!(
                            "::planus::UnionVector<'a, {}<'a>>",
                            format_relative_namespace(relative_namespace, &info.ref_name)
                        ),
                        ResolvedType::Vector(_) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::Array(_, _) => {
                            unreachable!("This should have been rejected in type-check")
                        }
                        ResolvedType::String => {
                            "::planus::Vector<'a, ::planus::Result<&'a ::core::primitive::str>>".into()
                        }
                        ResolvedType::Bool => "::planus::Vector<'a, bool>".into(),
                        ResolvedType::Integer(type_) if matches!(type_, IntegerType::U8 | IntegerType::I8) => format!("&'a [{}]", integer_type(type_)),
                        ResolvedType::Integer(type_) => format!("::planus::Vector<'a, {}>", integer_type(type_)),
                        ResolvedType::Float(type_) => format!("::planus::Vector<'a, {}>", float_type(type_)),
                    }
                }
                fn vector_try_into_func(type_: &ResolvedType<'_, RustBackend>) -> &'static str {
                    match type_ {
                        ResolvedType::Table(..)
                        | ResolvedType::Enum(..)
                        | ResolvedType::Vector(..)
                        | ResolvedType::String => "to_vec_result",
                        _ => "to_vec",
                    }
                }

                let offset_name = vector_offset_type(&type_);
                let read_name = vector_read_type(&type_);
                let owned_name = vector_owned_type(&type_);
                is_copy = false;
                primitive_size = 4;
                vtable_type = format!("::planus::Offset<[{offset_name}]>");

                let is_byte_slice = matches!(
                    &*type_,
                    ResolvedType::Integer(IntegerType::U8) | ResolvedType::Integer(IntegerType::I8)
                );
                try_from_code = match (&field.assign_mode, is_byte_slice) {
                    (AssignMode::Optional, true) => format!("value.{name}()?.map(|v| v.to_vec())"),
                    (_, true) => format!("value.{name}()?.to_vec()"),
                    (AssignMode::Optional, false) => format!(
                        r#"
                            if let ::core::option::Option::Some({name}) = value.{name}()? {{
                                ::core::option::Option::Some({name}.{try_into_func}()?)
                            }} else {{
                                ::core::option::Option::None
                            }}
                        "#,
                        try_into_func = vector_try_into_func(&type_)
                    ),
                    (_, false) => format!(
                        "value.{name}()?.{try_into_func}()?",
                        try_into_func = vector_try_into_func(&type_)
                    ),
                };
                match &field.assign_mode {
                    AssignMode::Required => {
                        read_type = read_name;
                        owned_type = format!("::planus::alloc::vec::Vec<{owned_name}>");
                        if matches!(field.object_tag_kind, TableFieldTagKind::UnionTagVector) {
                            create_trait = format!("WriteAsUnionVector<{owned_name}>");
                        } else {
                            create_trait = format!("WriteAs<{vtable_type}>");
                        }
                    }
                    AssignMode::Optional => {
                        read_type = format!("::core::option::Option<{read_name}>",);
                        owned_type = format!(
                            "::core::option::Option<::planus::alloc::vec::Vec<{owned_name}>>"
                        );
                        if matches!(field.object_tag_kind, TableFieldTagKind::UnionTagVector) {
                            create_trait = format!("WriteAsOptionalUnionVector<{owned_name}>");
                        } else {
                            create_trait = format!("WriteAsOptional<{vtable_type}>");
                        }
                    }
                    AssignMode::HasDefault(Literal::Vector(v)) if v.is_empty() => {
                        read_type = read_name;
                        owned_type = format!("::planus::alloc::vec::Vec<{owned_name}>");
                        if matches!(field.object_tag_kind, TableFieldTagKind::UnionTagVector) {
                            create_trait = format!("WriteAsDefaultUnionVector<{owned_name}>");
                            serialize_default = Some("".into());
                            deserialize_default = Some("::planus::UnionVector::new_empty()".into());
                        } else {
                            create_trait = format!("WriteAsDefault<{vtable_type}, ()>");
                            serialize_default = Some("&()".into());
                            deserialize_default = Some(
                                if is_byte_slice {
                                    "&[]"
                                } else {
                                    "::planus::Vector::new_empty()"
                                }
                                .into(),
                            );
                        }
                    }
                    AssignMode::HasDefault(..) => unreachable!(),
                }
            }
            ResolvedType::Array(_, _) => todo!(),
            ResolvedType::String => {
                is_copy = false;
                primitive_size = 4;
                vtable_type = "::planus::Offset<str>".to_string();
                match &field.assign_mode {
                    AssignMode::Required => {
                        read_type = "&'a ::core::primitive::str".to_string();
                        owned_type = "::planus::alloc::string::String".to_string();
                        create_trait = "WriteAs<::planus::Offset<str>>".to_string();
                        try_from_code = format!("::core::convert::Into::into(value.{name}()?)");
                    }
                    AssignMode::Optional => {
                        read_type =
                            "::core::option::Option<&'a ::core::primitive::str>".to_string();
                        owned_type =
                            "::core::option::Option<::planus::alloc::string::String>".to_string();
                        create_trait =
                            "WriteAsOptional<::planus::Offset<::core::primitive::str>>".to_string();
                        try_from_code = format!(
                            r#"
                                value.{name}()?.map(::core::convert::Into::into)
                            "#
                        );
                    }
                    AssignMode::HasDefault(Literal::String(s)) => {
                        read_type = "&'a ::core::primitive::str".to_string();
                        owned_type = "::planus::alloc::string::String".to_string();
                        create_trait =
                            "WriteAsDefault<::planus::Offset<::core::primitive::str>, ::core::primitive::str>"
                                .to_string();

                        impl_default_code = format!("::core::convert::Into::into({s:?})").into();
                        serialize_default = Some(format!("{s:?}").into());
                        deserialize_default = Some(impl_default_code.clone());
                        try_from_code = format!("::core::convert::Into::into(value.{name}()?)");
                    }
                    AssignMode::HasDefault(..) => unreachable!(),
                }
            }
            ResolvedType::Bool => {
                is_copy = true;
                primitive_size = 1;
                vtable_type = "bool".to_string();
                match &field.assign_mode {
                    AssignMode::HasDefault(Literal::Bool(lit)) => {
                        read_type = "bool".to_string();
                        owned_type = "bool".to_string();
                        create_trait = "WriteAsDefault<bool, bool>".to_string();
                        impl_default_code = format!("{lit}").into();
                        serialize_default = Some(format!("&{lit}").into());
                        deserialize_default = Some(impl_default_code.clone());
                    }
                    AssignMode::Optional => {
                        read_type = "::core::option::Option<bool>".to_string();
                        owned_type = "::core::option::Option<bool>".to_string();
                        create_trait = "WriteAsOptional<bool>".to_string();
                    }
                    AssignMode::HasDefault(..) => todo!(),
                    AssignMode::Required => todo!(),
                }
            }
            ResolvedType::Integer(typ) => {
                is_copy = true;
                primitive_size = typ.byte_size();
                vtable_type = integer_type(&typ).to_string();
                match &field.assign_mode {
                    AssignMode::HasDefault(Literal::Int(lit)) => {
                        read_type = vtable_type.clone();
                        owned_type = vtable_type.clone();
                        create_trait = format!("WriteAsDefault<{owned_type}, {owned_type}>");
                        impl_default_code = format!("{lit}").into();
                        serialize_default = Some(format!("&{lit}").into());
                        deserialize_default = Some(impl_default_code.clone());
                    }
                    AssignMode::Optional => {
                        read_type = format!("::core::option::Option<{vtable_type}>");
                        owned_type = read_type.clone();
                        create_trait = format!("WriteAsOptional<{vtable_type}>");
                    }
                    AssignMode::HasDefault(..) => unreachable!(),
                    AssignMode::Required => todo!(),
                }
            }
            ResolvedType::Float(typ) => {
                is_copy = true;
                primitive_size = typ.byte_size();
                vtable_type = float_type(&typ).to_string();
                match &field.assign_mode {
                    AssignMode::HasDefault(Literal::Float(lit)) => {
                        read_type = vtable_type.clone();
                        owned_type = vtable_type.clone();
                        create_trait = format!("WriteAsDefault<{owned_type}, {owned_type}>");
                        impl_default_code = format!("{lit}").into();
                        serialize_default = Some(format!("&{lit}").into());
                        deserialize_default = Some(impl_default_code.clone());
                    }
                    AssignMode::Optional => {
                        read_type = format!("::core::option::Option<{vtable_type}>");
                        owned_type = read_type.clone();
                        create_trait = format!("WriteAsOptional<{}>", float_type(&typ));
                    }
                    AssignMode::HasDefault(..) => todo!(),
                    AssignMode::Required => todo!(),
                }
            }
        }
        TableField {
            name,
            name_with_as,
            primitive_size,
            vtable_type,
            owned_type,
            read_type,
            create_name,
            create_trait,
            required: matches!(field.assign_mode, AssignMode::Required),
            optional: matches!(field.assign_mode, AssignMode::Optional),
            has_default: matches!(field.assign_mode, AssignMode::HasDefault(..)),
            impl_default_code,
            serialize_default,
            deserialize_default,
            try_from_code,
            is_copy,
        }
    }

    fn generate_struct_field(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        parent_info: &Self::StructInfo,
        _parent: &intermediate::Struct,
        field_name: &str,
        _field: &intermediate::StructField,
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
        let can_do_infallible_conversion;

        match resolved_type {
            ResolvedType::Struct(decl_id, _, info, relative_namespace) => {
                owned_type =
                    format_relative_namespace(&relative_namespace, &info.owned_name).to_string();
                let ref_name = format_relative_namespace(&relative_namespace, &info.ref_name);
                getter_return_type = format!("{ref_name}<'a>");
                getter_code = "::core::convert::From::from(buffer)".to_string();
                can_do_infallible_conversion = self.infallible_analysis[decl_id.0];
            }
            ResolvedType::Enum(_, decl, info, relative_namespace, _) => {
                owned_type = format_relative_namespace(&relative_namespace, &info.name).to_string();
                getter_return_type = format!(
                    "::core::result::Result<{owned_type}, ::planus::errors::UnknownEnumTag>"
                );
                getter_code = format!(
                    r#"let value: ::core::result::Result<{}, _> = ::core::convert::TryInto::try_into({}::from_le_bytes(*buffer.as_array()));
                    value.map_err(|e| e.with_error_location(
                        {:?},
                        {:?},
                        buffer.offset_from_start,
                    ))"#,
                    owned_type,
                    integer_type(&decl.type_),
                    parent_info.ref_name,
                    name
                );
                can_do_infallible_conversion = false;
            }
            ResolvedType::Bool => {
                owned_type = "bool".to_string();
                getter_return_type = owned_type.clone();
                getter_code = "buffer.as_array()[0] != 0".to_string();
                can_do_infallible_conversion = true;
            }
            ResolvedType::Integer(typ) => {
                owned_type = integer_type(&typ).to_string();
                getter_return_type = owned_type.clone();
                getter_code = format!("{owned_type}::from_le_bytes(*buffer.as_array())");
                can_do_infallible_conversion = true;
            }
            ResolvedType::Float(typ) => {
                owned_type = float_type(&typ).to_string();
                getter_return_type = owned_type.clone();
                getter_code = format!("{owned_type}::from_le_bytes(*buffer.as_array())");
                can_do_infallible_conversion = true;
            }
            _ => unreachable!(),
        }
        StructField {
            name,
            owned_type,
            getter_return_type,
            getter_code,
            can_do_infallible_conversion,
        }
    }

    fn generate_enum_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::EnumInfo,
        _parent: &intermediate::Enum,
        key: &str,
        value: &intermediate::IntegerLiteral,
    ) -> EnumVariant {
        let name =
            reserve_rust_enum_variant_name(key, "name", &mut translation_context.declaration_names);

        EnumVariant {
            name,
            value: format!("{value}"),
        }
    }

    fn generate_union_variant(
        &mut self,
        translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::UnionInfo,
        _parent: &intermediate::Union,
        key: &str,
        _index: u8,
        _value: &intermediate::UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> UnionVariant {
        let create_name = reserve_field_name(
            &format!("create_{key}"),
            "create_function",
            &mut translation_context.declaration_names,
        );
        let builder_name = reserve_field_name(
            key,
            "builder_function",
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
        let mut is_struct = false;
        let can_do_infallible_conversion;

        match resolved_type {
            ResolvedType::Table(_, _, info, relative_namespace) => {
                owned_type = format!(
                    "::planus::alloc::boxed::Box<{}>",
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
                can_do_infallible_conversion = false;
            }
            ResolvedType::Struct(decl_id, _, info, relative_namespace) => {
                owned_type =
                    format_relative_namespace(&relative_namespace, &info.owned_name).to_string();
                ref_type = format!(
                    "{}<'a>",
                    format_relative_namespace(&relative_namespace, &info.ref_name)
                );
                create_trait = format!(
                    "WriteAsOffset<{}>",
                    format_relative_namespace(&relative_namespace, &info.owned_name)
                );
                is_struct = true;
                can_do_infallible_conversion = self.infallible_analysis[decl_id.0];
            }
            ResolvedType::String => {
                owned_type = "::planus::alloc::string::String".to_string();
                ref_type = "&'a str".to_string();
                create_trait = "WriteAsOffset<str>".to_string();
                can_do_infallible_conversion = true;
            }
            _ => todo!(),
        }
        UnionVariant {
            create_name,
            enum_name,
            builder_name,
            create_trait,
            owned_type,
            ref_type,
            is_struct,
            can_do_infallible_conversion,
        }
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

pub fn format_string(s: &str, max_width: Option<u64>) -> eyre::Result<String> {
    let mut child = Command::new("rustfmt");

    child
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(max_width) = max_width {
        child.arg("--config");
        child.arg(format!("max_width={max_width}"));
    }

    let mut child = child
        .spawn()
        .wrap_err("Unable to spawn rustfmt. Perhaps it is not installed?")?;

    {
        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin
            .write_all(s.as_bytes())
            .wrap_err("Unable to write the file to rustfmt")?;
    }

    let output = child
        .wait_with_output()
        .wrap_err("Unable to get the formatted file back from rustfmt")?;

    if output.status.success() && output.stderr.is_empty() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else if output.stderr.is_empty() {
        eyre::bail!("rustfmt failed with exit code {}", output.status);
    } else {
        eyre::bail!(
            "rustfmt failed with exit code {} and message:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }
}
