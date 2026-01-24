use logos::Logos;

#[derive(Logos, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub enum Token<'input> {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'input str),

    #[token("+", |_| Symbol::Plus)]
    #[token("-", |_| Symbol::Minus)]
    #[token(";", |_| Symbol::Semicolon)]
    #[token(":", |_| Symbol::Colon)]
    #[token(",", |_| Symbol::Comma)]
    #[token(".", |_| Symbol::Period)]
    #[token("=", |_| Symbol::Equals)]
    #[token("(", |_| Symbol::ParenOpen)]
    #[token(")", |_| Symbol::ParenClose)]
    #[token("{", |_| Symbol::BraceOpen)]
    #[token("}", |_| Symbol::BraceClose)]
    #[token("[", |_| Symbol::BracketOpen)]
    #[token("]", |_| Symbol::BracketClose)]
    Symbol(Symbol),

    #[token("include", |_| Keyword::Include)]
    #[token("native_include", |_| Keyword::NativeInclude)]
    #[token("namespace", |_| Keyword::Namespace)]
    #[token("attribute", |_| Keyword::Attribute)]
    #[token("table", |_| Keyword::Table)]
    #[token("struct", |_| Keyword::Struct)]
    #[token("enum", |_| Keyword::Enum)]
    #[token("union", |_| Keyword::Union)]
    #[token("root_type", |_| Keyword::RootType)]
    #[token("rpc_service", |_| Keyword::RpcService)]
    #[token("file_extension", |_| Keyword::FileExtension)]
    #[token("file_identifier", |_| Keyword::FileIdentifier)]
    Keyword(Keyword),

    // This token is not derived by the Logos lexer derived for
    // this enum. The impl derived here will simply store an empty
    // string. When the full lexer encounters this token, it will
    // morph the raw lexer into a text lexer, and use that
    // to fill in the actual string, and then morphing the lexer back.
    // Look in the function next_token on the full lexer for how this is done.
    #[token("\"", |_| String::new())]
    #[token("'", |_| String::new())]
    StringLiteral(String),

    #[regex("[0-9][0-9_]*")]
    #[regex("0x[0-9a-fA-F_]+")]
    IntegerLiteral(&'input str),

    #[regex("(([.][0-9]+)|([0-9]+[.][0-9]*))([eE][-+]?[0-9]+)?")]
    #[regex("[0-9]+[eE][-+]?[0-9]+")]
    #[regex(
        "0[xX](([.][0-9a-fA-F]+)|([0-9a-fA-F]+[.][0-9a-fA-F]*)|([0-9a-fA-F]+))([pP][-+]?[0-9]+)"
    )]
    FloatLiteral(&'input str),

    #[regex(r"///[^\n]*", |lex| CommentToken { kind: CommentKind::OuterDocstring, content: &lex.slice()[3..] }, allow_greedy = true)]
    #[regex(r"//![^\n]*", |lex| CommentToken { kind: CommentKind::InnerDocstring, content: &lex.slice()[3..] }, allow_greedy = true)]
    #[regex(r"//[^\n]*", |lex| CommentToken { kind: CommentKind::Comment, content: &lex.slice()[2..] }, allow_greedy = true)]
    Comment(CommentToken<'input>),

    #[regex(r"(\n|\r\n)")]
    Newline,

    #[regex(r"[ \t\f]+", logos::skip)]
    EndOfStream,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Symbol {
    Plus,
    Minus,
    Semicolon,
    Colon,
    Comma,
    Period,
    Equals,
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
    Include,
    NativeInclude,
    Namespace,
    Attribute,
    Table,
    Struct,
    Enum,
    Union,
    RootType,
    RpcService,
    FileExtension,
    FileIdentifier,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommentToken<'input> {
    pub kind: CommentKind,
    pub content: &'input str,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommentKind {
    Comment,
    InnerDocstring,
    OuterDocstring,
}

impl CommentKind {
    pub fn to_string(self) -> &'static str {
        match self {
            CommentKind::Comment => "//",
            CommentKind::InnerDocstring => "//!",
            CommentKind::OuterDocstring => "///",
        }
    }
}

impl std::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "identifier {s:?}"),
            Token::Symbol(sym) => write!(f, "{sym:?}"),
            Token::Keyword(keyword) => write!(f, "{keyword:?}"),
            Token::StringLiteral(s) => write!(f, "string literal {s:?}"),
            Token::IntegerLiteral(s) => write!(f, "integer literal {s:?}"),
            Token::FloatLiteral(s) => write!(f, "float literal {s:?}"),
            Token::Comment(s) => write!(f, "comment {s:?}"),
            Token::Newline => write!(f, "newline"),
            Token::EndOfStream => write!(f, "end-of-stream token"),
        }
    }
}

impl Symbol {
    pub fn to_string(&self) -> &'static str {
        match self {
            Symbol::Plus => "+",
            Symbol::Minus => "-",
            Symbol::Semicolon => ";",
            Symbol::Colon => ":",
            Symbol::Comma => ",",
            Symbol::Period => ".",
            Symbol::Equals => "=",
            Symbol::ParenOpen => "(",
            Symbol::ParenClose => ")",
            Symbol::BraceOpen => "{",
            Symbol::BraceClose => "}",
            Symbol::BracketOpen => "[",
            Symbol::BracketClose => "]",
        }
    }
}
impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Keyword {
    pub fn to_string(&self) -> &'static str {
        match self {
            Keyword::Include => "include",
            Keyword::NativeInclude => "native_include",
            Keyword::Namespace => "namespace",
            Keyword::Attribute => "attribute",
            Keyword::Table => "table",
            Keyword::Struct => "struct",
            Keyword::Enum => "enum",
            Keyword::Union => "union",
            Keyword::RootType => "root_type",
            Keyword::RpcService => "rpc_service",
            Keyword::FileExtension => "file_extension",
            Keyword::FileIdentifier => "file_identifier",
        }
    }
}
impl std::fmt::Debug for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_lexer() {
        use Token::*;
        let lexer = Token::lexer(
            r#"+-;:,.=%%(){}[]"include namespace
              attribute:table struct enum union
              root_type rpc_service file_extension
              file_identifier foo _foo _foo99 enume
              99 9_9 99_ _99 0x99 0x_99 0x9_9 0x99_
              0x_ 0x99
              .1 1.1 .1e4 1.1e4 1e4
              0x.1p4 0x1.1p4 0x1p4 0x.p2 true false
              // foo
              /// bar

              //! baz
            "#,
        );

        let expected = [
            Ok(Symbol(super::Symbol::Plus)),
            Ok(Symbol(super::Symbol::Minus)),
            Ok(Symbol(super::Symbol::Semicolon)),
            Ok(Symbol(super::Symbol::Colon)),
            Ok(Symbol(super::Symbol::Comma)),
            Ok(Symbol(super::Symbol::Period)),
            Ok(Symbol(super::Symbol::Equals)),
            Err(()),
            Err(()),
            Ok(Symbol(super::Symbol::ParenOpen)),
            Ok(Symbol(super::Symbol::ParenClose)),
            Ok(Symbol(super::Symbol::BraceOpen)),
            Ok(Symbol(super::Symbol::BraceClose)),
            Ok(Symbol(super::Symbol::BracketOpen)),
            Ok(Symbol(super::Symbol::BracketClose)),
            Ok(StringLiteral(String::new())),
            Ok(Keyword(super::Keyword::Include)),
            Ok(Keyword(super::Keyword::Namespace)),
            Ok(Newline),
            Ok(Keyword(super::Keyword::Attribute)),
            Ok(Symbol(super::Symbol::Colon)),
            Ok(Keyword(super::Keyword::Table)),
            Ok(Keyword(super::Keyword::Struct)),
            Ok(Keyword(super::Keyword::Enum)),
            Ok(Keyword(super::Keyword::Union)),
            Ok(Newline),
            Ok(Keyword(super::Keyword::RootType)),
            Ok(Keyword(super::Keyword::RpcService)),
            Ok(Keyword(super::Keyword::FileExtension)),
            Ok(Newline),
            Ok(Keyword(super::Keyword::FileIdentifier)),
            Ok(Ident("foo")),
            Ok(Ident("_foo")),
            Ok(Ident("_foo99")),
            Ok(Ident("enume")),
            Ok(Newline),
            Ok(IntegerLiteral("99")),
            Ok(IntegerLiteral("9_9")),
            Ok(IntegerLiteral("99_")),
            Ok(Ident("_99")),
            Ok(IntegerLiteral("0x99")),
            Ok(IntegerLiteral("0x_99")),
            Ok(IntegerLiteral("0x9_9")),
            Ok(IntegerLiteral("0x99_")),
            Ok(Newline),
            Ok(IntegerLiteral("0x_")), // error will be caught in a later pass
            Ok(IntegerLiteral("0x99")),
            Ok(Newline),
            Ok(FloatLiteral(".1")),
            Ok(FloatLiteral("1.1")),
            Ok(FloatLiteral(".1e4")),
            Ok(FloatLiteral("1.1e4")),
            Ok(FloatLiteral("1e4")),
            Ok(Newline),
            Ok(FloatLiteral("0x.1p4")),
            Ok(FloatLiteral("0x1.1p4")),
            Ok(FloatLiteral("0x1p4")),
            Ok(IntegerLiteral("0")),
            Ok(Ident("x")),
            Ok(Symbol(super::Symbol::Period)),
            Ok(Ident("p2")),
            Ok(Ident("true")),
            Ok(Ident("false")),
            Ok(Newline),
            Ok(Comment(super::CommentToken {
                kind: super::CommentKind::Comment,
                content: " foo",
            })),
            Ok(Newline),
            Ok(Comment(super::CommentToken {
                kind: super::CommentKind::OuterDocstring,
                content: " bar",
            })),
            Ok(Newline),
            Ok(Newline),
            Ok(Comment(super::CommentToken {
                kind: super::CommentKind::InnerDocstring,
                content: " baz",
            })),
            Ok(Newline),
        ];

        for (actual, expected) in lexer.zip(expected.iter()) {
            println!("{actual:?}");
            assert_eq!(&actual, expected);
        }
    }
}
