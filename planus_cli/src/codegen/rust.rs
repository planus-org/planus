use crate::{
    ast::{FloatType, IntegerType},
    codegen::name_generator::Scope,
    intermediate_language::types::{
        self as il, AbsolutePath, Declaration, DeclarationIndex, DeclarationKind, Declarations,
        Enum, IntegerLiteral, Namespace, RpcMethod, SimpleType, StructField, TableField, Type,
        TypeKind, Union, UnionVariant,
    },
};
use askama::Template;
use heck::{CamelCase, SnakeCase};
use std::{
    borrow::Cow,
    fs::File,
    io::{self, Write},
    path::Path,
    process::Command,
};

use super::name_generator::{run_name_generator, NameGenerator, ReservedNames};

#[derive(Copy, Clone)]
struct Ctx<'a> {
    declarations: &'a Declarations,
    _namespace_infos: &'a [RustNamespaceInfo],
    decl_infos: &'a [RustDeclInfo],
    entry_infos: &'a [Vec<RustEntryInfo>],
}

impl<'a> Ctx<'a> {
    fn get_declaration(&self, index: &&DeclarationIndex) -> (&'a AbsolutePath, &'a Declaration) {
        self.declarations.get_declaration(**index)
    }
}

impl<'a, 'b> From<&'b Ctx<'a>> for Ctx<'a> {
    fn from(ctx: &'b Ctx<'a>) -> Self {
        *ctx
    }
}

impl<'a, 'b, 'c> From<&'c &'b Ctx<'a>> for Ctx<'a> {
    fn from(ctx: &'c &'b Ctx<'a>) -> Self {
        **ctx
    }
}

#[derive(Template)]
#[template(path = "namespace.template", escape = "none")]
struct RustNamespace<'a> {
    name: &'a AbsolutePath,
    info: &'a RustNamespaceInfo,
    namespace: &'a Namespace,
    ctx: Ctx<'a>,
}

#[derive(Template)]
#[template(path = "struct.template", escape = "none")]
struct RustStruct<'a> {
    name: &'a AbsolutePath,
    decl: &'a il::Struct,
    info: &'a RustDeclInfo,
    field_infos: &'a [RustEntryInfo],
    _ctx: Ctx<'a>,
}

impl<'a> RustStruct<'a> {
    fn new(
        index: impl Into<DeclarationIndex>,
        name: &'a AbsolutePath,
        decl: &'a il::Struct,
        ctx: impl Into<Ctx<'a>>,
    ) -> Self {
        let ctx = ctx.into();
        let index = index.into();
        Self {
            name,
            decl,
            _ctx: ctx,
            field_infos: &ctx.entry_infos[index.0],
            info: &ctx.decl_infos[index.0],
        }
    }
}

#[derive(Template)]
#[template(path = "table.template", escape = "none")]
struct RustTable<'a> {
    name: &'a AbsolutePath,
    decl: &'a il::Table,
    info: &'a RustDeclInfo,
    field_infos: &'a [RustEntryInfo],
    _ctx: Ctx<'a>,
}

impl<'a> RustTable<'a> {
    fn fields(&self) -> impl Iterator<Item = (&il::TableField, &RustEntryInfo)> {
        self.decl.fields.values().zip(self.field_infos.iter())
    }

    fn fields_in_alignment_order(&self) -> impl Iterator<Item = (&il::TableField, &RustEntryInfo)> {
        self.decl
            .alignment_order
            .iter()
            .map(move |&i| (&self.decl.fields[i], &self.field_infos[i]))
    }
    fn new(
        index: impl Into<DeclarationIndex>,
        name: &'a AbsolutePath,
        decl: &'a il::Table,
        ctx: impl Into<Ctx<'a>>,
    ) -> Self {
        let ctx = ctx.into();
        let index = index.into();
        Self {
            name,
            decl,
            _ctx: ctx,
            field_infos: &ctx.entry_infos[index.0],
            info: &ctx.decl_infos[index.0],
        }
    }
}

#[derive(Template)]
#[template(path = "union.template", escape = "none")]
struct RustUnion<'a> {
    info: &'a RustDeclInfo,
    variant_infos: &'a [RustEntryInfo],
    _ctx: Ctx<'a>,
}

impl<'a> RustUnion<'a> {
    fn new(index: impl Into<DeclarationIndex>, ctx: impl Into<Ctx<'a>>) -> Self {
        let ctx = ctx.into();
        let index = index.into();
        Self {
            _ctx: ctx,
            variant_infos: &ctx.entry_infos[index.0],
            info: &ctx.decl_infos[index.0],
        }
    }
}

#[derive(Template)]
#[template(path = "enum.template", escape = "none")]
struct RustEnum<'a> {
    decl: &'a il::Enum,
    info: &'a RustDeclInfo,
    variant_infos: &'a [RustEntryInfo],
}

impl<'a> RustEnum<'a> {
    fn new(
        index: impl Into<DeclarationIndex>,
        decl: &'a il::Enum,
        ctx: impl Into<Ctx<'a>>,
    ) -> Self {
        let ctx = ctx.into();
        let index = index.into();
        Self {
            decl,
            variant_infos: &ctx.entry_infos[index.0],
            info: &ctx.decl_infos[index.0],
        }
    }
}

pub struct Codegen {
    output_filename: String,
    file: File,
}

impl Codegen {
    pub fn new(output_filename: String) -> io::Result<Self> {
        let file = File::create(&output_filename)?;
        Ok(Self {
            file,
            output_filename,
        })
    }
}

impl Codegen {
    pub fn generate_code(&mut self, declarations: &Declarations) -> io::Result<()> {
        // let mod_name: Option<String> = namespace.name.0.first().map(|s| to_snake_case(s));
        let (namespace_infos, decl_infos, entry_infos) =
            run_name_generator(&mut RustNameGenerator, declarations);
        let (index, namespace) = declarations.get_root_namespace();
        let res = RustNamespace {
            name: &AbsolutePath::ROOT_PATH,
            info: &namespace_infos[index.0],
            namespace,
            ctx: Ctx {
                declarations,
                _namespace_infos: &namespace_infos,
                decl_infos: &decl_infos,
                entry_infos: &entry_infos,
            },
        }
        .render()
        .unwrap();
        self.file.write_all(res.as_bytes())?;
        self.file.flush()?;

        format_file(&self.output_filename)?;
        Ok(())
    }
}

struct RustNameGenerator;

const BINDING_KIND_TYPES: &str = "types";

#[derive(Debug)]
struct RustDeclInfo {
    owned_type: Cow<'static, str>,
    read_type: Cow<'static, str>,
    read_type_no_lifetime: Cow<'static, str>,
    repr_type: Cow<'static, str>,
}

#[derive(Debug)]
struct RustEntryInfo {
    owned_field_name: Cow<'static, str>,
    owned_field_type: Cow<'static, str>,
    read_name: Cow<'static, str>,
    read_type: Cow<'static, str>,
    read_type_no_lifetime: Cow<'static, str>,
    create_name: Cow<'static, str>,
    create_type: Cow<'static, str>,
    value: Cow<'static, str>,
    is_union: bool,
    is_string: bool,
}

#[derive(Debug)]
struct RustNamespaceInfo {
    name: Cow<'static, str>,
}

fn module_name<'a>(path: &'a AbsolutePath, reserved_names: &mut ReservedNames) -> Cow<'a, str> {
    if path.0.is_empty() {
        "".into()
    } else {
        let name = path.0.last().unwrap().to_snake_case().into();
        reserved_names.try_reserve_repeat(Scope::Namespace, BINDING_KIND_TYPES, name, '_')
    }
}

fn type_name<'a>(path: &'a AbsolutePath, reserved_names: &mut ReservedNames) -> Cow<'a, str> {
    let name = path.0.last().unwrap().to_camel_case().into();
    reserved_names.try_reserve_repeat(Scope::Namespace, BINDING_KIND_TYPES, name, '_')
}

fn integer_type(ityp: &IntegerType) -> &'static str {
    match &ityp {
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

fn owned_type(decl_infos: &[RustDeclInfo], type_: &Type, box_it: bool) -> Cow<'static, str> {
    match &type_.kind {
        TypeKind::Table(index) | TypeKind::Union(index) => {
            if box_it {
                format!("Box<self::{}>", decl_infos[index.0].owned_type).into()
            } else {
                format!("self::{}", decl_infos[index.0].owned_type).into()
            }
        }
        TypeKind::Vector(inner) => format!("Vec<{}>", owned_type(decl_infos, inner, false)).into(),
        TypeKind::Array(inner, size) => {
            format!("[{}; {}]", owned_type(decl_infos, inner, false), size).into()
        }
        TypeKind::SimpleType(typ) => owned_simple_type(decl_infos, typ),
        TypeKind::String => "String".into(),
    }
}

fn create_type(decl_infos: &[RustDeclInfo], type_: &Type) -> Cow<'static, str> {
    match &type_.kind {
        TypeKind::Table(index) => {
            format!("planus::Offset<self::{}>", decl_infos[index.0].owned_type).into()
        }
        TypeKind::Union(index) => format!("self::{}", decl_infos[index.0].owned_type).into(),
        TypeKind::Vector(typ) => {
            format!("planus::Offset<[{}]>", create_type(decl_infos, typ)).into()
        }
        TypeKind::Array(_typ, _size) => todo!(),
        TypeKind::SimpleType(styp) => owned_simple_type(decl_infos, styp),
        TypeKind::String => "planus::Offset<str>".into(),
    }
}
fn owned_simple_type(decl_infos: &[RustDeclInfo], type_: &SimpleType) -> Cow<'static, str> {
    match type_ {
        SimpleType::Struct(index) | SimpleType::Enum(index) => {
            format!("self::{}", decl_infos[index.0].owned_type).into()
        }
        SimpleType::Bool => "bool".into(),
        SimpleType::Integer(typ) => integer_type(typ).into(),
        SimpleType::Float(typ) => float_type(typ).into(),
    }
}

fn table_read_type(decl_infos: &[RustDeclInfo], type_: &Type) -> Cow<'static, str> {
    match &type_.kind {
        TypeKind::Table(index) | TypeKind::Union(index) => decl_infos[index.0].read_type.clone(),
        TypeKind::Vector(typ) => {
            format!("planus::Vector<'buf, {}>", table_read_type(decl_infos, typ)).into()
        }
        TypeKind::Array(_, _) => todo!(),
        TypeKind::SimpleType(typ) => match typ {
            SimpleType::Enum(index) => format!("self::{}", decl_infos[index.0].read_type).into(),
            SimpleType::Struct(index) => format!("self::{}", decl_infos[index.0].read_type).into(),
            SimpleType::Integer(typ) => integer_type(typ).into(),
            SimpleType::Float(typ) => float_type(typ).into(),
            SimpleType::Bool => "bool".into(),
        },
        TypeKind::String => "std::borrow::Cow<'buf, str>".into(),
    }
}

fn struct_read_type(decl_infos: &[RustDeclInfo], type_: &SimpleType) -> Cow<'static, str> {
    match type_ {
        SimpleType::Enum(index) => {
            format!("planus::Result<self::{}>", decl_infos[index.0].read_type).into()
        }
        SimpleType::Struct(index) => format!("self::{}", decl_infos[index.0].read_type).into(),
        SimpleType::Integer(typ) => format!("{:?}", typ).to_lowercase().into(),
        SimpleType::Float(typ) => format!("{:?}", typ).to_lowercase().into(),
        SimpleType::Bool => "bool".into(),
    }
}

impl NameGenerator for RustNameGenerator {
    type NamespaceInfo = RustNamespaceInfo;
    type DeclInfo = RustDeclInfo;
    type EntryInfo = RustEntryInfo;

    const KEYWORDS: &'static [&'static str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "Self", "self", "static", "struct", "super", "trait",
        "true", "type", "union", "unsafe", "use", "where", "while", "abstract", "become", "box",
        "do", "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
    ];

    fn generate_table(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        _decl: &il::Table,
    ) -> Self::DeclInfo {
        let name = type_name(decl_name, reserved_names);
        RustDeclInfo {
            owned_type: format!("{}", name).into(),
            read_type: format!("{}Ref<'buf>", name).into(),
            read_type_no_lifetime: format!("{}Ref", name).into(),
            repr_type: "".into(),
        }
    }

    fn generate_struct(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        _decl: &il::Struct,
    ) -> Self::DeclInfo {
        let name = type_name(decl_name, reserved_names);
        RustDeclInfo {
            owned_type: format!("{}", name).into(),
            read_type: format!("{}Ref<'buf>", name).into(),
            read_type_no_lifetime: format!("{}Ref", name).into(),
            repr_type: "".into(),
        }
    }

    fn generate_enum(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &Enum,
    ) -> Self::DeclInfo {
        let name = type_name(decl_name, reserved_names);
        RustDeclInfo {
            owned_type: format!("{}", name).into(),
            read_type: format!("{}", name).into(),
            read_type_no_lifetime: format!("{}", name).into(),
            repr_type: format!("{:?}", decl.type_).to_lowercase().into(),
        }
    }

    fn generate_union(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        _decl: &Union,
    ) -> Self::DeclInfo {
        let name = type_name(decl_name, reserved_names);
        RustDeclInfo {
            owned_type: format!("{}", name).into(),
            read_type: format!("{}Ref<'buf>", name).into(),
            read_type_no_lifetime: format!("{}Ref", name).into(),
            repr_type: "".into(),
        }
    }

    fn generate_rpc_service(
        &mut self,
        _reserved_names: &mut ReservedNames,
        _decl_name: &AbsolutePath,
        _decl: &il::RpcService,
    ) -> Self::DeclInfo {
        todo!()
    }

    fn generate_table_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        _decl_name: &AbsolutePath,
        _decl: &il::Table,
        field_name: &str,
        field: &TableField,
    ) -> Self::EntryInfo {
        let snake_field_name = field_name.to_snake_case();
        // Reserve the name buffer as we need it as an argument
        reserved_names.try_reserve(Scope::Declaration, "create_name", "buffer");

        let name = reserved_names.try_reserve_repeat(
            Scope::Declaration,
            "",
            snake_field_name.clone().into(),
            '_',
        );
        let create_name = reserved_names.try_reserve_repeat(
            Scope::Declaration,
            "create_name",
            snake_field_name.into(),
            '_',
        );

        RustEntryInfo {
            owned_field_name: name.clone(),
            owned_field_type: owned_type(decl_infos, &field.type_, true),
            read_name: name.clone(),
            read_type: table_read_type(decl_infos, &field.type_),
            read_type_no_lifetime: "".into(),
            create_name,
            create_type: create_type(decl_infos, &field.type_),
            value: "".into(),
            is_union: matches!(field.type_.kind, il::TypeKind::Union(_)),
            is_string: matches!(field.type_.kind, il::TypeKind::String),
        }
    }

    fn generate_struct_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        _decl_name: &AbsolutePath,
        _decl: &il::Struct,
        field_name: &str,
        field: &StructField,
    ) -> Self::EntryInfo {
        let field_name = field_name.to_snake_case();
        let name =
            reserved_names.try_reserve_repeat(Scope::Declaration, "", field_name.into(), '_');

        RustEntryInfo {
            owned_field_name: name.clone(),
            owned_field_type: owned_simple_type(decl_infos, &field.type_),
            read_name: name.clone(),
            read_type: struct_read_type(decl_infos, &field.type_),
            read_type_no_lifetime: if let SimpleType::Struct(index) = &field.type_ {
                decl_infos[index.0].read_type_no_lifetime.clone()
            } else {
                "".into()
            },
            create_name: "".into(),
            create_type: "".into(),
            value: "".into(),
            is_union: false,
            is_string: false,
        }
    }

    fn generate_enum_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        _decl_infos: &[Self::DeclInfo],
        _decl_name: &AbsolutePath,
        _decl: &Enum,
        key: &str,
        value: &IntegerLiteral,
    ) -> Self::EntryInfo {
        let field_name = key.to_camel_case();
        let name =
            reserved_names.try_reserve_repeat(Scope::Declaration, "", field_name.into(), '_');
        RustEntryInfo {
            owned_field_name: name.clone(),
            owned_field_type: "".into(),
            read_name: name.clone(),
            read_type: "".into(),
            read_type_no_lifetime: "".into(),
            create_name: "".into(),
            create_type: "".into(),
            value: format!("{}", value).into(),
            is_union: false,
            is_string: false,
        }
    }

    fn generate_union_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        _decl_name: &AbsolutePath,
        _decl: &il::Union,
        key: &str,
        variant: &UnionVariant,
    ) -> Self::EntryInfo {
        let name = key.to_camel_case();
        let name = reserved_names.try_reserve_repeat(Scope::Declaration, "", name.into(), '_');

        let create_name = format!("create_{}", key.to_snake_case());
        let create_name = reserved_names.try_reserve_repeat(
            Scope::Declaration,
            "create",
            create_name.into(),
            '_',
        );

        RustEntryInfo {
            owned_field_name: name.clone(),
            owned_field_type: owned_type(decl_infos, &variant.type_, false),
            read_name: name.clone(),
            read_type: table_read_type(decl_infos, &variant.type_),
            read_type_no_lifetime: "".into(),
            create_name,
            create_type: create_type(decl_infos, &variant.type_),
            value: "".into(),
            is_union: false,
            is_string: false,
        }
    }

    fn generate_rpc_method(
        &mut self,
        _reserved_names: &mut ReservedNames,
        _decl_infos: &[Self::DeclInfo],
        _decl_name: &AbsolutePath,
        _decl: &il::RpcService,
        _method_name: &str,
        _method: &RpcMethod,
    ) -> Self::EntryInfo {
        todo!()
    }

    fn generate_namespace(
        &mut self,
        reserved_names: &mut ReservedNames,
        namespace_name: &AbsolutePath,
        _namespace: &Namespace,
    ) -> Self::NamespaceInfo {
        RustNamespaceInfo {
            name: module_name(namespace_name, reserved_names)
                .into_owned()
                .into(),
        }
    }
}

pub fn format_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let output = Command::new("rustfmt")
        .args(&[path.as_ref().as_os_str()])
        .output()?;

    if !output.stderr.is_empty() {
        println!("{}", String::from_utf8(output.stderr).unwrap());
    }

    Ok(())
}
