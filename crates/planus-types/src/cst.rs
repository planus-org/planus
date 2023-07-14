use codespan::{ByteIndex, Span};
use lalrpop_util::ErrorRecovery;
use planus_lexer::{LexicalError, TokenMetadata, TokenWithMetadata};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError<'input> {
    pub span: Span,
    pub inner_error: ErrorRecovery<codespan::ByteIndex, TokenWithMetadata<'input>, LexicalError>,
}

#[derive(Clone, Debug)]
pub struct Schema<'input> {
    pub span: Span,
    pub declarations: Vec<Declaration<'input>>,
    pub end_of_stream: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct Declaration<'input> {
    pub span: Span,
    pub kind: DeclarationKind<'input>,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum DeclarationKind<'input> {
    Include(IncludeDeclaration<'input>),
    NativeInclude(NativeIncludeDeclaration<'input>),
    Namespace(NamespaceDeclaration<'input>),
    Attribute(AttributeDeclaration<'input>),
    Table(TableDeclaration<'input>),
    Struct(StructDeclaration<'input>),
    Enum(EnumDeclaration<'input>),
    Union(UnionDeclaration<'input>),
    RootType(RootTypeDeclaration<'input>),
    RpcService(RpcServiceDeclaration<'input>),
    FileExtension(FileExtensionDeclaration<'input>),
    FileIdentifier(FileIdentifierDeclaration<'input>),
    Invalid(ErrorRecovery<ByteIndex, TokenWithMetadata<'input>, LexicalError>),
}

#[derive(Clone, Debug)]
pub struct IncludeDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub path: StringLiteral<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct NativeIncludeDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub path: StringLiteral<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct NamespaceDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub namespace: NamespacePath<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct AttributeDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub attribute: AttributeKind<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub enum AttributeKind<'input> {
    Ident(IdentToken<'input>),
    String(StringLiteral<'input>),
}

#[derive(Clone, Debug)]
pub struct TableDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub ident: IdentToken<'input>,
    pub metadata: Option<Metadata<'input>>,
    pub start_brace: SimpleToken<'input>,
    pub fields: Vec<FieldDeclaration<'input>>,
    pub end_brace: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct StructDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub ident: IdentToken<'input>,
    pub metadata: Option<Metadata<'input>>,
    pub start_brace: SimpleToken<'input>,
    pub fields: Vec<FieldDeclaration<'input>>,
    pub end_brace: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct EnumDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub ident: IdentToken<'input>,
    pub type_: Option<(SimpleToken<'input>, Type<'input>)>,
    pub metadata: Option<Metadata<'input>>,
    pub start_brace: SimpleToken<'input>,
    pub declarations: Vec<EnumValDeclaration<'input>>,
    pub end_brace: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct UnionDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub ident: IdentToken<'input>,
    pub metadata: Option<Metadata<'input>>,
    pub start_brace: SimpleToken<'input>,
    pub declarations: Vec<UnionValDeclaration<'input>>,
    pub end_brace: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct RootTypeDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub root_type: Type<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct RpcServiceDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub ident: IdentToken<'input>,
    pub start_brace: SimpleToken<'input>,
    pub methods: Vec<RpcMethod<'input>>,
    pub end_brace: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct RpcMethod<'input> {
    pub span: Span,
    pub ident: IdentToken<'input>,
    pub start_paren: SimpleToken<'input>,
    pub argument_type: Type<'input>,
    pub end_paren: SimpleToken<'input>,
    pub colon: SimpleToken<'input>,
    pub return_type: Type<'input>,
    pub metadata: Option<Metadata<'input>>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct FileExtensionDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub file_extension: StringLiteral<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct FileIdentifierDeclaration<'input> {
    pub keyword: SimpleToken<'input>,
    pub file_identifier: StringLiteral<'input>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct Metadata<'input> {
    pub span: Span,
    pub start_paren: SimpleToken<'input>,
    pub values: Vec<MetadataValue<'input>>,
    pub end_paren: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct MetadataValue<'input> {
    pub span: Span,
    pub key: IdentToken<'input>,
    pub assignment: Option<(SimpleToken<'input>, Expr<'input>)>, // the token is the '='
    pub comma: Option<SimpleToken<'input>>,
}

#[derive(Clone, Debug)]
pub struct FieldDeclaration<'input> {
    pub span: Span,
    pub ident: IdentToken<'input>,
    pub colon: SimpleToken<'input>,
    pub type_: Type<'input>,
    pub assignment: Option<(SimpleToken<'input>, Expr<'input>)>, // the token is the '='
    pub metadata: Option<Metadata<'input>>,
    pub semicolon: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct EnumValDeclaration<'input> {
    pub span: Span,
    pub ident: IdentToken<'input>,
    pub assignment: Option<(SimpleToken<'input>, Expr<'input>)>, // the token is the '='
    pub comma: Option<SimpleToken<'input>>,
}

#[derive(Clone, Debug)]
pub struct UnionValDeclaration<'input> {
    pub span: Span,
    pub name: Option<(IdentToken<'input>, SimpleToken<'input>)>, // the token is the ':'
    pub type_: Type<'input>,
    pub comma: Option<SimpleToken<'input>>,
}

#[derive(Clone, Debug)]
pub struct Type<'input> {
    pub span: Span,
    pub kind: TypeKind<'input>,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum TypeKind<'input> {
    Vector(VectorType<'input>),
    Array(ArrayType<'input>),
    Path(NamespacePath<'input>),
}

#[derive(Clone, Debug)]
pub struct VectorType<'input> {
    pub span: Span,
    pub start_bracket: SimpleToken<'input>,
    pub inner_type: Box<Type<'input>>,
    pub end_bracket: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct ArrayType<'input> {
    pub span: Span,
    pub start_bracket: SimpleToken<'input>,
    pub inner_type: Box<Type<'input>>,
    pub colon: SimpleToken<'input>,
    pub size: Expr<'input>,
    pub end_bracket: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct Expr<'input> {
    pub span: Span,
    pub kind: ExprKind<'input>,
}

#[derive(Clone, Debug)]
pub enum ExprKind<'input> {
    Ident(IdentToken<'input>),
    Integer(IntegerLiteral<'input>),
    Float(FloatLiteral<'input>),
    String(StringLiteral<'input>),
    List(ListLiteral<'input>),
    Signed {
        sign: Sign<'input>,
        inner: Box<Expr<'input>>,
    },
}

#[derive(Clone, Debug)]
pub struct Sign<'input> {
    pub span: Span,
    pub token: SimpleToken<'input>,
    pub is_negative: bool,
}

#[derive(Clone, Debug)]
pub struct IdentToken<'input> {
    pub span: Span,
    pub token_metadata: TokenMetadata<'input>,
    pub ident: &'input str,
}

#[derive(Clone, Debug)]
pub struct IntegerLiteral<'input> {
    pub span: Span,
    pub token_metadata: TokenMetadata<'input>,
    pub value: &'input str,
}

#[derive(Clone, Debug)]
pub struct FloatLiteral<'input> {
    pub span: Span,
    pub token_metadata: TokenMetadata<'input>,
    pub value: &'input str,
}

#[derive(Clone, Debug)]
pub struct StringLiteral<'input> {
    pub span: Span,
    pub token_metadata: TokenMetadata<'input>,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ListLiteral<'input> {
    pub span: Span,
    pub start_bracket: SimpleToken<'input>,
    pub values: Vec<ListLiteralValue<'input>>, // the SimpleToken is the comma
    pub end_bracket: SimpleToken<'input>,
}

#[derive(Clone, Debug)]
pub struct ListLiteralValue<'input> {
    pub span: Span,
    pub expr: Expr<'input>,
    pub comma: Option<SimpleToken<'input>>,
}

#[derive(Clone, Debug)]
pub struct SimpleToken<'input> {
    pub span: Span,
    pub token_metadata: TokenMetadata<'input>,
}

#[derive(Clone, Debug)]
pub struct NamespacePath<'input> {
    pub span: Span,
    pub initial_segments: Vec<NamespacePathSegment<'input>>,
    pub final_segment: IdentToken<'input>,
}

impl<'input> AttributeKind<'input> {
    pub fn token_meta<'a>(&'a self) -> &'a TokenMetadata<'input> {
        match self {
            AttributeKind::Ident(ident) => &ident.token_metadata,
            AttributeKind::String(string) => &string.token_metadata,
        }
    }
}

impl<'input> Metadata<'input> {
    pub fn token_metas<'a>(&'a self) -> impl Iterator<Item = &'a TokenMetadata<'input>> {
        std::iter::once(&self.start_paren.token_metadata)
            .chain(self.values.iter().flat_map(|value| value.token_metas()))
            .chain([&self.end_paren.token_metadata])
    }
}

impl<'input> MetadataValue<'input> {
    pub fn token_metas<'a>(&'a self) -> impl Iterator<Item = &'a TokenMetadata<'input>> {
        std::iter::once(&self.key.token_metadata)
            .chain(self.assignment.iter().flat_map(|(equals, expr)| {
                std::iter::once(&equals.token_metadata).chain(expr.kind.token_metas())
            }))
            .chain(
                self.comma
                    .as_ref()
                    .map(|token| &token.token_metadata),
            )
    }
}

mod token_metadata_iterator {
    use super::*;

    pub(super) enum Node<'a, 'input> {
        Direct(&'a TokenMetadata<'input>),
        Expr(&'a ExprKind<'input>),
        Type(&'a TypeKind<'input>),
    }

    pub(super) struct Iter<'a, 'input> {
        queue: Vec<Node<'a, 'input>>,
    }

    impl<'a, 'input> Iter<'a, 'input> {
        pub(super) fn new(initial: Node<'a, 'input>) -> Self {
            Self {
                queue: vec![initial],
            }
        }
    }

    impl<'a, 'input> Iterator for Iter<'a, 'input> {
        type Item = &'a TokenMetadata<'input>;

        fn next(&mut self) -> Option<Self::Item> {
            Some(match self.queue.pop()? {
                Node::Direct(token_metadata) => token_metadata,
                Node::Expr(ExprKind::Ident(tok)) => &tok.token_metadata,
                Node::Expr(ExprKind::Integer(tok)) => &tok.token_metadata,
                Node::Expr(ExprKind::Float(tok)) => &tok.token_metadata,
                Node::Expr(ExprKind::String(tok)) => &tok.token_metadata,
                Node::Expr(ExprKind::List(literal)) => {
                    self.queue
                        .push(Node::Direct(&literal.end_bracket.token_metadata));
                    for value in literal.values.iter().rev() {
                        if let Some(comma) = &value.comma {
                            self.queue.push(Node::Direct(&comma.token_metadata));
                        }
                        self.queue.push(Node::Expr(&value.expr.kind));
                    }
                    &literal.start_bracket.token_metadata
                }
                Node::Expr(ExprKind::Signed { sign, inner }) => {
                    self.queue.push(Node::Expr(&inner.kind));
                    &sign.token.token_metadata
                }
                Node::Type(TypeKind::Vector(typ)) => {
                    self.queue
                        .push(Node::Direct(&typ.end_bracket.token_metadata));
                    self.queue.push(Node::Type(&typ.inner_type.kind));
                    &typ.start_bracket.token_metadata
                }
                Node::Type(TypeKind::Array(typ)) => {
                    self.queue
                        .push(Node::Direct(&typ.end_bracket.token_metadata));
                    self.queue.push(Node::Expr(&typ.size.kind));
                    self.queue.push(Node::Direct(&typ.colon.token_metadata));
                    self.queue.push(Node::Type(&typ.inner_type.kind));
                    &typ.start_bracket.token_metadata
                }
                Node::Type(TypeKind::Path(typ)) => {
                    let mut result = &typ.final_segment.token_metadata;
                    for segment in typ.initial_segments.iter().rev() {
                        self.queue.push(Node::Direct(result));
                        self.queue
                            .push(Node::Direct(&segment.period.token_metadata));
                        result = &segment.ident.token_metadata;
                    }
                    result
                }
            })
        }
    }
}

impl<'input> ExprKind<'input> {
    pub fn token_metas<'a>(&'a self) -> impl Iterator<Item = &'a TokenMetadata<'input>> {
        token_metadata_iterator::Iter::new(token_metadata_iterator::Node::Expr(self))
    }
}

impl<'input> TypeKind<'input> {
    pub fn token_metas<'a>(&'a self) -> impl Iterator<Item = &'a TokenMetadata<'input>> {
        token_metadata_iterator::Iter::new(token_metadata_iterator::Node::Type(self))
    }
}

impl<'input> NamespacePath<'input> {
    pub fn token_metas<'a>(&'a self) -> impl Iterator<Item = &'a TokenMetadata<'input>> {
        self.initial_segments
            .iter()
            .flat_map(|segment| {
                [
                    &segment.ident.token_metadata,
                    &segment.period.token_metadata,
                ]
            })
            .chain([&self.final_segment.token_metadata])
    }
}

#[derive(Clone, Debug)]
pub struct NamespacePathSegment<'input> {
    pub span: Span,
    pub ident: IdentToken<'input>,
    pub period: SimpleToken<'input>,
}

pub trait CstNode {
    fn span(&self) -> Span;
}

macro_rules! cst_node {
    ($t:ident) => {
        impl<'input> CstNode for $t<'input> {
            fn span(&self) -> Span {
                self.span
            }
        }
    };
}

cst_node!(Schema);
cst_node!(Declaration);
cst_node!(RpcMethod);
cst_node!(Metadata);
cst_node!(MetadataValue);
cst_node!(FieldDeclaration);
cst_node!(EnumValDeclaration);
cst_node!(UnionValDeclaration);
cst_node!(Type);
cst_node!(VectorType);
cst_node!(ArrayType);
cst_node!(Expr);
cst_node!(Sign);
cst_node!(IntegerLiteral);
cst_node!(FloatLiteral);
cst_node!(StringLiteral);
cst_node!(ListLiteral);
cst_node!(ListLiteralValue);
cst_node!(SimpleToken);
cst_node!(IdentToken);
cst_node!(NamespacePath);
cst_node!(NamespacePathSegment);
