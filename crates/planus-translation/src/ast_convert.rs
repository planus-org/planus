use std::collections::{hash_map, HashMap};

use codespan::{FileId, Span};
use codespan_reporting::diagnostic::Label;
use indexmap::{map::Entry, IndexMap};
use planus_lexer::{Comment, CommentKind, TokenMetadata};
use planus_types::{ast::*, cst};

use crate::{ctx::Ctx, error::ErrorKind, pretty_print::PrettyPrinter};

struct CstConverter<'ctx> {
    pub schema: Schema,
    pub ctx: &'ctx Ctx,
    pub current_span: Span,
    pub allow_outer_docstrings: bool,
}

pub fn convert(ctx: &Ctx, file_id: FileId, schema: cst::Schema<'_>) -> Schema {
    let location = format!("* File `{}`", ctx.get_filename(file_id).display());
    let mut converter = CstConverter {
        schema: Schema::new(file_id, location),
        ctx,
        current_span: schema.span,
        allow_outer_docstrings: true,
    };
    for decl in &schema.declarations {
        converter.convert_declaration(decl);
    }
    converter.handle_invalid_docstrings(&schema.end_of_stream.token_metadata);
    converter.schema
}

impl<'ctx> CstConverter<'ctx> {
    fn emit_error(
        &self,
        error_type: ErrorKind,
        labels: impl IntoIterator<Item = Label<FileId>>,
        msg: Option<&str>,
    ) {
        self.ctx.emit_error(error_type, labels, msg);
    }

    fn emit_simple_error<T>(&mut self, error_type: ErrorKind, msg: &str) -> Option<T> {
        self.ctx
            .emit_simple_error(error_type, self.schema.file_id, self.current_span, msg);
        None
    }

    fn with_span<F, R>(&mut self, span: Span, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let saved_span = self.current_span;
        self.current_span = span;
        let result = f(self);
        self.current_span = saved_span;
        result
    }

    pub fn convert_expr(&mut self, value: &cst::Expr<'_>) -> Option<Literal> {
        self.with_span(value.span, |self_| {
            Some(Literal {
                span: value.span,
                kind: self_.convert_expr_kind(&value.kind)?,
            })
        })
    }

    pub fn convert_expr_kind(&mut self, value: &cst::ExprKind<'_>) -> Option<LiteralKind> {
        match value {
            cst::ExprKind::Ident(lit) => match lit.ident {
                "inf" | "infinity" => Some(LiteralKind::Float {
                    value: "inf".to_string(),
                    is_negative: false,
                }),
                "nan" | "NaN" => Some(LiteralKind::Float {
                    value: "nan".to_string(),
                    is_negative: false,
                }),
                "true" => Some(LiteralKind::Bool(true)),
                "false" => Some(LiteralKind::Bool(false)),
                "null" => Some(LiteralKind::Null),
                s => Some(LiteralKind::Constant(s.to_string())),
            },
            cst::ExprKind::Integer(lit) => Some(LiteralKind::Integer {
                is_negative: false,
                value: lit.value.to_string(),
            }),
            cst::ExprKind::Float(lit) => Some(LiteralKind::Float {
                is_negative: false,
                value: lit.value.to_string(),
            }),
            cst::ExprKind::String(lit) => Some(LiteralKind::String(lit.value.clone())),
            cst::ExprKind::List(lit) => {
                let mut values = Some(Vec::new());
                for value in &lit.values {
                    if let Some(value) = self.convert_expr(&value.expr) {
                        if let Some(out) = &mut values {
                            out.push(value)
                        }
                    } else {
                        values = None;
                    }
                }

                values.map(LiteralKind::List)
            }
            cst::ExprKind::Signed { sign, inner } => match self.convert_expr(inner)?.kind {
                LiteralKind::Integer { is_negative, value } => {
                    let is_negative = is_negative ^ sign.is_negative;
                    Some(LiteralKind::Integer { is_negative, value })
                }
                LiteralKind::Float { is_negative, value } => {
                    let is_negative = is_negative ^ sign.is_negative;
                    Some(LiteralKind::Float { is_negative, value })
                }
                LiteralKind::Bool(_) => self
                    .emit_simple_error(ErrorKind::TYPE_ERROR, "Cannot use prefix sign on booleans"),
                LiteralKind::String(_) => self
                    .emit_simple_error(ErrorKind::TYPE_ERROR, "Cannot use prefix sign on strings"),
                LiteralKind::List(_) => {
                    self.emit_simple_error(ErrorKind::TYPE_ERROR, "Cannot use prefix sign on lists")
                }
                LiteralKind::Null => {
                    self.emit_simple_error(ErrorKind::TYPE_ERROR, "Cannot use prefix sign on null")
                }
                LiteralKind::Constant(_) => self.emit_simple_error(
                    ErrorKind::TYPE_ERROR,
                    "Cannot use prefix sign on constants",
                ),
            },
        }
    }

    pub fn convert_expr_to_integer_literal(
        &mut self,
        expr: &cst::Expr<'_>,
    ) -> Option<IntegerLiteral> {
        self.with_span(expr.span, |self_| {
            match self_.convert_expr_kind(&expr.kind)? {
                LiteralKind::Integer { is_negative, value } => Some(IntegerLiteral {
                    span: expr.span,
                    is_negative,
                    value,
                }),
                LiteralKind::Bool(_)
                | LiteralKind::Float { .. }
                | LiteralKind::String(_)
                | LiteralKind::Null
                | LiteralKind::Constant(_)
                | LiteralKind::List(_) => {
                    self_.emit_simple_error(ErrorKind::TYPE_ERROR, "expecting integer literal")
                }
            }
        })
    }

    pub fn convert_type(&mut self, type_: &cst::Type<'_>) -> Type {
        self.with_span(type_.span, |self_| Type {
            span: type_.span,
            kind: self_.convert_type_kind(&type_.kind),
        })
    }

    pub fn convert_type_to_integer_type(&mut self, type_: &cst::Type<'_>) -> Option<IntegerType> {
        self.with_span(type_.span, |self_| {
            match self_.convert_type_kind(&type_.kind) {
                TypeKind::Builtin(BuiltinType::Integer(type_)) => Some(type_),
                _ => self_
                    .emit_simple_error(ErrorKind::TYPE_ERROR, "expected a primitive integer type"),
            }
        })
    }

    pub fn convert_type_kind(&mut self, type_kind: &cst::TypeKind<'_>) -> TypeKind {
        match type_kind {
            cst::TypeKind::Vector(typ) => {
                let inner_type = self.convert_type(&typ.inner_type);
                TypeKind::Vector {
                    inner_type: Box::new(inner_type),
                }
            }
            cst::TypeKind::Array(typ) => {
                let inner_type = self.convert_type(&typ.inner_type);
                self.with_span(typ.size.span, |self_| {
                    let size = match self_.convert_expr_to_integer_literal(&typ.size) {
                        None => 1,
                        Some(size) => match integer_literal_to_u32(&size) {
                            Ok(size) => size,
                            Err(NumericalParseError::Overflow) => {
                                self_.emit_simple_error::<()>(
                                    ErrorKind::NUMERICAL_RANGE_ERROR,
                                    "size is too large",
                                );
                                1
                            }
                            Err(NumericalParseError::Underflow) => {
                                self_.emit_simple_error::<()>(
                                    ErrorKind::NUMERICAL_RANGE_ERROR,
                                    "size cannot be negative",
                                );
                                1
                            }
                            Err(NumericalParseError::Format) => {
                                self_.emit_simple_error::<()>(
                                    ErrorKind::NUMERICAL_PARSE_ERROR,
                                    "invalid integer",
                                );
                                1
                            }
                        },
                    };
                    TypeKind::Array {
                        inner_type: Box::new(inner_type),
                        size,
                    }
                })
            }
            cst::TypeKind::Path(path) => {
                let bultin_type = if path.initial_segments.is_empty() {
                    match path.final_segment.ident {
                        "bool" => Some(BuiltinType::Bool),
                        "uint8" | "ubyte" => Some(BuiltinType::Integer(IntegerType::U8)),
                        "uint16" | "ushort" => Some(BuiltinType::Integer(IntegerType::U16)),
                        "uint32" | "uint" => Some(BuiltinType::Integer(IntegerType::U32)),
                        "uint64" | "ulong" => Some(BuiltinType::Integer(IntegerType::U64)),
                        "int8" | "byte" => Some(BuiltinType::Integer(IntegerType::I8)),
                        "int16" | "short" => Some(BuiltinType::Integer(IntegerType::I16)),
                        "int32" | "int" => Some(BuiltinType::Integer(IntegerType::I32)),
                        "int64" | "long" => Some(BuiltinType::Integer(IntegerType::I64)),
                        "float32" | "float" => Some(BuiltinType::Float(FloatType::F32)),
                        "float64" | "double" => Some(BuiltinType::Float(FloatType::F64)),
                        "string" => Some(BuiltinType::String),
                        _ => None,
                    }
                } else {
                    None
                };
                if let Some(builtin_type) = bultin_type {
                    TypeKind::Builtin(builtin_type)
                } else {
                    TypeKind::Path(self.convert_namespace_path(path))
                }
            }
        }
    }

    fn convert_namespace_path(&mut self, path: &cst::NamespacePath<'_>) -> NamespacePath {
        let mut parts = Vec::with_capacity(path.initial_segments.len() + 1);
        for segment in &path.initial_segments {
            parts.push(self.ctx.intern(segment.ident.ident));
        }
        parts.push(self.ctx.intern(path.final_segment.ident));
        NamespacePath {
            span: path.span,
            parts,
        }
    }

    fn check_include(&mut self) {
        if let Some((namespace_span, _)) = self.schema.namespace {
            self.emit_error(
                ErrorKind::FILE_ORDER,
                [
                    Label::secondary(self.schema.file_id, namespace_span)
                        .with_message("namespace was here"),
                    Label::secondary(self.schema.file_id, self.current_span)
                        .with_message("include was here"),
                ],
                Some("Cannot have includes after the namespace has been set"),
            );
        } else if let Some((_, type_declaration)) = self.schema.type_declarations.first() {
            self.emit_error(
                ErrorKind::FILE_ORDER,
                [
                    Label::secondary(self.schema.file_id, type_declaration.full_span)
                        .with_message("type declaration was here"),
                    Label::secondary(self.schema.file_id, self.current_span)
                        .with_message("include was here"),
                ],
                Some("Cannot have includes after the first type declaration"),
            );
        }
    }

    fn check_namespace(&mut self) {
        if let Some((namespace_span, _)) = self.schema.namespace {
            self.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [
                    Label::secondary(self.schema.file_id, namespace_span)
                        .with_message("first declaration was here"),
                    Label::secondary(self.schema.file_id, self.current_span)
                        .with_message("additional declaration was here"),
                ],
                Some("Cannot set namespace twice"),
            );
        } else if let Some((_, type_declaration)) = self.schema.type_declarations.first() {
            self.emit_error(
                ErrorKind::FILE_ORDER,
                [
                    Label::secondary(self.schema.file_id, type_declaration.full_span)
                        .with_message("type declaration was here"),
                    Label::secondary(self.schema.file_id, self.current_span)
                        .with_message("namespace declaration was here"),
                ],
                Some("Cannot have namespace declaration after the first type declaration"),
            );
        }
    }

    fn check_root_type(&mut self) {
        if let Some((root_type_span, _)) = self.schema.root_type {
            self.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [
                    Label::primary(self.schema.file_id, root_type_span)
                        .with_message("first declaration was here"),
                    Label::primary(self.schema.file_id, self.current_span)
                        .with_message("additional declaration was here"),
                ],
                Some("Cannot set root_type twice"),
            );
        }
    }

    fn check_file_extension(&mut self) {
        if let Some((file_extension_span, _)) = self.schema.file_extension {
            self.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [
                    Label::primary(self.schema.file_id, file_extension_span)
                        .with_message("first declaration was here"),
                    Label::primary(self.schema.file_id, self.current_span)
                        .with_message("additional declaration was here"),
                ],
                Some("Cannot set file_extension twice"),
            );
        }
    }

    fn check_file_identifier(&mut self) {
        if let Some((file_identifier_span, _)) = self.schema.file_identifier {
            self.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [
                    Label::secondary(self.schema.file_id, file_identifier_span)
                        .with_message("first declaration was here"),
                    Label::primary(self.schema.file_id, self.current_span)
                        .with_message("additional declaration was here"),
                ],
                Some("Cannot set file_identifier twice"),
            );
        }
    }

    fn convert_declaration_kind(
        &mut self,
        declaration: &cst::DeclarationKind<'_>,
    ) -> Option<Declaration> {
        match declaration {
            cst::DeclarationKind::Include(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_invalid_docstrings(&decl.path.token_metadata);
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_include();
                let lit = self.convert_string_literal(&decl.path);
                self.schema.includes.push(lit);
                None
            }
            cst::DeclarationKind::NativeInclude(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_invalid_docstrings(&decl.path.token_metadata);
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_include();
                let lit = self.convert_string_literal(&decl.path);
                self.schema.native_includes.push(lit);
                None
            }
            cst::DeclarationKind::Namespace(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_many_invalid_docstrings(decl.namespace.token_metas());
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_namespace();
                if self.schema.namespace.is_none() {
                    self.schema.namespace = Some((
                        decl.keyword.span.merge(decl.semicolon.span),
                        self.convert_namespace_path(&decl.namespace),
                    ));
                }
                None
            }
            cst::DeclarationKind::RootType(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_many_invalid_docstrings(decl.root_type.kind.token_metas());
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_root_type();
                if self.schema.root_type.is_none() {
                    self.schema.root_type = Some((
                        decl.keyword.span.merge(decl.semicolon.span),
                        self.convert_type(&decl.root_type),
                    ));
                }
                None
            }
            cst::DeclarationKind::FileExtension(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_invalid_docstrings(&decl.file_extension.token_metadata);
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_file_extension();
                if self.schema.file_extension.is_none() {
                    self.schema.file_extension = Some((
                        decl.keyword.span.merge(decl.semicolon.span),
                        self.convert_string_literal(&decl.file_extension),
                    ));
                }
                None
            }
            cst::DeclarationKind::FileIdentifier(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_invalid_docstrings(&decl.file_identifier.token_metadata);
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                self.check_file_identifier();
                if self.schema.file_identifier.is_none() {
                    self.schema.file_identifier = Some((
                        decl.keyword.span.merge(decl.semicolon.span),
                        self.convert_string_literal(&decl.file_identifier),
                    ));
                }
                None
            }
            cst::DeclarationKind::Attribute(decl) => {
                self.handle_invalid_docstrings(&decl.keyword.token_metadata);
                self.handle_invalid_docstrings(decl.attribute.token_meta());
                self.handle_invalid_docstrings(&decl.semicolon.token_metadata);

                let attribute = self.convert_attribute(&decl.attribute);
                self.schema.attributes.push(attribute);
                None
            }
            cst::DeclarationKind::Table(decl) => Some(self.convert_table(decl)),
            cst::DeclarationKind::Struct(decl) => Some(self.convert_struct(decl)),
            cst::DeclarationKind::Enum(decl) => Some(self.convert_enum(decl)),
            cst::DeclarationKind::Union(decl) => Some(self.convert_union(decl)),
            cst::DeclarationKind::RpcService(decl) => Some(self.convert_rpc_service(decl)),
            cst::DeclarationKind::Invalid(_) => None,
        }
    }

    fn convert_declaration(&mut self, declaration: &cst::Declaration<'_>) {
        self.with_span(declaration.span, |self_| {
            if let Some(declaration) = self_.convert_declaration_kind(&declaration.kind) {
                match self_
                    .schema
                    .type_declarations
                    .entry(declaration.identifier.value)
                {
                    Entry::Occupied(entry) => {
                        let span = entry.get().full_span;
                        self_.emit_error(
                            ErrorKind::TYPE_DEFINED_TWICE,
                            [
                                Label::secondary(self_.schema.file_id, span)
                                    .with_message("first definition was here"),
                                Label::secondary(self_.schema.file_id, self_.current_span)
                                    .with_message("second definition was here"),
                            ],
                            Some(&format!(
                                "cannot define type {} twice",
                                self_.ctx.resolve_identifier(declaration.identifier.value)
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(declaration);
                    }
                }
            }
        })
    }

    fn convert_string_literal(&mut self, lit: &cst::StringLiteral) -> StringLiteral {
        StringLiteral {
            span: lit.span,
            value: lit.value.clone(),
        }
    }

    fn convert_attribute(&mut self, attribute: &cst::AttributeKind<'_>) -> Attribute {
        match attribute {
            cst::AttributeKind::Ident(ident) => Attribute {
                span: ident.span,
                kind: AttributeKind::Identifier(self.ctx.intern(ident.ident)),
            },
            cst::AttributeKind::String(literal) => Attribute {
                span: literal.span,
                kind: AttributeKind::String(literal.value.clone()),
            },
        }
    }

    fn convert_table(&mut self, decl: &cst::TableDeclaration<'_>) -> Declaration {
        let default_docstring = if let Some(namespace) = &self.schema.namespace {
            format!(
                "The table `{}` in the namespace `{}`",
                decl.ident.ident,
                self.ctx.format_namespace_path(&namespace.1)
            )
        } else {
            format!("The table `{}`", decl.ident.ident)
        };
        let docstrings = self.convert_docstrings(
            &decl.keyword.token_metadata,
            default_docstring,
            Some(("Table", decl.ident.ident, decl.keyword.span)),
        );
        self.handle_invalid_docstrings(&decl.ident.token_metadata);
        if let Some(metadata) = &decl.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&decl.start_brace.token_metadata);
        self.handle_invalid_docstrings(&decl.end_brace.token_metadata);

        let identifier = self.convert_ident(&decl.ident);
        let metadata = self.convert_metadata(&decl.metadata);
        let mut fields: IndexMap<RawIdentifier, StructField> = IndexMap::new();

        for field in &decl.fields {
            self.with_span(field.span, |self_| {
                let field = self_.convert_field(field, decl.ident.ident, true);
                let identifier = field.ident;
                match fields.entry(identifier.value) {
                    Entry::Occupied(entry) => {
                        let span = entry.get().span;
                        self_.emit_error(
                            ErrorKind::FIELD_DEFINED_TWICE,
                            [
                                Label::secondary(self_.schema.file_id, span)
                                    .with_message("first field was here"),
                                Label::secondary(self_.schema.file_id, self_.current_span)
                                    .with_message("second field was here"),
                            ],
                            Some(&format!(
                                "cannot define field {} twice",
                                self_.ctx.resolve_identifier(identifier.value)
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(field);
                    }
                }
            })
        }
        Declaration {
            file_id: self.schema.file_id,
            docstrings,
            full_span: decl.keyword.span.merge(decl.end_brace.span),
            definition_span: decl.keyword.span.merge(decl.ident.span),
            identifier,
            kind: TypeDeclarationKind::Table(Struct { metadata, fields }),
        }
    }

    fn convert_struct(&mut self, decl: &cst::StructDeclaration<'_>) -> Declaration {
        let default_docstring = if let Some(namespace) = &self.schema.namespace {
            format!(
                "The struct `{}` in the namespace `{}`",
                decl.ident.ident,
                self.ctx.format_namespace_path(&namespace.1)
            )
        } else {
            format!("The struct `{}`", decl.ident.ident)
        };
        let docstrings = self.convert_docstrings(
            &decl.keyword.token_metadata,
            default_docstring,
            Some(("Struct", decl.ident.ident, decl.keyword.span)),
        );
        self.handle_invalid_docstrings(&decl.ident.token_metadata);
        if let Some(metadata) = &decl.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&decl.start_brace.token_metadata);
        self.handle_invalid_docstrings(&decl.end_brace.token_metadata);

        let identifier = self.convert_ident(&decl.ident);
        let metadata = self.convert_metadata(&decl.metadata);
        let mut fields: IndexMap<RawIdentifier, StructField> = IndexMap::new();

        for field in &decl.fields {
            self.with_span(field.span, |self_| {
                let field = self_.convert_field(field, decl.ident.ident, false);
                let identifier = field.ident;
                match fields.entry(identifier.value) {
                    Entry::Occupied(entry) => {
                        let span = entry.get().span;
                        self_.emit_error(
                            ErrorKind::FIELD_DEFINED_TWICE,
                            [
                                Label::secondary(self_.schema.file_id, span)
                                    .with_message("first field was here"),
                                Label::secondary(self_.schema.file_id, self_.current_span)
                                    .with_message("second field was here"),
                            ],
                            Some(&format!(
                                "cannot define field {} twice",
                                self_.ctx.resolve_identifier(identifier.value)
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(field);
                    }
                }
            })
        }
        Declaration {
            file_id: self.schema.file_id,
            docstrings,
            full_span: decl.keyword.span.merge(decl.end_brace.span),
            definition_span: decl.keyword.span.merge(decl.ident.span),
            identifier,
            kind: TypeDeclarationKind::Struct(Struct { metadata, fields }),
        }
    }

    fn convert_field(
        &mut self,
        field: &cst::FieldDeclaration<'_>,
        parent_ident: &str,
        is_table: bool,
    ) -> StructField {
        let default_docstring = format!(
            "The field `{}` in the {} `{}`",
            field.ident.ident,
            if is_table { "table" } else { "struct" },
            parent_ident
        );
        let docstrings =
            self.convert_docstrings(&field.ident.token_metadata, default_docstring, None);
        self.handle_invalid_docstrings(&field.colon.token_metadata);
        self.handle_many_invalid_docstrings(field.type_.kind.token_metas());
        if let Some((eq, expr)) = &field.assignment {
            self.handle_invalid_docstrings(&eq.token_metadata);
            self.handle_many_invalid_docstrings(expr.kind.token_metas());
        }
        if let Some(metadata) = &field.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&field.semicolon.token_metadata);

        StructField {
            span: field.span,
            ident: self.convert_ident(&field.ident),
            docstrings,
            type_: self.convert_type(&field.type_),
            assignment: field
                .assignment
                .as_ref()
                .and_then(|(_eq, assignment)| self.convert_expr(assignment)),
            metadata: self.convert_metadata(&field.metadata),
        }
    }

    fn convert_enum(&mut self, decl: &cst::EnumDeclaration<'_>) -> Declaration {
        let default_docstring = if let Some(namespace) = &self.schema.namespace {
            format!(
                "The enum `{}` in the namespace `{}`",
                decl.ident.ident,
                self.ctx.format_namespace_path(&namespace.1)
            )
        } else {
            format!("The enum `{}`", decl.ident.ident)
        };
        let docstrings = self.convert_docstrings(
            &decl.keyword.token_metadata,
            default_docstring,
            Some(("Enum", decl.ident.ident, decl.keyword.span)),
        );
        self.handle_invalid_docstrings(&decl.ident.token_metadata);
        if let Some((colon, type_)) = &decl.type_ {
            self.handle_invalid_docstrings(&colon.token_metadata);
            self.handle_many_invalid_docstrings(type_.kind.token_metas());
        }
        if let Some(metadata) = &decl.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&decl.start_brace.token_metadata);
        self.handle_invalid_docstrings(&decl.end_brace.token_metadata);

        let identifier = self.convert_ident(&decl.ident);
        let type_ = if let Some((_colon, type_)) = &decl.type_ {
            self.convert_type_to_integer_type(type_)
                .unwrap_or(IntegerType::U64)
        } else {
            self.emit_error(
                ErrorKind::DECLARATION_PARSE_ERROR,
                [
                    Label::primary(
                        self.schema.file_id,
                        Span::new(decl.ident.span.end(), decl.ident.span.end()),
                    )
                    .with_message("Type should be inserted here"),
                    Label::secondary(
                        self.schema.file_id,
                        Span::new(decl.ident.span.end(), decl.ident.span.end()),
                    )
                    .with_message("Hint: Try inserting `: [integer type]`"),
                ],
                Some("Enum declarations must have a representation-type"),
            );
            IntegerType::U64
        };
        let metadata = self.convert_metadata(&decl.metadata);

        let mut variants: IndexMap<RawIdentifier, EnumVariant> = IndexMap::new();

        for variant in &decl.declarations {
            self.with_span(variant.span, |self_| {
                let variant = self_.convert_enum_variant(variant, decl.ident.ident);
                let identifier = variant.ident;
                match variants.entry(variant.ident.value) {
                    Entry::Occupied(entry) => {
                        let span = entry.get().span;
                        self_.emit_error(
                            ErrorKind::FIELD_DEFINED_TWICE,
                            [
                                Label::secondary(self_.schema.file_id, span)
                                    .with_message("first variant was here"),
                                Label::secondary(self_.schema.file_id, self_.current_span)
                                    .with_message("second variant was here"),
                            ],
                            Some(&format!(
                                "cannot define variant {} twice",
                                self_.ctx.resolve_identifier(identifier.value)
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(variant);
                    }
                }
            });
        }
        let definition_span = decl.keyword.span.merge(decl.ident.span);

        Declaration {
            file_id: self.schema.file_id,
            docstrings,
            full_span: decl.keyword.span.merge(decl.end_brace.span),
            definition_span,
            identifier,
            kind: TypeDeclarationKind::Enum(Enum {
                type_span: decl
                    .type_
                    .as_ref()
                    .map(|(_colon, type_)| type_.span)
                    .unwrap_or(definition_span),
                metadata,
                type_,
                variants,
            }),
        }
    }

    fn convert_enum_variant(
        &mut self,
        variant: &cst::EnumValDeclaration<'_>,
        parent_ident: &str,
    ) -> EnumVariant {
        let default_docstring = format!(
            "The variant `{}` in the enum `{}`",
            variant.ident.ident, parent_ident
        );
        let docstrings =
            self.convert_docstrings(&variant.ident.token_metadata, default_docstring, None);
        if let Some((eq, value)) = &variant.assignment {
            self.handle_invalid_docstrings(&eq.token_metadata);
            self.handle_many_invalid_docstrings(value.kind.token_metas());
        }
        if let Some(comma) = &variant.comma {
            self.handle_invalid_docstrings(&comma.token_metadata);
        }

        let ident = self.convert_ident(&variant.ident);
        let value = if let Some((_equals, assignment)) = &variant.assignment {
            self.convert_expr_to_integer_literal(assignment)
        } else {
            None
        };
        EnumVariant {
            span: variant.span,
            ident,
            value,
            docstrings,
        }
    }

    fn convert_union(&mut self, decl: &cst::UnionDeclaration<'_>) -> Declaration {
        let default_docstring = if let Some(namespace) = &self.schema.namespace {
            format!(
                "The union `{}` in the namespace `{}`",
                decl.ident.ident,
                self.ctx.format_namespace_path(&namespace.1)
            )
        } else {
            format!("The union `{}`", decl.ident.ident)
        };
        let docstrings = self.convert_docstrings(
            &decl.keyword.token_metadata,
            default_docstring,
            Some(("Union", decl.ident.ident, decl.keyword.span)),
        );
        self.handle_invalid_docstrings(&decl.ident.token_metadata);
        if let Some(metadata) = &decl.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&decl.start_brace.token_metadata);
        self.handle_invalid_docstrings(&decl.end_brace.token_metadata);

        let identifier = self.convert_ident(&decl.ident);
        let metadata = self.convert_metadata(&decl.metadata);

        let mut variants: IndexMap<UnionKey, UnionVariant> = IndexMap::new();

        for variant in &decl.declarations {
            self.with_span(variant.span, |self_| {
                let variant = self_.convert_union_variant(variant, decl.ident.ident);
                let key = variant
                    .ident
                    .map(|ident| UnionKey::Identifier(ident.value))
                    .unwrap_or_else(|| UnionKey::Type(variant.type_.clone()));
                match variants.entry(key) {
                    Entry::Occupied(entry) => {
                        let span = entry.get().span;
                        if let Some(ident) = variant.ident {
                            self_.emit_error(
                                ErrorKind::FIELD_DEFINED_TWICE,
                                [
                                    Label::secondary(self_.schema.file_id, span)
                                        .with_message("first variant was here"),
                                    Label::secondary(self_.schema.file_id, self_.current_span)
                                        .with_message("second variant was here"),
                                ],
                                Some(&format!(
                                    "cannot define union variant with name {} twice",
                                    self_.ctx.resolve_identifier(ident.value)
                                )),
                            );
                        } else {
                            self_.emit_error(
                                ErrorKind::FIELD_DEFINED_TWICE,
                                [
                                    Label::secondary(self_.schema.file_id, span)
                                        .with_message("first variant was here"),
                                    Label::secondary(self_.schema.file_id, self_.current_span)
                                        .with_message("second variant was here"),
                                ],
                                Some(&format!(
                                    "cannot define union variant with type {} twice",
                                    self_.ctx.format_type(&variant.type_),
                                )),
                            );
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(variant);
                    }
                }
            });
        }

        Declaration {
            docstrings,
            file_id: self.schema.file_id,
            full_span: decl.keyword.span.merge(decl.end_brace.span),
            definition_span: decl.keyword.span.merge(decl.ident.span),
            identifier,
            kind: TypeDeclarationKind::Union(Union { metadata, variants }),
        }
    }

    fn convert_union_variant(
        &mut self,
        variant: &cst::UnionValDeclaration<'_>,
        parent_ident: &str,
    ) -> UnionVariant {
        let mut type_metas = variant.type_.kind.token_metas();

        let docstrings;
        if let Some((name, colon)) = &variant.name {
            let default_docstring = format!(
                "The variant `{}` in the union `{}`",
                name.ident, parent_ident
            );
            docstrings = self.convert_docstrings(&name.token_metadata, default_docstring, None);
            self.handle_invalid_docstrings(&colon.token_metadata);
        } else {
            let mut type_name = String::new();
            // TODO: figure out if we should keep maintaining that entire pretty printer or do something simpler
            let mut printer = PrettyPrinter::new(&mut type_name, "");
            printer.write_type(&variant.type_).unwrap();
            let default_docstring =
                format!("The variant of type `{type_name}` in the union `{parent_ident}`",);
            docstrings =
                self.convert_docstrings(type_metas.next().unwrap(), default_docstring, None);
        }

        self.handle_many_invalid_docstrings(type_metas);

        if let Some(comma) = &variant.comma {
            self.handle_invalid_docstrings(&comma.token_metadata);
        }

        let ident = if let Some((name, _colon)) = &variant.name {
            Some(self.convert_ident(name))
        } else {
            None
        };
        UnionVariant {
            span: variant.span,
            ident,
            type_: self.convert_type(&variant.type_),
            docstrings,
        }
    }

    fn convert_rpc_service(&mut self, decl: &cst::RpcServiceDeclaration<'_>) -> Declaration {
        let default_docstring = if let Some(namespace) = &self.schema.namespace {
            format!(
                "The rpc service `{}` in the namespace `{}`",
                decl.ident.ident,
                self.ctx.format_namespace_path(&namespace.1)
            )
        } else {
            format!("The rpc service `{}`", decl.ident.ident)
        };
        let docstrings = self.convert_docstrings(
            &decl.keyword.token_metadata,
            default_docstring,
            Some(("RPC Service", decl.ident.ident, decl.keyword.span)),
        );
        self.handle_invalid_docstrings(&decl.ident.token_metadata);
        self.handle_invalid_docstrings(&decl.start_brace.token_metadata);
        self.handle_invalid_docstrings(&decl.end_brace.token_metadata);

        let identifier = self.convert_ident(&decl.ident);
        let mut methods: IndexMap<RawIdentifier, RpcMethod> = IndexMap::new();

        for method in &decl.methods {
            self.with_span(method.span, |self_| {
                let method = self_.convert_rpc_method(method, decl.ident.ident);
                let identifier = method.ident;
                match methods.entry(identifier.value) {
                    Entry::Occupied(entry) => {
                        let span = entry.get().span;
                        self_.emit_error(
                            ErrorKind::FIELD_DEFINED_TWICE,
                            [
                                Label::secondary(self_.schema.file_id, span)
                                    .with_message("first method was here"),
                                Label::secondary(self_.schema.file_id, self_.current_span)
                                    .with_message("second method was here"),
                            ],
                            Some(&format!(
                                "cannot define rpc method {} twice",
                                self_.ctx.resolve_identifier(identifier.value)
                            )),
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(method);
                    }
                }
            })
        }

        Declaration {
            file_id: self.schema.file_id,
            docstrings,
            full_span: decl.keyword.span.merge(decl.end_brace.span),
            definition_span: decl.keyword.span.merge(decl.ident.span),
            identifier,
            kind: TypeDeclarationKind::RpcService(RpcService { methods }),
        }
    }

    fn convert_rpc_method(&mut self, method: &cst::RpcMethod<'_>, parent_ident: &str) -> RpcMethod {
        let default_docstring = format!(
            "The method `{}` on the service `{}`",
            method.ident.ident, parent_ident
        );
        let docstrings =
            self.convert_docstrings(&method.ident.token_metadata, default_docstring, None);
        self.handle_invalid_docstrings(&method.start_paren.token_metadata);
        self.handle_many_invalid_docstrings(method.argument_type.kind.token_metas());
        self.handle_invalid_docstrings(&method.end_paren.token_metadata);
        self.handle_invalid_docstrings(&method.colon.token_metadata);
        self.handle_many_invalid_docstrings(method.return_type.kind.token_metas());
        if let Some(metadata) = &method.metadata {
            self.handle_many_invalid_docstrings(metadata.token_metas());
        }
        self.handle_invalid_docstrings(&method.semicolon.token_metadata);

        RpcMethod {
            span: method.span,
            docstrings,
            ident: self.convert_ident(&method.ident),
            argument_type: self.convert_type(&method.argument_type),
            return_type: self.convert_type(&method.return_type),
            metadata: self.convert_metadata(&method.metadata),
        }
    }

    fn convert_metadata(&mut self, metadata: &Option<cst::Metadata<'_>>) -> MetadataMap {
        if let Some(metadata) = metadata {
            self.with_span(metadata.span, |self_| {
                let mut seen: HashMap<MetadataValueKindKey, Span> = HashMap::new();
                let values = metadata
                    .values
                    .iter()
                    .flat_map(|value| {
                        let (key, value) = self_.convert_metadata_value(value)?;
                        match seen.entry(key) {
                            hash_map::Entry::Occupied(entry) => {
                                self_.emit_error(
                                    ErrorKind::MISC_SEMANTIC_ERROR,
                                    [
                                        Label::secondary(self_.schema.file_id, *entry.get())
                                            .with_message("first attribute was here"),
                                        Label::secondary(self_.schema.file_id, value.span)
                                            .with_message("second attribute was here"),
                                    ],
                                    Some("cannot set the same attribute twice"),
                                );
                                None
                            }
                            hash_map::Entry::Vacant(entry) => {
                                entry.insert(value.span);
                                Some(value)
                            }
                        }
                    })
                    .collect();
                MetadataMap { seen, values }
            })
        } else {
            MetadataMap::default()
        }
    }

    fn convert_metadata_value(
        &mut self,
        metadata_value: &cst::MetadataValue<'_>,
    ) -> Option<(MetadataValueKindKey, MetadataValue)> {
        self.with_span(metadata_value.span, |self_| {
            // let key = self_.convert_ident(&metadata_value.key);
            let value = if let Some((_equals, assignment)) = &metadata_value.assignment {
                self_.convert_expr(assignment)
            } else {
                None
            };
            let key = MetadataValueKindKey::parse(metadata_value.key.ident).or_else(|| {
                self_.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    [Label::primary(
                        self_.schema.file_id,
                        metadata_value.key.span,
                    )],
                    Some(&format!("Unknown attribute `{}`", metadata_value.key.ident)),
                );
                None
            })?;
            use MetadataValueKindKey::*;

            let bail_span = value
                .as_ref()
                .map_or(metadata_value.span, |value| value.span);
            let bail = || -> Option<(MetadataValueKindKey, MetadataValue)> {
                self_.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    [Label::primary(self_.schema.file_id, bail_span)],
                    Some(&format!(
                        "Attribute `{}` {}",
                        metadata_value.key.ident,
                        key.requirement()
                    )),
                );
                None
            };

            let kind = match value {
                Some(Literal {
                    span,
                    kind: LiteralKind::Integer { is_negative, value },
                }) => {
                    let literal = IntegerLiteral {
                        span,
                        is_negative,
                        value,
                    };
                    match key {
                        ForceAlign => MetadataValueKind::ForceAlign(literal),
                        Id => MetadataValueKind::Id(literal),
                        _ => return bail(),
                    }
                }
                Some(Literal {
                    span,
                    kind: LiteralKind::String(value),
                }) => {
                    let literal = StringLiteral { span, value };
                    match key {
                        NativeType => MetadataValueKind::NativeType(literal),
                        NativeTypePackName => MetadataValueKind::NativeTypePackName(literal),
                        NestedFlatbuffer => MetadataValueKind::NestedFlatbuffer(literal),
                        Hash => MetadataValueKind::Hash(literal),
                        CppType => MetadataValueKind::CppType(literal),
                        CppPtrType => MetadataValueKind::CppPtrType(literal),
                        CppPtrTypeGet => MetadataValueKind::CppPtrTypeGet(literal),
                        CppStrType => MetadataValueKind::CppStrType(literal),
                        NativeDefault => MetadataValueKind::NativeDefault(literal),
                        Streaming => MetadataValueKind::Streaming(literal),
                        _ => return bail(),
                    }
                }
                Some(_) => return bail(),
                None => match key {
                    BitFlags => MetadataValueKind::BitFlags,
                    CsharpPartial => MetadataValueKind::CsharpPartial,
                    Private => MetadataValueKind::Private,
                    OriginalOrder => MetadataValueKind::OriginalOrder,
                    Required => MetadataValueKind::Required,
                    Deprecated => MetadataValueKind::Deprecated,
                    Key => MetadataValueKind::Key,
                    Shared => MetadataValueKind::Shared,
                    CppStrFlexCtor => MetadataValueKind::CppStrFlexCtor,
                    NativeInline => MetadataValueKind::NativeInline,
                    Flexbuffer => MetadataValueKind::Flexbuffer,
                    Idempotent => MetadataValueKind::Idempotent,
                    _ => return bail(),
                },
            };
            Some((
                key,
                MetadataValue {
                    span: metadata_value.span,
                    kind,
                },
            ))
        })
    }

    fn convert_ident(&mut self, ident: &cst::IdentToken<'_>) -> Identifier {
        Identifier {
            span: ident.span,
            value: self.ctx.intern(ident.ident),
        }
    }

    fn convert_docstrings(
        &mut self,
        token_metadata: &TokenMetadata<'_>,
        default_docstring: String,
        location: Option<(&str, &str, Span)>,
    ) -> Docstrings {
        let locations = location.map_or(Vec::new(), |(decl_type, ident, decl_span)| {
            vec![format!(
                "* {} `{}` in the file `{}:{}`",
                decl_type,
                ident,
                self.ctx.get_filename(self.schema.file_id).display(),
                self.ctx.get_line_number(self.schema.file_id, decl_span),
            )]
        });

        let mut out = Vec::new();
        for block in &token_metadata.pre_comment_blocks {
            for comment in &block.0 {
                match comment.kind {
                    CommentKind::Comment => continue,
                    CommentKind::OuterDocstring => {
                        out.push(Docstring {
                            span: comment.span,
                            value: comment.content.to_owned(),
                        });
                        self.allow_outer_docstrings = false;
                    }
                    CommentKind::InnerDocstring => {
                        if self.allow_outer_docstrings {
                            self.schema.docstrings.docstrings.push(Docstring {
                                span: comment.span,
                                value: comment.content.to_owned(),
                            });
                        } else {
                            self.emit_error(
                                ErrorKind::FILE_ORDER,
                                [Label::primary(self.schema.file_id, comment.span)],
                                Some(concat!(
                                    "Inner doc comments (those starting with `//!`) are only allowed ",
                                    "at the beginning of the file and are used to document the namespace. ",
                                    "If you meant to write a normal doc comment, those start with `///`."
                                )),
                            );
                        }
                    }
                }
            }
        }
        self.allow_outer_docstrings = false;
        if let Some(comment) = &token_metadata.post_comment {
            self.handle_invalid_docstring(comment);
        }
        Docstrings {
            docstrings: out,
            default_docstring,
            locations,
        }
    }

    fn handle_invalid_docstring(&mut self, comment: &Comment<'_>) {
        match comment.kind {
            CommentKind::Comment => (),
            CommentKind::OuterDocstring => {
                self.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    [Label::primary(self.schema.file_id, comment.span)],
                    Some("Doc comments are not meaningful here."),
                );
                self.allow_outer_docstrings = false;
            }
            CommentKind::InnerDocstring => {
                if self.allow_outer_docstrings {
                    self.schema.docstrings.docstrings.push(Docstring {
                        span: comment.span,
                        value: comment.content.to_owned(),
                    });
                } else {
                    self.emit_error(
                        ErrorKind::MISC_SEMANTIC_ERROR,
                        [Label::primary(self.schema.file_id, comment.span)],
                        Some("Inner doc comments are not meaningful here."),
                    );
                }
            }
        }
    }

    fn handle_invalid_docstrings(&mut self, token_metadata: &TokenMetadata<'_>) {
        for block in &token_metadata.pre_comment_blocks {
            for comment in &block.0 {
                self.handle_invalid_docstring(comment);
            }
        }
        self.allow_outer_docstrings = false;
        if let Some(comment) = &token_metadata.post_comment {
            self.handle_invalid_docstring(comment);
        }
    }

    fn handle_many_invalid_docstrings<'a, I: IntoIterator<Item = &'a TokenMetadata<'a>>>(
        &mut self,
        token_metadatas: I,
    ) {
        for token_metadata in token_metadatas {
            self.handle_invalid_docstrings(token_metadata);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NumericalParseError {
    Overflow,
    Underflow,
    Format,
}

fn integer_literal_to_u32(literal: &IntegerLiteral) -> Result<u32, NumericalParseError> {
    let mut base = 10;
    let mut s = literal.value.as_str();
    if literal.value.starts_with("0x") {
        base = 16;
        s = &s[2..];
    }

    let mut result = 0u32;
    let mut seen_char = false;
    for c in s.chars() {
        let c = match c {
            '_' => continue,
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'a' | 'A' if base == 16 => 10,
            'b' | 'B' if base == 16 => 11,
            'c' | 'C' if base == 16 => 12,
            'd' | 'D' if base == 16 => 13,
            'e' | 'E' if base == 16 => 14,
            'f' | 'F' if base == 16 => 15,
            _ => return Err(NumericalParseError::Format),
        };
        seen_char = true;
        if let Some(n) = result
            .checked_mul(base)
            .and_then(|result| result.checked_add(c))
        {
            result = n;
        } else if literal.is_negative {
            return Err(NumericalParseError::Underflow);
        } else {
            return Err(NumericalParseError::Overflow);
        }
    }

    if literal.is_negative && result > 0 {
        Err(NumericalParseError::Underflow)
    } else if !seen_char {
        Err(NumericalParseError::Format)
    } else {
        Ok(result)
    }
}
