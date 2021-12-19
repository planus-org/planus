use std::borrow::Cow;

#[derive(Clone, thiserror::Error, Debug)]
pub struct Error {
    pub source_location: ErrorLocation,
    #[source]
    pub error_kind: ErrorKind,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "In {}: {}", self.source_location, self.error_kind)
    }
}

#[derive(Clone, thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Invalid offset")]
    InvalidOffset,
    #[error("Invalid length")]
    InvalidLength,
    #[error("Unknown enum (tag = {tag})")]
    UnknownEnumTag { tag: i128 },
    #[error("Unknown union (tag = {tag})")]
    UnknownUnionTag { tag: u8 },
    #[error("Invalid vtable length (length = {length})")]
    InvalidVtableLength { length: u16 },
    #[error("Invalid utf-8")]
    InvalidUtf8 {
        #[from]
        source: std::str::Utf8Error,
    },
    #[error("Missing required field")]
    MissingRequired,
}

#[derive(Clone, Debug)]
pub struct ErrorLocation {
    pub type_: Cow<'static, str>,
    pub method: &'static str,
    pub byte_offset: usize,
}

impl std::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.byte_offset != usize::MAX {
            write!(
                f,
                "<{}@{:x}>::{}()",
                self.type_, self.byte_offset, self.method,
            )
        } else {
            write!(f, "<{}>::{}()", self.type_, self.method,)
        }
    }
}
