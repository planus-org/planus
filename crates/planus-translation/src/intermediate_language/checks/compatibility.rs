use codespan_reporting::diagnostic::Label;
use planus_types::ast;

use crate::{ctx::Ctx, error::ErrorKind};

pub fn check_ast(ctx: &Ctx, schema: &ast::Schema) {
    for native_include in &schema.native_includes {
        ctx.emit_error(
            ErrorKind::NOT_SUPPORTED,
            [Label::primary(schema.file_id, native_include.span)],
            Some("Native includes are not supported"),
        );
    }

    for attribute in &schema.attributes {
        ctx.emit_error(
            ErrorKind::NOT_SUPPORTED,
            [Label::primary(schema.file_id, attribute.span)],
            Some("User attributes are not supported"),
        );
    }

    for decl in schema.type_declarations.values() {
        match &decl.kind {
            ast::TypeDeclarationKind::RpcService(_) => {
                ctx.emit_error(
                    ErrorKind::NOT_SUPPORTED,
                    [Label::primary(schema.file_id, decl.definition_span)],
                    Some("Rpc services are not currently supported"),
                );
            }
            ast::TypeDeclarationKind::Struct(inner_decl) if inner_decl.fields.is_empty() => {
                ctx.emit_error(
                    ErrorKind::NOT_SUPPORTED,
                    [Label::primary(schema.file_id, decl.definition_span)],
                    Some("Empty structs are not supported"),
                );
            }
            _ => (),
        }
    }
}
