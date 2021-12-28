use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    sync::RwLock,
};

use codespan::{ByteIndex, FileId, Files, Span};
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

use crate::{
    ast::{Interner, RawIdentifier},
    error::{ErrorKind, LexicalError},
    lexer::TokenWithMetadata,
};

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
        Self {
            files: Files::default(),
            file_map: IndexMap::default(),
            interner: RefCell::new(Interner::default()),
            error_config: Config::default(),
            error_stream: RwLock::new(BufferedStandardStream::stderr(ColorChoice::Auto)),
            errors_seen: Cell::new(ErrorKind::empty()),
        }
    }
}

pub trait Labels {
    type Iter: Iterator<Item = Label<FileId>>;
    fn into_labels(self) -> Self::Iter;
}

impl Labels for Label<FileId> {
    type Iter = std::iter::Once<Label<FileId>>;
    fn into_labels(self) -> Self::Iter {
        std::iter::once(self)
    }
}

impl<T> Labels for Option<T>
where
    T: Labels,
{
    type Iter = std::iter::Flatten<std::option::IntoIter<T::Iter>>;
    fn into_labels(self) -> Self::Iter {
        self.map(|v| v.into_labels()).into_iter().flatten()
    }
}

impl Labels for FullSpan {
    type Iter = std::iter::Once<Label<FileId>>;
    fn into_labels(self) -> Self::Iter {
        std::iter::once(Label::primary(self.file_id, self.span))
    }
}

impl<'a> Labels for (FullSpan, &'a str) {
    type Iter = std::iter::Once<Label<FileId>>;
    fn into_labels(self) -> Self::Iter {
        std::iter::once(Label::primary(self.0.file_id, self.0.span).with_message(self.1))
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

    pub fn emit_warning(&self, labels: impl IntoIterator<Item = Label<FileId>>, msg: Option<&str>) {
        self.emit(Severity::Warning, labels, msg)
    }

    pub fn emit_note(&self, labels: impl IntoIterator<Item = Label<FileId>>, msg: Option<&str>) {
        self.emit(Severity::Note, labels, msg)
    }

    pub fn emit_bug(&self, labels: impl IntoIterator<Item = Label<FileId>>, msg: Option<&str>) {
        self.emit(Severity::Bug, labels, msg)
    }

    pub fn emit_simple_error(&self, error_type: ErrorKind, file_id: FileId, span: Span, msg: &str) {
        self.emit_error(
            error_type,
            FullSpan { file_id, span }.into_labels(),
            Some(msg),
        )
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
                msg = format!("unexpected token {:?}", token);
            }
            ParseError::User { error } => {
                span = error.span;
                msg = format!("{:?}", error.err);
            }
        }
        self.emit_error(
            ErrorKind::DECLARATION_PARSE_ERROR,
            FullSpan { file_id, span }.into_labels(),
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
                            Some(&format!("Could not read file {:?}: {}", path, e)),
                        );
                        None
                    }
                }
            }
        }
    }

    pub fn parse_file(&self, file_id: FileId) -> Option<crate::cst::Schema<'_>> {
        let lexer = crate::lexer::Lexer::new(self.files.source(file_id));
        let parser = crate::grammar::SchemaParser::new();
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
        path.push(&relative);
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
}
