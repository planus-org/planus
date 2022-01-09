#[derive(Copy, Clone, Debug)]
pub struct Error {
    pub source_location: ErrorLocation,
    pub error_kind: ErrorKind,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "In {}: {}", self.source_location, self.error_kind)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error_kind)
    }
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    InvalidOffset,
    InvalidLength,
    UnknownEnumTag { source: UnknownEnumTagKind },
    UnknownUnionTag { tag: u8 },
    InvalidVtableLength { length: u16 },
    InvalidUtf8 { source: core::str::Utf8Error },
    MissingRequired,
    MissingNullTerminator,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::InvalidOffset => write!(f, "Invalid offset"),
            ErrorKind::InvalidLength => write!(f, "Invalid length"),
            ErrorKind::UnknownEnumTag { source } => source.fmt(f),
            ErrorKind::UnknownUnionTag { tag } => write!(f, "Unknown union (tag = {})", tag),
            ErrorKind::InvalidVtableLength { length } => {
                write!(f, "Invalid vtable length (length = {})", length)
            }
            ErrorKind::InvalidUtf8 { source } => write!(f, "Invalid utf-8: {}", source),
            ErrorKind::MissingRequired => write!(f, "Missing required field"),
            ErrorKind::MissingNullTerminator => write!(f, "Missing null terminator"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorKind::InvalidOffset => None,
            ErrorKind::InvalidLength => None,
            ErrorKind::UnknownEnumTag { source } => Some(source),
            ErrorKind::UnknownUnionTag { .. } => None,
            ErrorKind::InvalidVtableLength { .. } => None,
            ErrorKind::InvalidUtf8 { source } => Some(source),
            ErrorKind::MissingRequired => None,
            ErrorKind::MissingNullTerminator => None,
        }
    }
}

impl From<UnknownEnumTagKind> for ErrorKind {
    fn from(source: UnknownEnumTagKind) -> Self {
        ErrorKind::UnknownEnumTag { source }
    }
}

impl From<core::str::Utf8Error> for ErrorKind {
    fn from(source: core::str::Utf8Error) -> Self {
        ErrorKind::InvalidUtf8 { source }
    }
}

#[derive(Clone, Debug)]
pub struct UnknownEnumTag {
    pub source_location: ErrorLocation,
    pub error_kind: UnknownEnumTagKind,
}

impl core::fmt::Display for UnknownEnumTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "In {}: {}", self.source_location, self.error_kind)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnknownEnumTag {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error_kind)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnknownEnumTagKind {
    pub tag: i128,
}

impl core::fmt::Display for UnknownEnumTagKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unknown enum (tag = {})", self.tag)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UnknownEnumTagKind {}

#[derive(Copy, Clone, Debug)]
pub struct ErrorLocation {
    pub type_: &'static str,
    pub method: &'static str,
    pub byte_offset: usize,
}

impl core::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl From<core::convert::Infallible> for Error {
    fn from(value: core::convert::Infallible) -> Self {
        match value {}
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
