use codespan::Span;

use crate::{
    cst::{
        AttributeKind, Declaration, DeclarationKind, EnumValDeclaration, Expr, FieldDeclaration,
        Metadata, NamespacePath, RpcMethod, Schema, StringLiteral, Type, UnionValDeclaration,
    },
    lexer::{Comment, CommentBlock, TokenMetadata, TokenWithMetadata},
};

const INDENT_STRING: &str = "    ";

fn indent_string(do_indent: bool) -> &'static str {
    if do_indent {
        INDENT_STRING
    } else {
        ""
    }
}

pub struct PrettyPrinter<'writer, 'src, T> {
    is_at_new_paragraph: bool,
    writer: &'writer mut T,
    source: &'src str,
}

impl<'writer, 'src, T: std::fmt::Write> PrettyPrinter<'writer, 'src, T> {
    fn write_standalone_comment(
        &mut self,
        indent: bool,
        comment: &Comment<'_>,
    ) -> std::fmt::Result {
        writeln!(
            self.writer,
            "{}{}{}",
            indent_string(indent),
            comment.kind.to_string(),
            comment.content
        )?;
        self.is_at_new_paragraph = false;
        Ok(())
    }

    fn write_post_comment(&mut self, post_comment: Option<&Comment<'_>>) -> std::fmt::Result {
        if let Some(post_comment) = post_comment {
            writeln!(
                self.writer,
                " {}{}",
                post_comment.kind.to_string(),
                post_comment.content
            )?;
        } else {
            writeln!(self.writer)?;
        }
        self.is_at_new_paragraph = false;
        Ok(())
    }

    fn begin_new_paragraph(&mut self) -> std::fmt::Result {
        if !self.is_at_new_paragraph {
            self.writer.write_str("\n")?;
        }
        self.is_at_new_paragraph = true;
        Ok(())
    }

    fn write_comment_block(&mut self, indent: bool, block: &CommentBlock<'_>) -> std::fmt::Result {
        for comment in &block.0 {
            self.write_standalone_comment(indent, comment)?;
        }
        Ok(())
    }

    fn write_token_meta(
        &mut self,
        indent: bool,
        output_post_comment: bool,
        token_meta: &TokenMetadata<'_>,
    ) -> std::fmt::Result {
        for block in &token_meta.pre_comment_blocks {
            self.begin_new_paragraph()?;
            self.write_comment_block(indent, block)?;
        }

        if output_post_comment {
            if let Some(post_comment) = &token_meta.post_comment {
                self.begin_new_paragraph()?;
                self.write_standalone_comment(indent, post_comment)?;
            }
        }

        Ok(())
    }

    fn write_token_metas<'a, I>(
        &mut self,
        indent: bool,
        token_metas: I,
    ) -> Result<Option<&'a Comment<'a>>, std::fmt::Error>
    where
        I: IntoIterator<Item = &'a TokenMetadata<'a>>,
    {
        let mut token_metas = token_metas.into_iter();
        if let Some(mut cur) = token_metas.next() {
            let do_new_paragraph = cur.token_begins_paragraph;
            for next in token_metas {
                self.write_token_meta(indent, true, cur)?;
                cur = next;
            }
            self.write_token_meta(indent, false, cur)?;
            if do_new_paragraph {
                self.begin_new_paragraph()?;
            }
            Ok(cur.post_comment.as_ref())
        } else {
            Ok(None)
        }
    }

    fn write_span(&mut self, span: Span) -> std::fmt::Result {
        self.write_str(
            self.source
                .get(span.start().to_usize()..span.end().to_usize())
                .unwrap(),
        )?;
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer.write_str(s)?;
        self.is_at_new_paragraph = false;
        Ok(())
    }

    fn write_string_literal(&mut self, s: &StringLiteral<'_>) -> std::fmt::Result {
        self.write_span(s.span)
    }

    fn write_declaration(&mut self, decl: &Declaration<'_>) -> Result<bool, std::fmt::Error> {
        match &decl.kind {
            DeclarationKind::Include(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    [
                        &decl.keyword.token_metadata,
                        &decl.path.token_metadata,
                        &decl.semicolon.token_metadata,
                    ],
                )?;
                self.write_str("include ")?;
                self.write_string_literal(&decl.path)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::NativeInclude(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    [
                        &decl.keyword.token_metadata,
                        &decl.path.token_metadata,
                        &decl.semicolon.token_metadata,
                    ],
                )?;
                self.write_str("native_include ")?;
                self.write_string_literal(&decl.path)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::Namespace(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    std::iter::once(&decl.keyword.token_metadata)
                        .chain(decl.namespace.token_metas())
                        .chain([&decl.semicolon.token_metadata]),
                )?;
                self.write_str("namespace ")?;
                self.write_namespace_path(&decl.namespace)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::RootType(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    std::iter::once(&decl.keyword.token_metadata)
                        .chain(decl.root_type.kind.token_metas())
                        .chain([&decl.semicolon.token_metadata]),
                )?;
                self.write_str("root_type ")?;
                self.write_type(&decl.root_type)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::FileExtension(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    [
                        &decl.keyword.token_metadata,
                        &decl.file_extension.token_metadata,
                        &decl.semicolon.token_metadata,
                    ],
                )?;
                self.write_str("file_extension ")?;
                self.write_span(decl.file_extension.span)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::FileIdentifier(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    [
                        &decl.keyword.token_metadata,
                        &decl.file_identifier.token_metadata,
                        &decl.semicolon.token_metadata,
                    ],
                )?;
                self.write_str("file_identifier ")?;
                self.write_span(decl.file_identifier.span)?;
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }

            DeclarationKind::Attribute(decl) => {
                let saved_post_comment = self.write_token_metas(
                    false,
                    [
                        &decl.keyword.token_metadata,
                        decl.attribute.token_meta(),
                        &decl.semicolon.token_metadata,
                    ],
                )?;
                self.write_str("attribute ")?;
                match &decl.attribute {
                    AttributeKind::Ident(ident) => self.write_str(ident.ident)?,
                    AttributeKind::String(s) => self.write_string_literal(s)?,
                }
                self.write_str(";")?;
                self.write_post_comment(saved_post_comment)?;
            }
            DeclarationKind::Table(decl) => {
                if decl.fields.is_empty() {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([
                                &decl.start_brace.token_metadata,
                                &decl.end_brace.token_metadata,
                            ]),
                    )?;
                    self.write_str("table ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {}")?;
                    self.write_post_comment(saved_post_comment)?;
                } else {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([&decl.start_brace.token_metadata]),
                    )?;
                    self.write_str("table ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {")?;
                    self.write_post_comment(saved_post_comment)?;
                    self.is_at_new_paragraph = true;
                    for field in &decl.fields {
                        self.write_field_decl(field)?;
                    }
                    self.write_token_meta(true, false, &decl.end_brace.token_metadata)?;
                    self.write_str("}")?;
                    self.write_post_comment(decl.end_brace.token_metadata.post_comment.as_ref())?;
                }
            }
            DeclarationKind::Struct(decl) => {
                if decl.fields.is_empty() {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([
                                &decl.start_brace.token_metadata,
                                &decl.end_brace.token_metadata,
                            ]),
                    )?;
                    self.write_str("struct ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {}")?;
                    self.write_post_comment(saved_post_comment)?;
                } else {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([&decl.start_brace.token_metadata]),
                    )?;
                    self.write_str("struct ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {")?;
                    self.write_post_comment(saved_post_comment)?;
                    self.is_at_new_paragraph = true;
                    for field in &decl.fields {
                        self.write_field_decl(field)?;
                    }
                    self.write_token_meta(true, false, &decl.end_brace.token_metadata)?;
                    self.write_str("}")?;
                    self.write_post_comment(decl.end_brace.token_metadata.post_comment.as_ref())?;
                }
            }
            DeclarationKind::Enum(decl) => {
                if decl.declarations.is_empty() {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [
                            &decl.keyword.token_metadata,
                            &decl.ident.token_metadata,
                            &decl.colon.token_metadata,
                        ]
                        .into_iter()
                        .chain(decl.type_.kind.token_metas())
                        .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                        .chain([
                            &decl.start_brace.token_metadata,
                            &decl.end_brace.token_metadata,
                        ]),
                    )?;
                    self.write_str("enum ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_str(": ")?;
                    self.write_type(&decl.type_)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {}")?;
                    self.write_post_comment(saved_post_comment)?;
                } else {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [
                            &decl.keyword.token_metadata,
                            &decl.ident.token_metadata,
                            &decl.colon.token_metadata,
                        ]
                        .into_iter()
                        .chain(decl.type_.kind.token_metas())
                        .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                        .chain([&decl.start_brace.token_metadata]),
                    )?;
                    self.write_str("enum ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_str(": ")?;
                    self.write_type(&decl.type_)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {")?;
                    self.write_post_comment(saved_post_comment)?;
                    self.is_at_new_paragraph = true;
                    for decl in &decl.declarations {
                        self.write_enum_val_decl(decl)?;
                    }
                    self.write_token_meta(true, false, &decl.end_brace.token_metadata)?;
                    self.write_str("}")?;
                    self.write_post_comment(decl.end_brace.token_metadata.post_comment.as_ref())?;
                }
            }
            DeclarationKind::Union(decl) => {
                if decl.declarations.is_empty() {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([
                                &decl.start_brace.token_metadata,
                                &decl.end_brace.token_metadata,
                            ]),
                    )?;
                    self.write_str("union ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {}")?;
                    self.write_post_comment(saved_post_comment)?;
                } else {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [&decl.keyword.token_metadata, &decl.ident.token_metadata]
                            .into_iter()
                            .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                            .chain([&decl.start_brace.token_metadata]),
                    )?;
                    self.write_str("union ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_metadata(&decl.metadata)?;
                    self.write_str(" {")?;
                    self.write_post_comment(saved_post_comment)?;
                    self.is_at_new_paragraph = true;
                    for decl in &decl.declarations {
                        self.write_union_val_decl(decl)?;
                    }
                    self.write_token_meta(true, false, &decl.end_brace.token_metadata)?;
                    self.write_str("}")?;
                    self.write_post_comment(decl.end_brace.token_metadata.post_comment.as_ref())?;
                }
            }
            DeclarationKind::RpcService(decl) => {
                if decl.methods.is_empty() {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [
                            &decl.keyword.token_metadata,
                            &decl.ident.token_metadata,
                            &decl.start_brace.token_metadata,
                            &decl.end_brace.token_metadata,
                        ],
                    )?;
                    self.write_str("rpc_service ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_str(" {}")?;
                    self.write_post_comment(saved_post_comment)?;
                } else {
                    let saved_post_comment = self.write_token_metas(
                        false,
                        [
                            &decl.keyword.token_metadata,
                            &decl.ident.token_metadata,
                            &decl.start_brace.token_metadata,
                        ],
                    )?;
                    self.write_str("rpc_service ")?;
                    self.write_str(decl.ident.ident)?;
                    self.write_str(" {")?;
                    self.write_post_comment(saved_post_comment)?;
                    self.is_at_new_paragraph = true;
                    for decl in &decl.methods {
                        self.write_rpc_method(decl)?;
                    }
                    self.write_token_meta(true, false, &decl.end_brace.token_metadata)?;
                    self.write_str("}")?;
                    self.write_post_comment(decl.end_brace.token_metadata.post_comment.as_ref())?;
                }
            }
            DeclarationKind::Invalid(error_recovery) => {
                let dropped_tokens = &error_recovery.dropped_tokens;
                if !dropped_tokens.is_empty() {
                    self.begin_new_paragraph()?;
                    let (mut start, TokenWithMetadata(_token, token_metadata), _end) =
                        dropped_tokens.first().unwrap();
                    for pre_comment_block in token_metadata.pre_comment_blocks.iter() {
                        if let Some(pre_comment) = pre_comment_block.0.first() {
                            start = start.min(pre_comment.span.start());
                            break;
                        }
                    }

                    let (_start, TokenWithMetadata(_token, token_metadata), mut end) =
                        dropped_tokens.last().unwrap();
                    if let Some(post_comment) = &token_metadata.post_comment {
                        end = end.max(post_comment.span.end());
                    }

                    self.write_span(Span::new(start, end))?;
                    self.write_post_comment(None)?;
                }
            }
        }

        match &decl.kind {
            DeclarationKind::Include(_)
            | DeclarationKind::NativeInclude(_)
            | DeclarationKind::Namespace(_)
            | DeclarationKind::Attribute(_)
            | DeclarationKind::RootType(_)
            | DeclarationKind::FileExtension(_)
            | DeclarationKind::FileIdentifier(_)
            | DeclarationKind::Invalid(_) => Ok(false),
            DeclarationKind::Table(_)
            | DeclarationKind::Struct(_)
            | DeclarationKind::Enum(_)
            | DeclarationKind::Union(_)
            | DeclarationKind::RpcService(_) => Ok(true),
        }
    }

    fn write_field_decl(&mut self, decl: &FieldDeclaration<'_>) -> Result<(), std::fmt::Error> {
        let saved_post_comment = self.write_token_metas(
            true,
            [&decl.ident.token_metadata, &decl.colon.token_metadata]
                .into_iter()
                .chain(decl.type_.kind.token_metas())
                .chain(decl.assignment.iter().flat_map(|(equals, expr)| {
                    std::iter::once(&equals.token_metadata).chain(expr.kind.token_metas())
                }))
                .chain(decl.metadata.iter().flat_map(|meta| meta.token_metas()))
                .chain([&decl.semicolon.token_metadata]),
        )?;
        self.write_str(INDENT_STRING)?;
        self.write_str(decl.ident.ident)?;
        self.write_str(": ")?;
        self.write_type(&decl.type_)?;
        if let Some((_equals, expr)) = &decl.assignment {
            self.write_str(" = ")?;
            self.write_expr(expr)?;
        }
        self.write_metadata(&decl.metadata)?;
        self.write_str(";")?;
        self.write_post_comment(saved_post_comment)?;

        Ok(())
    }

    fn write_enum_val_decl(
        &mut self,
        decl: &EnumValDeclaration<'_>,
    ) -> Result<(), std::fmt::Error> {
        let saved_post_comment = self.write_token_metas(
            true,
            std::iter::once(&decl.ident.token_metadata)
                .chain(decl.assignment.iter().flat_map(|(equals, expr)| {
                    std::iter::once(&equals.token_metadata).chain(expr.kind.token_metas())
                }))
                .chain(decl.comma.iter().flat_map(|token| [&token.token_metadata])),
        )?;
        self.write_str(INDENT_STRING)?;
        self.write_str(decl.ident.ident)?;
        if let Some((_equals, expr)) = &decl.assignment {
            self.write_str(" = ")?;
            self.write_expr(expr)?;
        }
        if decl.comma.is_some() {
            self.write_str(",")?;
        }
        self.write_post_comment(saved_post_comment)?;
        Ok(())
    }

    fn write_union_val_decl(
        &mut self,
        decl: &UnionValDeclaration<'_>,
    ) -> Result<(), std::fmt::Error> {
        let saved_post_comment = self.write_token_metas(
            true,
            decl.name
                .iter()
                .flat_map(|(ident, equals)| [&ident.token_metadata, &equals.token_metadata])
                .chain(decl.type_.kind.token_metas())
                .chain(decl.comma.iter().flat_map(|comma| [&comma.token_metadata])),
        )?;
        self.write_str(INDENT_STRING)?;
        if let Some((ident, _colon)) = &decl.name {
            self.write_str(ident.ident)?;
            self.write_str(": ")?;
        }
        self.write_type(&decl.type_)?;
        if decl.comma.is_some() {
            self.write_str(",")?;
        }
        self.write_post_comment(saved_post_comment)?;
        Ok(())
    }

    fn write_rpc_method(&mut self, decl: &RpcMethod<'_>) -> Result<(), std::fmt::Error> {
        let saved_post_comment = self.write_token_metas(
            true,
            [&decl.ident.token_metadata, &decl.start_paren.token_metadata]
                .into_iter()
                .chain(decl.argument_type.kind.token_metas())
                .chain([&decl.end_paren.token_metadata, &decl.colon.token_metadata])
                .chain(decl.return_type.kind.token_metas())
                .chain(
                    decl.metadata
                        .iter()
                        .flat_map(|metadata| metadata.token_metas()),
                )
                .chain([&decl.semicolon.token_metadata]),
        )?;
        self.write_str(INDENT_STRING)?;
        self.write_str(decl.ident.ident)?;
        self.write_str("(")?;
        self.write_type(&decl.argument_type)?;
        self.write_str("): ")?;
        self.write_type(&decl.return_type)?;
        self.write_metadata(&decl.metadata)?;
        self.write_str(";")?;
        self.write_post_comment(saved_post_comment)?;
        Ok(())
    }

    fn write_type(&mut self, decl: &Type<'_>) -> Result<(), std::fmt::Error> {
        match &decl.kind {
            crate::cst::TypeKind::Vector(typ) => {
                self.write_str("[")?;
                self.write_type(&typ.inner_type)?;
                self.write_str("]")?;
            }
            crate::cst::TypeKind::Array(typ) => {
                self.write_str("[")?;
                self.write_type(&typ.inner_type)?;
                self.write_str(": ")?;
                self.write_expr(&typ.size)?;
                self.write_str("]")?;
            }
            crate::cst::TypeKind::Path(path) => {
                self.write_namespace_path(path)?;
            }
        }
        Ok(())
    }

    fn write_expr(&mut self, decl: &Expr<'_>) -> Result<(), std::fmt::Error> {
        match &decl.kind {
            crate::cst::ExprKind::Ident(ident) => self.write_str(ident.ident),
            crate::cst::ExprKind::Integer(literal) => self.write_str(literal.value),
            crate::cst::ExprKind::Float(literal) => self.write_str(literal.value),
            crate::cst::ExprKind::String(literal) => self.write_span(literal.span),
            crate::cst::ExprKind::List(literal) => {
                self.write_str("[")?;
                let mut first = true;
                for value in &literal.values {
                    if !first {
                        self.write_str(", ")?;
                    }
                    first = false;
                    self.write_expr(&value.expr)?;
                }
                self.write_str("]")?;
                Ok(())
            }
            crate::cst::ExprKind::Signed { sign, inner } => {
                self.write_str(if sign.is_negative { "-" } else { "+" })?;
                self.write_expr(inner)
            }
        }
    }

    fn write_metadata(&mut self, decl: &Option<Metadata<'_>>) -> Result<(), std::fmt::Error> {
        if let Some(decl) = decl {
            self.write_str(" (")?;
            let mut first = true;
            for value in &decl.values {
                if !first {
                    self.write_str(", ")?;
                }
                first = false;
                self.write_str(value.key.ident)?;
                if let Some((_equals, value)) = &value.assignment {
                    self.write_str(": ")?;
                    self.write_expr(value)?;
                }
            }
            self.write_str(")")?;
        }
        Ok(())
    }

    fn write_namespace_path(&mut self, namespace: &NamespacePath) -> Result<(), std::fmt::Error> {
        for segment in &namespace.initial_segments {
            self.write_str(segment.ident.ident)?;
            self.write_str(".")?;
        }
        self.write_str(namespace.final_segment.ident)?;
        Ok(())
    }
}

pub fn pretty_print<W: std::fmt::Write>(
    source: &str,
    schema: &Schema<'_>,
    writer: &mut W,
) -> std::fmt::Result {
    let mut printer = PrettyPrinter {
        is_at_new_paragraph: true,
        writer,
        source,
    };

    let mut last_declaration_requires_paragraph = false;
    for decl in &schema.declarations {
        if last_declaration_requires_paragraph {
            printer.begin_new_paragraph()?;
        }
        last_declaration_requires_paragraph = printer.write_declaration(decl)?;
    }
    printer.write_token_meta(false, true, &schema.end_of_stream.token_metadata)?;
    Ok(())
}
