use std::borrow::Cow;

use planus_types::intermediate::{self, AbsolutePath, AssignMode, DeclarationIndex, Literal};

use super::backend::{
    Backend, DeclarationNames, DeclarationTranslationContext, NamespaceNames, ResolvedType,
};

#[derive(Debug, Clone)]
pub struct DotBackend {
    pub color_seed: u64,
}

#[derive(Clone, Debug)]
pub struct Namespace {
    pub is_root: bool,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub decl_id: DeclarationIndex,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct TableField {
    pub name: String,
    pub primitive_size: u32,
    pub type_: Cow<'static, str>,
    pub type_ref: Option<DeclarationIndex>,
    pub assign_mode: Cow<'static, str>,
    pub color: String,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub decl_id: DeclarationIndex,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct StructField {
    pub name: String,
    pub type_: Cow<'static, str>,
    pub type_ref: Option<DeclarationIndex>,
    pub color: String,
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub decl_id: DeclarationIndex,
    pub name: String,
    pub repr_type: &'static str,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct Union {
    pub decl_id: DeclarationIndex,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct UnionVariant {
    pub name: String,
    pub type_: Cow<'static, str>,
    pub type_ref: Option<DeclarationIndex>,
    pub color: String,
}

#[derive(Clone, Debug)]
pub struct RpcService {
    pub decl_id: DeclarationIndex,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct RpcMethod {
    pub name: String,
    pub arg_type: String,
    pub arg_type_ref: Option<DeclarationIndex>,
    pub return_type: String,
    pub return_type_ref: Option<DeclarationIndex>,
}

impl DotBackend {
    fn random_color(&mut self) -> String {
        self.color_seed += 1;
        random_color::RandomColor::new()
            .luminosity(random_color::Luminosity::Bright)
            .seed(self.color_seed)
            .to_hex()
    }
}

fn get_name(type_: &ResolvedType<'_, DotBackend>) -> (Cow<'static, str>, Option<DeclarationIndex>) {
    match type_ {
        ResolvedType::Struct(_, _, Struct { name, decl_id }, _) => {
            (name.clone().into(), Some(*decl_id))
        }
        ResolvedType::Table(_, _, Table { name, decl_id }, _) => {
            (name.clone().into(), Some(*decl_id))
        }
        ResolvedType::Enum(_, _, Enum { name, decl_id, .. }, _, _) => {
            (name.clone().into(), Some(*decl_id))
        }
        ResolvedType::Union(_, _, Union { name, decl_id }, _) => {
            (name.clone().into(), Some(*decl_id))
        }
        ResolvedType::Vector(inner) => {
            let (name, decl_id) = get_name(inner);
            (format!("[{name}]").into(), decl_id)
        }
        ResolvedType::Array(inner, size) => {
            let (name, decl_id) = get_name(inner);
            (format!("[{name}: {size}]").into(), decl_id)
        }
        ResolvedType::String => ("string".into(), None),
        ResolvedType::Bool => ("bool".into(), None),
        ResolvedType::Integer(type_) => (type_.flatbuffer_name().into(), None),
        ResolvedType::Float(type_) => (type_.flatbuffer_name().into(), None),
    }
}

impl Backend for DotBackend {
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

    const KEYWORDS: &'static [&'static str] = &[];

    fn generate_namespace(
        &mut self,
        _namespace_names: &mut NamespaceNames<'_, '_>,
        namespace_name: &AbsolutePath,
        _namespace: &intermediate::Namespace,
    ) -> Namespace {
        Namespace {
            is_root: namespace_name.0.is_empty(),
        }
    }

    fn generate_table(
        &mut self,
        _declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        _decl: &intermediate::Table,
    ) -> Table {
        Table {
            decl_id,
            name: decl_name.to_string(),
        }
    }

    fn generate_struct(
        &mut self,
        _declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        _decl: &intermediate::Struct,
    ) -> Struct {
        let decl_name = decl_name.0.last().unwrap();
        Struct {
            decl_id,
            name: decl_name.to_string(),
        }
    }

    fn generate_enum(
        &mut self,
        _declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        decl: &intermediate::Enum,
    ) -> Enum {
        Enum {
            decl_id,
            name: decl_name.to_string(),
            repr_type: decl.type_.flatbuffer_name(),
        }
    }

    fn generate_union(
        &mut self,
        _declaration_names: &mut DeclarationNames<'_, '_>,
        _translated_namespaces: &[Self::NamespaceInfo],
        decl_id: DeclarationIndex,
        decl_name: &AbsolutePath,
        _decl: &intermediate::Union,
    ) -> Union {
        Union {
            decl_id,
            name: decl_name.to_string(),
        }
    }

    fn generate_table_field(
        &mut self,
        _translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::TableInfo,
        _parent: &intermediate::Table,
        field_name: &str,
        field: &intermediate::TableField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> TableField {
        let (type_, type_ref) = get_name(&resolved_type);

        let primitive_size = match resolved_type {
            ResolvedType::Struct(_, decl, _, _) => decl.size,
            ResolvedType::Table(_, _, _, _) => 4,
            ResolvedType::Union(_, _, _, _) => 4,
            ResolvedType::Enum(_, decl, _, _, _) => decl.type_.byte_size(),
            ResolvedType::Vector(_) => 4,
            ResolvedType::Array(_, _) => todo!(),
            ResolvedType::String => 4,
            ResolvedType::Bool => 1,
            ResolvedType::Integer(typ) => typ.byte_size(),
            ResolvedType::Float(typ) => typ.byte_size(),
        };
        let assign_mode = match (&field.assign_mode, resolved_type) {
            (AssignMode::Required, _) => "required".into(),
            (AssignMode::Optional, _) => "optional".into(),
            (
                AssignMode::HasDefault(Literal::EnumTag { variant_index, .. }),
                ResolvedType::Enum(_, _, _, _, variants),
            ) => format!("default {}", variants[*variant_index].name).into(),
            (AssignMode::HasDefault(default), _) => format!("default {default}").into(),
        };

        TableField {
            name: field_name.to_string(),
            primitive_size,
            type_,
            type_ref,
            assign_mode,
            color: self.random_color(),
        }
    }

    fn generate_struct_field(
        &mut self,
        _translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::StructInfo,
        _parent: &intermediate::Struct,
        field_name: &str,
        _field: &intermediate::StructField,
        resolved_type: ResolvedType<'_, Self>,
    ) -> StructField {
        let (type_, type_ref) = get_name(&resolved_type);

        StructField {
            name: field_name.to_string(),
            type_,
            type_ref,
            color: self.random_color(),
        }
    }

    fn generate_enum_variant(
        &mut self,
        _translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::EnumInfo,
        _parent: &intermediate::Enum,
        key: &str,
        value: &intermediate::IntegerLiteral,
    ) -> EnumVariant {
        EnumVariant {
            name: key.to_string(),
            value: format!("{value}"),
        }
    }

    fn generate_union_variant(
        &mut self,
        _translation_context: &mut DeclarationTranslationContext<'_, '_, Self>,
        _parent_info: &Self::UnionInfo,
        _parent: &intermediate::Union,
        key: &str,
        _index: u8,
        _value: &intermediate::UnionVariant,
        resolved_type: ResolvedType<'_, Self>,
    ) -> UnionVariant {
        let (type_, type_ref) = get_name(&resolved_type);

        UnionVariant {
            name: key.to_string(),
            type_,
            type_ref,
            color: self.random_color(),
        }
    }
}
