use super::{
    Attribute, AttributeKind, BuiltinType, Declaration, Enum, EnumVariant, FloatType,
    IntegerLiteral, IntegerType, Literal, LiteralKind, MetadataValue, NamespacePath, RpcMethod,
    RpcService, Schema, Struct, StructField, Type, Union, UnionVariant,
};
use crate::{
    ast::{TypeDeclarationKind, TypeKind},
    ctx::Ctx,
};

pub trait PrettyPrint {
    fn print(&self, ctx: &Ctx);
}

impl PrettyPrint for Schema {
    fn print(&self, ctx: &Ctx) {
        println!("file {}:", ctx.get_filename(self.file_id).display());
        if !self.native_includes.is_empty() {
            println!(
                "  native includes: {:?}",
                self.native_includes
                    .iter()
                    .map(|include| &include.value)
                    .collect::<Vec<_>>()
            )
        }
        if !self.includes.is_empty() {
            println!(
                "  includes: {:?}",
                self.includes
                    .iter()
                    .map(|include| &include.value)
                    .collect::<Vec<_>>()
            )
        }
        if let Some((_, namespace)) = &self.namespace {
            print!("  namespace: ");
            namespace.print(ctx);
            println!();
        }
        if let Some((_, root_type)) = &self.root_type {
            print!("  root type: ");
            root_type.print(ctx);
            println!();
        }
        if let Some((_, file_extension)) = &self.file_extension {
            println!("  file extension: {}", file_extension.value);
        }
        if let Some((_, file_identifier)) = &self.file_identifier {
            println!("  file identifier: {}", file_identifier.value);
        }
        if !self.attributes.is_empty() {
            println!("  attributes: ");
            let mut first = true;
            for attribute in &self.attributes {
                if first {
                    first = false;
                } else {
                    print!(", ");
                }
                attribute.print(ctx);
            }
            println!();
        }
        for decl in self.type_declarations.values() {
            decl.print(ctx)
        }
    }
}

impl PrettyPrint for NamespacePath {
    fn print(&self, ctx: &Ctx) {
        let mut first = true;
        for &symbol in &self.parts {
            if first {
                first = false;
            } else {
                print!(".");
            }
            print!("{}", ctx.resolve_identifier(symbol));
        }
    }
}

impl PrettyPrint for Type {
    fn print(&self, ctx: &Ctx) {
        match &self.kind {
            TypeKind::Builtin(typ) => typ.print(ctx),
            TypeKind::Vector { inner_type } => {
                print!("[");
                inner_type.print(ctx);
                print!("]");
            }
            TypeKind::Array { inner_type, size } => {
                print!("[");
                inner_type.print(ctx);
                print!("; {}]", size);
            }
            TypeKind::Path(path) => path.print(ctx),
            TypeKind::Invalid => print!("#invalid_type"),
        }
    }
}

impl PrettyPrint for BuiltinType {
    fn print(&self, ctx: &Ctx) {
        match self {
            BuiltinType::Bool => print!("bool"),
            BuiltinType::Integer(typ) => typ.print(ctx),
            BuiltinType::Float(typ) => typ.print(ctx),
            BuiltinType::String => print!("string"),
        }
    }
}

impl PrettyPrint for IntegerType {
    fn print(&self, _ctx: &Ctx) {
        match self {
            IntegerType::U8 => print!("uint8"),
            IntegerType::U16 => print!("uint16"),
            IntegerType::U32 => print!("uint32"),
            IntegerType::U64 => print!("uint64"),
            IntegerType::I8 => print!("int8"),
            IntegerType::I16 => print!("int16"),
            IntegerType::I32 => print!("int32"),
            IntegerType::I64 => print!("int64"),
        }
    }
}

impl PrettyPrint for FloatType {
    fn print(&self, _ctx: &Ctx) {
        match self {
            FloatType::F32 => print!("float32"),
            FloatType::F64 => print!("float64"),
        }
    }
}

impl PrettyPrint for Declaration {
    fn print(&self, ctx: &Ctx) {
        match &self.kind {
            TypeDeclarationKind::Table(decl) => {
                println!("  table {}:", ctx.resolve_identifier(self.identifier.value));
                decl.print(ctx);
            }
            TypeDeclarationKind::Struct(decl) => {
                println!(
                    "  struct {}:",
                    ctx.resolve_identifier(self.identifier.value)
                );
                decl.print(ctx);
            }
            TypeDeclarationKind::Enum(decl) => {
                println!("  enum {}:", ctx.resolve_identifier(self.identifier.value));
                decl.print(ctx);
            }
            TypeDeclarationKind::Union(decl) => {
                println!("  union {}:", ctx.resolve_identifier(self.identifier.value));
                decl.print(ctx);
            }
            TypeDeclarationKind::RpcService(decl) => {
                println!(
                    "  rpc service {}:",
                    ctx.resolve_identifier(self.identifier.value)
                );
                decl.print(ctx);
            }
        }
    }
}

impl PrettyPrint for IntegerLiteral {
    fn print(&self, _ctx: &Ctx) {
        if self.is_negative {
            print!("-{}", self.value)
        } else {
            print!("{}", self.value);
        }
    }
}

impl PrettyPrint for Literal {
    fn print(&self, ctx: &Ctx) {
        match &self.kind {
            LiteralKind::Bool(value) => print!("{}", value),
            LiteralKind::Integer { is_negative, value } => {
                if *is_negative {
                    print!("-{}", value);
                } else {
                    print!("{}", value);
                }
            }
            LiteralKind::Float { is_negative, value } => {
                if *is_negative {
                    print!("-{}", value);
                } else {
                    print!("{}", value);
                }
            }
            LiteralKind::String(s) => print!("{:?}", s),
            LiteralKind::List(l) => {
                print!("[");
                let mut first = true;
                for v in l {
                    if first {
                        first = false;
                    }
                    {
                        print!(", ");
                    }
                    v.print(ctx);
                }
                print!("]");
            }
            LiteralKind::Null => print!("null"),
            LiteralKind::Constant(s) => print!("{}", s),
        }
    }
}

impl PrettyPrint for MetadataValue {
    fn print(&self, ctx: &Ctx) {
        match &self.kind {
            crate::ast::MetadataValueKind::ForceAlign(literal) => {
                print!("force_align: ");
                literal.print(ctx);
            }
            crate::ast::MetadataValueKind::BitFlags => print!("bit_flags"),
            crate::ast::MetadataValueKind::CsharpPartial => print!("csharp_partial"),
            crate::ast::MetadataValueKind::Private => print!("private"),
            crate::ast::MetadataValueKind::NativeType(literal) => {
                print!("native_type: {}", literal.value)
            }
            crate::ast::MetadataValueKind::NativeTypePackName(literal) => {
                print!("native_type_pack_name: {}", literal.value);
            }
            crate::ast::MetadataValueKind::OriginalOrder => print!("original_order"),
            crate::ast::MetadataValueKind::Required => print!("required"),
            crate::ast::MetadataValueKind::Deprecated => print!("deprecated"),
            crate::ast::MetadataValueKind::Key => print!("key"),
            crate::ast::MetadataValueKind::Shared => print!("shared"),
            crate::ast::MetadataValueKind::NestedFlatbuffer(literal) => {
                print!("nested_flatbuffer: {}", literal.value)
            }
            crate::ast::MetadataValueKind::Id(literal) => {
                print!("id: ");
                literal.print(ctx);
            }
            crate::ast::MetadataValueKind::Hash(literal) => print!("hash: {}", literal.value),
            crate::ast::MetadataValueKind::CppType(literal) => {
                print!("cpp_type: {}", literal.value)
            }
            crate::ast::MetadataValueKind::CppPtrType(literal) => {
                print!("cpp_ptr_type: {}", literal.value)
            }
            crate::ast::MetadataValueKind::CppPtrTypeGet(literal) => {
                print!("cpp_ptr_type_get: {}", literal.value)
            }
            crate::ast::MetadataValueKind::CppStrType(literal) => {
                print!("cpp_str_type: {}", literal.value)
            }
            crate::ast::MetadataValueKind::CppStrFlexCtor => {
                print!("cpp_str_flex_ctor");
            }
            crate::ast::MetadataValueKind::NativeInline => {
                print!("native_inline");
            }
            crate::ast::MetadataValueKind::NativeDefault(literal) => {
                print!("native_default: {}", literal.value);
            }
            crate::ast::MetadataValueKind::Flexbuffer => print!("flexbuffer"),
            crate::ast::MetadataValueKind::Streaming(literal) => {
                print!("streaming: {}", literal.value);
            }
            crate::ast::MetadataValueKind::Idempotent => print!("idempotent"),
        }
    }
}

impl PrettyPrint for Vec<MetadataValue> {
    fn print(&self, ctx: &Ctx) {
        let mut first = true;
        for value in self {
            if first {
                first = false;
            } else {
                print!(", ");
            }
            value.print(ctx)
        }
    }
}

impl PrettyPrint for Struct {
    fn print(&self, ctx: &Ctx) {
        if !self.metadata.values.is_empty() {
            print!("    metadata: ");
            self.metadata.values.print(ctx);
            println!();
        }
        for field in self.fields.values() {
            field.print(ctx);
        }
    }
}

impl PrettyPrint for StructField {
    fn print(&self, ctx: &Ctx) {
        print!("    field {}: ", ctx.resolve_identifier(self.ident.value));
        self.type_.print(ctx);
        println!();
        if !self.metadata.values.is_empty() {
            print!("      metadata: ");
            self.metadata.values.print(ctx);
            println!();
        }
    }
}

impl PrettyPrint for Enum {
    fn print(&self, ctx: &Ctx) {
        if !self.metadata.values.is_empty() {
            print!("    metadata: ");
            self.metadata.values.print(ctx);
            println!();
        }
        print!("    type: ");
        self.type_.print(ctx);
        println!();
        for variant in self.variants.values() {
            variant.print(ctx);
        }
    }
}

impl PrettyPrint for EnumVariant {
    fn print(&self, ctx: &Ctx) {
        print!("    variant {}: ", ctx.resolve_identifier(self.ident.value));
        if let Some(value) = &self.value {
            print!(" = ");
            value.print(ctx);
        }
        println!();
    }
}

impl PrettyPrint for Union {
    fn print(&self, ctx: &Ctx) {
        if !self.metadata.values.is_empty() {
            print!("    metadata: ");
            self.metadata.values.print(ctx);
            println!();
        }
        for variant in self.variants.values() {
            variant.print(ctx);
        }
    }
}
impl PrettyPrint for UnionVariant {
    fn print(&self, ctx: &Ctx) {
        if let Some(ident) = self.ident {
            print!("    variant {}: ", ctx.resolve_identifier(ident.value));
        } else {
            print!("    variant: ");
        }
        self.type_.print(ctx);
        println!();
    }
}
impl PrettyPrint for RpcService {
    fn print(&self, ctx: &Ctx) {
        for method in self.methods.values() {
            method.print(ctx);
        }
    }
}

impl PrettyPrint for RpcMethod {
    fn print(&self, ctx: &Ctx) {
        print!("    method {}(", ctx.resolve_identifier(self.ident.value));
        self.argument_type.print(ctx);
        print!(") -> ");
        self.return_type.print(ctx);
        println!();
        if !self.metadata.values.is_empty() {
            print!("      metadata: ");
            self.metadata.values.print(ctx);
        }
        println!();
    }
}

impl PrettyPrint for Attribute {
    fn print(&self, ctx: &Ctx) {
        match &self.kind {
            AttributeKind::Identifier(symbol) => print!("{}", ctx.resolve_identifier(*symbol)),
            AttributeKind::String(s) => print!("{:?}", s),
        }
    }
}
