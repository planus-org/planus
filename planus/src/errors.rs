#[derive(Copy, Clone, thiserror::Error, Debug)]
#[error("In {source_location}: {error_kind}")]
pub struct Error {
    pub source_location: ErrorLocation,
    #[source]
    pub error_kind: ErrorKind,
}

#[derive(Copy, Clone, thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("Invalid offset")]
    InvalidOffset,
    #[error("Invalid length")]
    InvalidLength,
    #[error(transparent)]
    UnknownEnumTag {
        #[from]
        source: UnknownEnumTagKind,
    },
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

#[derive(Clone, thiserror::Error, Debug)]
#[error("In {source_location}: {error_kind}")]
pub struct UnknownEnumTag {
    pub source_location: ErrorLocation,
    #[source]
    pub error_kind: UnknownEnumTagKind,
}

#[derive(Copy, Clone, thiserror::Error, Debug)]
#[error("Unknown enum (tag = {tag})")]
pub struct UnknownEnumTagKind {
    pub tag: i128,
}

#[derive(Copy, Clone, Debug)]
pub struct ErrorLocation {
    pub type_: &'static str,
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

impl From<UnknownEnumTag> for Error {
    fn from(error: UnknownEnumTag) -> Self {
        Self {
            source_location: error.source_location,
            error_kind: error.error_kind.into(),
        }
    }
}

impl UnknownEnumTagKind {
    pub fn with_error_location(
        self,
        type_: &'static str,
        method: &'static str,
        byte_offset: usize,
    ) -> UnknownEnumTag {
        UnknownEnumTag {
            source_location: ErrorLocation {
                type_,
                method,
                byte_offset,
            },
            error_kind: self,
        }
    }
}

impl ErrorKind {
    pub fn with_error_location(
        self,
        type_: &'static str,
        method: &'static str,
        byte_offset: usize,
    ) -> Error {
        Error {
            source_location: ErrorLocation {
                type_,
                method,
                byte_offset,
            },
            error_kind: self,
        }
    }
}
