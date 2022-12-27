use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    sync::RwLock,
};

use codespan::{ByteIndex, FileId, Files, LineNumber, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, Severity},
    term::{
        self,
        termcolor::{BufferedStandardStream, ColorChoice},
        Config,
    },
};
use indexmap::IndexMap;
use lalrpop_util::ParseError;
use planus_lexer::{error::LexicalError, Lexer, TokenWithMetadata};
use planus_types::{
    ast::{self, BuiltinType, FloatType, IntegerType, Interner, RawIdentifier, Type, TypeKind},
    cst,
    intermediate::AbsolutePath,
};

use crate::error::ErrorKind;

pub struct FullSpan {
    pub file_id: FileId,
    pub span: Span,
}

pub struct Ctx {
    files: Files<String>,
    file_map: IndexMap<PathBuf, FileId>,
    interner: RefCell<Interner>,
    error_config: Config,
    error_stream: RwLock<BufferedStandardStream>,
    errors_seen: Cell<ErrorKind>,
}

impl Default for Ctx {
    fn default() -> Self {
        let color_choice = if atty::is(atty::Stream::Stderr) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        Self {
            files: Files::default(),
            file_map: IndexMap::default(),
            interner: RefCell::new(Interner::default()),
            error_config: Config::default(),
            error_stream: RwLock::new(BufferedStandardStream::stderr(color_choice)),
            errors_seen: Cell::new(ErrorKind::empty()),
        }
    }
}

impl Ctx {
    pub fn intern(&self, s: &str) -> RawIdentifier {
        self.interner.borrow_mut().get_or_intern(s)
    }

    pub fn resolve_identifier(&self, symbol: RawIdentifier) -> String {
        self.interner.borrow().resolve(symbol).unwrap().to_owned()
    }

    fn emit(
        &self,
        severity: Severity,
        labels: impl IntoIterator<Item = Label<FileId>>,
        msg: Option<&str>,
    ) {
        let mut diagnostic = Diagnostic::new(severity);
        if let Some(msg) = msg {
            diagnostic = diagnostic.with_message(msg);
        }
        let labels = labels.into_iter().collect::<Vec<_>>();
        if !labels.is_empty() {
            diagnostic = diagnostic.with_labels(labels);
        }
        term::emit(
            &mut *self.error_stream.write().unwrap(),
            &self.error_config,
            &self.files,
            &diagnostic,
        )
        .unwrap();
    }

    pub fn emit_error(
        &self,
        error_type: ErrorKind,
        labels: impl IntoIterator<Item = Label<FileId>>,
        msg: Option<&str>,
    ) {
        self.emit(Severity::Error, labels, msg);
        self.errors_seen.set(self.errors_seen.get() | error_type);
    }

    pub fn emit_simple_error(&self, error_type: ErrorKind, file_id: FileId, span: Span, msg: &str) {
        self.emit_error(error_type, [Label::primary(file_id, span)], Some(msg))
    }

    pub fn emit_parse_error(
        &self,
        file_id: FileId,
        error: &ParseError<ByteIndex, TokenWithMetadata<'_>, LexicalError>,
    ) {
        let span: Span;
        let msg: String;
        match error {
            ParseError::InvalidToken { location } => {
                span = Span::new(*location, *location);
                msg = "invalid token".to_string();
            }
            ParseError::UnrecognizedEOF { location, expected } => {
                span = Span::new(*location, *location);
                msg = format!("unexpected EOF, expected one of {}", expected.join(", "));
            }
            ParseError::UnrecognizedToken {
                token: (start, token, end),
                expected,
            } => {
                span = Span::new(*start, *end);
                msg = format!(
                    "unrecognized token `{:?}`, expected one of {}",
                    token,
                    expected.join(", ")
                );
            }
            ParseError::ExtraToken {
                token: (start, token, end),
            } => {
                span = Span::new(*start, *end);
                msg = format!("unexpected token {token:?}");
            }
            ParseError::User { error } => {
                span = error.span;
                msg = error.err.to_string();
            }
        }
        self.emit_error(
            ErrorKind::DECLARATION_PARSE_ERROR,
            [Label::primary(file_id, span)],
            Some(msg.as_str()),
        );
    }

    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        labels: impl IntoIterator<Item = Label<FileId>>,
    ) -> Option<FileId> {
        let normalized_path = crate::util::normalize_path(path.as_ref());
        match self.file_map.entry(normalized_path) {
            indexmap::map::Entry::Occupied(entry) => Some(*entry.into_mut()),
            indexmap::map::Entry::Vacant(entry) => {
                let path = entry.key().clone();
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let file_id = self.files.add(path, content);
                        entry.insert(file_id);
                        Some(file_id)
                    }
                    Err(e) => {
                        self.emit_error(
                            ErrorKind::DECLARATION_PARSE_ERROR,
                            labels,
                            Some(&format!("Could not read file {path:?}: {e}")),
                        );
                        None
                    }
                }
            }
        }
    }

    pub fn parse_file(&self, file_id: FileId) -> Option<cst::Schema<'_>> {
        let lexer = Lexer::new(self.files.source(file_id));
        let parser = crate::parser::SchemaParser::new();
        let parsed = parser.parse(file_id, self, lexer);
        match parsed {
            Ok(value) => Some(value),
            Err(error) => {
                self.emit_parse_error(file_id, &error);
                None
            }
        }
    }

    pub fn add_relative_path(
        &mut self,
        file_id: FileId,
        relative: &str,
        labels: impl IntoIterator<Item = Label<FileId>>,
    ) -> Option<FileId> {
        let path = self.get_filename(file_id);
        let mut path = PathBuf::from(path);
        path.push("..");
        path.push(relative);
        self.add_file(crate::util::normalize_path(&path), labels)
    }

    pub fn errors_seen(&self) -> ErrorKind {
        self.errors_seen.get()
    }

    pub fn has_errors(&self) -> bool {
        self.errors_seen() != ErrorKind::empty()
    }

    pub fn get_source(&self, file_id: FileId) -> &str {
        self.files.source(file_id)
    }

    pub fn get_filename(&self, file_id: FileId) -> &Path {
        self.files.name(file_id).as_ref()
    }

    pub fn get_line_number(&self, file_id: FileId, span: Span) -> LineNumber {
        self.files.line_index(file_id, span.start()).number()
    }

    pub fn format_namespace_path(&self, namespace_path: &ast::NamespacePath) -> String {
        let mut out = String::new();
        let mut first = true;
        for &symbol in &namespace_path.parts {
            if first {
                first = false;
            } else {
                out.push('.');
            }
            out.push_str(&self.resolve_identifier(symbol));
        }
        out
    }

    pub fn format_type(&self, type_: &Type) -> String {
        pub fn format_type(ctx: &Ctx, type_: &Type, out: &mut String) {
            match &type_.kind {
                TypeKind::Builtin(BuiltinType::Bool) => out.push_str("bool"),
                TypeKind::Builtin(BuiltinType::String) => out.push_str("string"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::U8)) => out.push_str("uint8"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::U16)) => out.push_str("uint16"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::U32)) => out.push_str("uint32"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::U64)) => out.push_str("uint64"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::I8)) => out.push_str("int8"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::I16)) => out.push_str("int16"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::I32)) => out.push_str("int32"),
                TypeKind::Builtin(BuiltinType::Integer(IntegerType::I64)) => out.push_str("int64"),
                TypeKind::Builtin(BuiltinType::Float(FloatType::F32)) => out.push_str("float32"),
                TypeKind::Builtin(BuiltinType::Float(FloatType::F64)) => out.push_str("float64"),
                TypeKind::Vector { inner_type } => {
                    out.push('[');
                    format_type(ctx, inner_type, out);
                    out.push(']');
                }
                TypeKind::Array { inner_type, size } => {
                    out.push('[');
                    format_type(ctx, inner_type, out);
                    out.push_str(&format!(": {size}]"));
                }
                TypeKind::Path(path) => {
                    out.push_str(&ctx.format_namespace_path(path));
                }
                TypeKind::Invalid => {
                    out.push('!');
                }
            }
        }

        let mut res = String::new();
        format_type(self, type_, &mut res);
        res
    }

    pub fn absolute_path_from_parts(&self, parts: &[ast::RawIdentifier]) -> AbsolutePath {
        let path = parts.iter().map(|&s| self.resolve_identifier(s)).collect();
        AbsolutePath(path)
    }
}
