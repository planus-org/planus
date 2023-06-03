bitflags::bitflags! {
    #[derive(Copy, Clone, Default, PartialEq, Eq)]
    pub struct ErrorKind: u32 {
        const DECLARATION_PARSE_ERROR = 0x1;
        const UNKNOWN_IDENTIFIER = 0x2;
        const TYPE_ERROR = 0x4;
        const NUMERICAL_RANGE_ERROR = 0x8;
        const NUMERICAL_PARSE_ERROR = 0x10;
        const MISC_SEMANTIC_ERROR = 0x20;
        const TYPE_DEFINED_TWICE = 0x40;
        const FIELD_DEFINED_TWICE = 0x80;
        const FILE_ORDER = 0x100;
        const NOT_SUPPORTED = 0x200;
    }
}
