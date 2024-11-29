use std::collections::{btree_map, BTreeMap, HashSet};

use codespan::{FileId, Span};
use codespan_reporting::diagnostic::Label;
use indexmap::{map::Entry, IndexMap};
use planus_types::{
    ast::{self, FloatType, LiteralKind, MetadataValueKind, NamespacePath},
    intermediate::*,
};

use crate::{
    ctx::Ctx,
    error::ErrorKind,
    intermediate_language::checks::compatibility,
    util::sorted_map::{SortedMap, SortedSet},
};

pub struct Translator<'a> {
    ctx: &'a Ctx,
    reachability: SortedMap<FileId, SortedSet<FileId>>,
    ast_declarations: IndexMap<AbsolutePath, ast::Declaration>,
    declarations: IndexMap<AbsolutePath, Declaration>,
    namespaces: IndexMap<AbsolutePath, Namespace>,
    descriptions: Vec<TypeDescription>,
}

#[derive(Clone)]
enum TypeDescription {
    Table,
    Struct { size: u32, alignment: u32 },
    Enum(Enum),
    Union,
    RpcService,
}

// do not start translating any declarations, until all declarations have been collected
// then:
//  1) get names and type descriptions for all declarations
//     - enums are translated immediately since they are self-contained and are needed
//       to translate table fields
//  2) do preliminary translation with wrong sizes which resolves
//     what each ast::NamespacePath points to
//  3) do topological sort of all structs to get sizes
//  4) use this information to update all sizes

fn default_docstring_for_namespace(path: &AbsolutePath) -> String {
    if path == &AbsolutePath::ROOT_PATH {
        "The root namespace".to_string()
    } else {
        format!("The namespace `{path}`")
    }
}

impl<'a> Translator<'a> {
    pub fn new(ctx: &'a Ctx, reachability: SortedMap<FileId, SortedSet<FileId>>) -> Self {
        Self {
            ctx,
            reachability,
            ast_declarations: Default::default(),
            declarations: Default::default(),
            descriptions: Default::default(),
            namespaces: Default::default(),
        }
    }

    pub fn add_schema(&mut self, schema: &ast::Schema) {
        compatibility::check_ast(self.ctx, schema);
        let mut namespace_path = if let Some((_span, path)) = &schema.namespace {
            self.ctx.absolute_path_from_parts(&path.parts)
        } else {
            AbsolutePath::ROOT_PATH
        };

        let namespace_entry = self.namespaces.entry(namespace_path.clone());
        let mut namespace_index = namespace_entry.index();
        let namespace = namespace_entry.or_default();
        namespace.spans.push((
            schema.file_id,
            schema.namespace.as_ref().map(|(span, _)| *span),
        ));
        namespace
            .docstrings
            .docstrings
            .extend(schema.docstrings.docstrings.iter().cloned());
        namespace
            .docstrings
            .locations
            .extend(schema.docstrings.locations.iter().cloned());
        namespace.docstrings.default_docstring = default_docstring_for_namespace(&namespace_path);

        for decl in schema.type_declarations.values() {
            let name = self.ctx.resolve_identifier(decl.identifier.value);
            match self
                .ast_declarations
                .entry(namespace_path.clone_push(&name))
            {
                Entry::Occupied(entry) => {
                    self.ctx.emit_error(
                        ErrorKind::TYPE_DEFINED_TWICE,
                        [
                            Label::secondary(entry.get().file_id, entry.get().definition_span)
                                .with_message("first definition was here"),
                            Label::secondary(schema.file_id, decl.definition_span)
                                .with_message("second definition was here"),
                        ],
                        Some("Overlapping declarations"),
                    );
                    continue;
                }
                Entry::Vacant(entry) => {
                    namespace
                        .declaration_ids
                        .insert(name, DeclarationIndex(entry.index()));
                    entry.insert(decl.clone());
                }
            }
        }
        for decl in schema.type_declarations.values() {
            self.descriptions.push(match &decl.kind {
                ast::TypeDeclarationKind::Table(_) => TypeDescription::Table,
                ast::TypeDeclarationKind::Struct(_) => TypeDescription::Struct {
                    size: u32::MAX,
                    alignment: u32::MAX,
                },
                ast::TypeDeclarationKind::Enum(decl) => {
                    TypeDescription::Enum(self.translate_enum(schema.file_id, decl))
                }
                ast::TypeDeclarationKind::Union(_) => TypeDescription::Union,
                ast::TypeDeclarationKind::RpcService(_) => TypeDescription::RpcService,
            })
        }

        while let Some(last) = namespace_path.pop() {
            match self.namespaces.entry(namespace_path) {
                Entry::Occupied(mut entry) => {
                    entry
                        .get_mut()
                        .child_namespaces
                        .insert(last, NamespaceIndex(namespace_index));
                    break;
                }
                Entry::Vacant(entry) => {
                    namespace_path = entry.key().clone();
                    let next_namespace_index = entry.index();
                    let entry = entry.insert(Namespace::default());
                    entry.docstrings.locations.push(format!(
                        "* File `{}`",
                        self.ctx.get_filename(schema.file_id).display()
                    ));
                    entry
                        .child_namespaces
                        .insert(last, NamespaceIndex(namespace_index));
                    entry.docstrings.default_docstring =
                        default_docstring_for_namespace(&namespace_path);
                    namespace_index = next_namespace_index;
                }
            }
        }
    }

    fn lookup_path(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        namespace_path: &NamespacePath,
    ) -> Option<TypeKind> {
        let absolute_path = self.ctx.absolute_path_from_parts(&namespace_path.parts);
        let mut relative_path = current_namespace.clone();
        relative_path.0.extend(absolute_path.0.iter().cloned());
        let mut hints: Vec<Label<FileId>> = Vec::new();
        let mut seen_hints = HashSet::new();
        for path in [relative_path, absolute_path] {
            if let Some((index, _name, decl)) = self.ast_declarations.get_full(&path) {
                let result = match self.descriptions[index] {
                    TypeDescription::Table => TypeKind::Table(DeclarationIndex(index)),
                    TypeDescription::Struct { .. } => {
                        TypeKind::SimpleType(SimpleType::Struct(DeclarationIndex(index)))
                    }
                    TypeDescription::Enum { .. } => {
                        TypeKind::SimpleType(SimpleType::Enum(DeclarationIndex(index)))
                    }
                    TypeDescription::Union => TypeKind::Union(DeclarationIndex(index)),
                    TypeDescription::RpcService => {
                        self.ctx.emit_error(
                            ErrorKind::TYPE_ERROR,
                            [
                                Label::primary(current_file_id, namespace_path.span)
                                    .with_message("Rpc services cannot be used as types"),
                                Label::secondary(decl.file_id, decl.definition_span)
                                    .with_message("Service was defined here"),
                            ],
                            Some(&format!(
                                "Cannot use the RpcService {path} in a type context"
                            )),
                        );
                        return None;
                    }
                };
                if self
                    .reachability
                    .get(&current_file_id)
                    .is_some_and(|reachable| reachable.contains(&decl.file_id))
                {
                    return Some(result);
                } else if seen_hints.insert((decl.file_id, decl.definition_span)) {
                    hints.push(
                        Label::secondary(decl.file_id, decl.definition_span).with_message(format!(
                            "Perhaps you meant to include {}",
                            self.ctx.get_filename(decl.file_id).display()
                        )),
                    );
                }
            }
        }
        self.ctx.emit_error(
            ErrorKind::UNKNOWN_IDENTIFIER,
            std::iter::once(Label::primary(current_file_id, namespace_path.span)).chain(hints),
            Some("Unknown type"),
        );
        None
    }

    fn translate_alignment(
        &self,
        file_id: FileId,
        metadata_span: Span,
        literal: &ast::IntegerLiteral,
    ) -> Option<u32> {
        let value = self.translate_integer_generic::<u32>(
            file_id,
            literal.span,
            literal.is_negative,
            &literal.value,
        )?;
        if value.is_power_of_two() {
            Some(value)
        } else {
            self.ctx.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [Label::primary(file_id, metadata_span)],
                Some("Alignment must be a power of two"),
            );
            None
        }
    }

    fn translate_integer(
        &self,
        file_id: FileId,
        span: Span,
        is_negative: bool,
        v: &str,
        type_: &ast::IntegerType,
    ) -> Option<IntegerLiteral> {
        use IntegerLiteral::*;

        Some(match type_ {
            ast::IntegerType::U8 => {
                U8(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::U16 => {
                U16(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::U32 => {
                U32(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::U64 => {
                U64(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::I8 => {
                I8(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::I16 => {
                I16(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::I32 => {
                I32(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
            ast::IntegerType::I64 => {
                I64(self.translate_integer_generic(file_id, span, is_negative, v)?)
            }
        })
    }

    fn translate_integer_generic<
        T: num_traits::CheckedAdd
            + num_traits::CheckedSub
            + num_traits::CheckedMul
            + num_traits::CheckedNeg
            + num_traits::NumCast
            + num_traits::Bounded
            + std::fmt::Display,
    >(
        &self,
        file_id: FileId,
        span: Span,
        is_negative: bool,
        v: &str,
    ) -> Option<T> {
        let mut v = v.as_bytes();
        let mut base = 10;
        if v.starts_with(b"0x") {
            v = &v[2..];
            base = 16;
        }
        let base_t = T::from(base).unwrap();
        let mut result = T::from(0u8).unwrap();
        for c in v {
            let cur = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' if base == 16 => c - b'a' + 0xa,
                b'A'..=b'F' if base == 16 => c - b'A' + 0xa,
                _ => {
                    // TODO error message
                    // Can it even happen?
                    return None;
                }
            };

            match result.checked_mul(&base_t).and_then(|v| {
                if is_negative {
                    v.checked_sub(&T::from(cur).unwrap())
                } else {
                    v.checked_add(&T::from(cur).unwrap())
                }
            }) {
                Some(r) => result = r,
                None => {
                    self.ctx.emit_error(
                        ErrorKind::NUMERICAL_RANGE_ERROR,
                        [Label::primary(file_id, span)],
                        Some(&format!(
                            "Integer is out of range for type {} (range is [{}; {}])",
                            std::any::type_name::<T>(),
                            T::min_value(),
                            T::max_value(),
                        )),
                    );
                    return None;
                }
            }
        }

        Some(result)
    }

    fn translate_float(
        &self,
        _span: Span,
        _is_negative: bool,
        v: &str,
        type_: &ast::FloatType,
    ) -> Option<FloatLiteral> {
        use FloatLiteral::*;

        // TODO error messages
        // TODO: hex parsing and other formats
        Some(match type_ {
            ast::FloatType::F32 => F32(v.parse().unwrap()),
            ast::FloatType::F64 => F64(v.parse().unwrap()),
        })
    }

    fn translate_literal(
        &self,
        current_file_id: FileId,
        literal: &ast::Literal,
        type_: &Type,
    ) -> Option<Literal> {
        use planus_types::intermediate::{SimpleType::*, TypeKind::*};

        match (&literal.kind, &type_.kind) {
            (LiteralKind::Bool(value), SimpleType(Bool)) => Some(Literal::Bool(*value)),
            (LiteralKind::Integer { is_negative, value }, SimpleType(Integer(type_))) => self
                .translate_integer(current_file_id, literal.span, *is_negative, value, type_)
                .map(Literal::Int),
            (LiteralKind::Integer { is_negative, value }, SimpleType(Enum(decl_index))) => {
                match &self.descriptions[decl_index.0] {
                    TypeDescription::Enum(decl) => {
                        let int = self.translate_integer(
                            current_file_id,
                            literal.span,
                            *is_negative,
                            value,
                            &decl.type_,
                        )?;
                        if let Some(variant_index) = decl.variants.get_index_of(&int) {
                            Some(Literal::EnumTag {
                                variant_index,
                                value: int,
                            })
                        } else {
                            self.ctx.emit_error(
                                ErrorKind::TYPE_ERROR,
                                std::iter::once(
                                    Label::primary(current_file_id, literal.span).with_message(
                                        "Expression does not map to a valid enum variant",
                                    ),
                                ),
                                None,
                            );
                            // We try to output as valid data as possible to prevent bogus error
                            // messages from being emitted on account of the bad data returned
                            // here
                            Some(decl.variants.keys().enumerate().next().map_or_else(
                                || Literal::EnumTag {
                                    variant_index: 0,
                                    value: IntegerLiteral::U8(0),
                                },
                                |(variant_index, &value)| Literal::EnumTag {
                                    variant_index,
                                    value,
                                },
                            ))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            (
                LiteralKind::Float { is_negative, value }
                | LiteralKind::Integer { is_negative, value },
                SimpleType(Float(type_)),
            ) => self
                .translate_float(literal.span, *is_negative, value, type_)
                .map(Literal::Float),
            (LiteralKind::String(s), String) => Some(Literal::String(s.clone())),
            (LiteralKind::List(literals), Vector(type_)) => {
                let mut out = Vec::new();
                for literal in literals.iter() {
                    out.push(self.translate_literal(current_file_id, literal, type_)?);
                }
                Some(Literal::Vector(out))
            }
            (LiteralKind::List(literals), Array(_type_, size)) => {
                if literals.len() == *size as usize {
                    let mut out = Vec::new();
                    for literal in literals.iter() {
                        out.push(self.translate_literal(current_file_id, literal, type_)?);
                    }
                    Some(Literal::Array(out))
                } else {
                    self.ctx.emit_error(
                        ErrorKind::TYPE_ERROR,
                        std::iter::once(Label::primary(current_file_id, literal.span)),
                        Some("Array literal does not have the correct length"),
                    );
                    None
                }
            }
            (LiteralKind::Constant(s), _) => {
                if let SimpleType(Enum(decl_index)) = &type_.kind {
                    if let TypeDescription::Enum(decl) = &self.descriptions[decl_index.0] {
                        if let Some((variant_index, (key, _value))) = decl
                            .variants
                            .iter()
                            .enumerate()
                            .find(|(_variant_index, (_key, variant))| &variant.name == s)
                        {
                            return Some(Literal::EnumTag {
                                variant_index,
                                value: *key,
                            });
                        }
                    }
                }

                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    std::iter::once(Label::primary(current_file_id, literal.span)),
                    Some(&format!("Unknown constant {s:?}")),
                );
                None
            }
            _ => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    std::iter::once(
                        Label::primary(current_file_id, literal.span)
                            .with_message("Expression not of the correct type"),
                    ),
                    None,
                );
                None
            }
        }
    }

    fn translate_type(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        type_: &ast::Type,
    ) -> Option<Type> {
        match &type_.kind {
            ast::TypeKind::Builtin(inner_type) => Some(Type {
                span: type_.span,
                kind: inner_type.into(),
            }),
            ast::TypeKind::Vector { inner_type } => Some(Type {
                span: type_.span,
                kind: TypeKind::Vector(Box::new(self.translate_type(
                    current_namespace,
                    current_file_id,
                    inner_type,
                )?)),
            }),
            ast::TypeKind::Array { inner_type, size } => Some(Type {
                span: type_.span,
                kind: TypeKind::Array(
                    Box::new(self.translate_type(
                        current_namespace,
                        current_file_id,
                        inner_type,
                    )?),
                    *size,
                ),
            }),
            ast::TypeKind::Path(path) => Some(Type {
                span: type_.span,
                kind: self.lookup_path(current_namespace, current_file_id, path)?,
            }),
            ast::TypeKind::Invalid => None,
        }
    }

    fn translate_struct(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        decl: &ast::Struct,
    ) -> Struct {
        for m in &decl.metadata.values {
            match m.kind {
                MetadataValueKind::ForceAlign(_) => (), // Handled elsewhere
                _ => {
                    self.emit_metadata_support_error(
                        current_file_id,
                        m,
                        "structs",
                        m.kind.accepted_on_structs(),
                    );
                }
            }
        }

        let fields = decl
            .fields
            .iter()
            .filter_map(|(ident, field)| {
                self.translate_struct_field(current_namespace, current_file_id, field, ident)
            })
            .collect();
        Struct {
            fields,
            size: u32::MAX,
            alignment: u32::MAX,
        }
    }

    fn translate_struct_field(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        field: &ast::StructField,
        ident: &string_interner::symbol::SymbolU32,
    ) -> Option<(String, StructField)> {
        let type_ = self.translate_type(current_namespace, current_file_id, &field.type_)?;
        if let Some(assignment) = &field.assignment {
            self.ctx.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                std::iter::once(
                    Label::primary(current_file_id, assignment.span)
                        .with_message("Struct fields cannot have default values"),
                ),
                None,
            )
        }

        for m in &field.metadata.values {
            self.emit_metadata_support_error(
                current_file_id,
                m,
                "struct fields",
                m.kind.accepted_on_struct_fields(),
            );
        }
        match type_.kind {
            TypeKind::Table(_) => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    [Label::primary(current_file_id, type_.span)
                        .with_message("Tables in structs are not supported")],
                    Some("Only simple types are permitted in structs"),
                );
                None
            }
            TypeKind::Union(_) => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    [Label::primary(current_file_id, type_.span)
                        .with_message("Unions in structs are not supported")],
                    Some("Only simple types are permitted in structs"),
                );
                None
            }
            TypeKind::Vector(_) => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    [Label::primary(current_file_id, type_.span)
                        .with_message("Vectors in structs are not supported")],
                    Some("Only simple types are permitted in structs"),
                );
                None
            }
            TypeKind::String => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    [Label::primary(current_file_id, type_.span)
                        .with_message("Strings in structs are not supported")],
                    Some("Only simple types are permitted in structs"),
                );
                None
            }
            TypeKind::SimpleType(type_) => Some((
                self.ctx.resolve_identifier(*ident),
                StructField {
                    type_,
                    offset: u32::MAX,
                    size: u32::MAX,
                    padding_after_field: u32::MAX,
                    docstrings: field.docstrings.clone(),
                },
            )),
            TypeKind::Array(_, _) => {
                self.ctx.emit_error(
                    ErrorKind::TYPE_ERROR,
                    std::iter::once(Label::primary(current_file_id, field.type_.span)),
                    Some("Arrays are not currently supported in planus"),
                );
                None
            }
        }
    }

    fn translate_decl(
        &self,
        id: usize,
        current_namespace: &AbsolutePath,
        current_namespace_index: NamespaceIndex,
        decl: &ast::Declaration,
    ) -> Declaration {
        let current_file_id = decl.file_id;
        let definition_span = decl.definition_span;
        let kind =
            match &decl.kind {
                ast::TypeDeclarationKind::Table(decl) => DeclarationKind::Table(
                    self.translate_table(current_namespace, current_file_id, definition_span, decl),
                ),
                ast::TypeDeclarationKind::Struct(decl) => DeclarationKind::Struct(
                    self.translate_struct(current_namespace, current_file_id, decl),
                ),
                ast::TypeDeclarationKind::Enum(_) => match &self.descriptions[id] {
                    TypeDescription::Enum(decl) => DeclarationKind::Enum(decl.clone()),
                    _ => unreachable!(),
                },
                ast::TypeDeclarationKind::Union(decl) => DeclarationKind::Union(
                    self.translate_union(current_namespace, current_file_id, decl),
                ),
                ast::TypeDeclarationKind::RpcService(decl) => DeclarationKind::RpcService(
                    self.translate_rpc_service(current_namespace, current_file_id, decl),
                ),
            };

        Declaration {
            namespace_id: current_namespace_index,
            definition_span: decl.definition_span,
            file_id: current_file_id,
            kind,
            docstrings: decl.docstrings.clone(),
        }
    }

    fn check_valid_default_literal(
        &self,
        current_file_id: FileId,
        default_value: &Literal,
        assignment_span: Span,
    ) {
        match default_value {
            Literal::Bool(_)
            | Literal::String(_)
            | Literal::Int(_)
            | Literal::Float(_)
            | Literal::EnumTag { .. }
            | Literal::Array(_) => (),
            Literal::Vector(l) => {
                if !l.is_empty() {
                    self.ctx.emit_error(
                        ErrorKind::MISC_SEMANTIC_ERROR,
                        [
                            Label::primary(current_file_id, assignment_span).with_message(
                                "Vectors only support default values of null or the empty list",
                            ),
                        ],
                        Some("Unsupported default value"),
                    );
                }
            }
        }
    }

    fn check_valid_vector_type(&self, current_file_id: FileId, type_: &Type) {
        match type_.kind {
            TypeKind::Table(_) => (),
            TypeKind::SimpleType(_) => (),
            TypeKind::String => (),

            TypeKind::Vector(_) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Vectors in vectors are not currently supported")],
                Some("Unsupported type"),
            ),
            TypeKind::Union(_) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Unions in vectors are not currently supported")],
                Some("Unsupported type"),
            ),
            TypeKind::Array(_, _) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Arrays in vectors are not currently supported")],
                Some("Unsupported type"),
            ),
        }
    }

    fn check_valid_table_field_type(&self, current_file_id: FileId, type_: &Type) {
        match &type_.kind {
            TypeKind::Table(_)
            | TypeKind::Union(_)
            | TypeKind::SimpleType(_)
            | TypeKind::String => (),
            TypeKind::Vector(type_) => self.check_valid_vector_type(current_file_id, type_),
            TypeKind::Array(_, _) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Arrays in tables are not supported")],
                Some("Unsupported type"),
            ),
        }
    }

    fn check_valid_union_variant_type(
        &self,
        current_file_id: FileId,
        type_: &Type,
        has_ident: bool,
    ) {
        match type_.kind {
            TypeKind::Table(_) | TypeKind::SimpleType(SimpleType::Struct(_)) => (),
            TypeKind::String if has_ident => (),
            TypeKind::String => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Strings in unions without variant names are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::Union(_) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Unions in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::SimpleType(SimpleType::Enum(_)) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Enums in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::Vector(_) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Vectors in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::Array(_, _) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Arrays in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::SimpleType(SimpleType::Bool) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Bools in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::SimpleType(SimpleType::Integer(_)) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Integers in unions are not supported")],
                Some("Unsupported type"),
            ),
            TypeKind::SimpleType(SimpleType::Float(_)) => self.ctx.emit_error(
                ErrorKind::TYPE_ERROR,
                [Label::primary(current_file_id, type_.span)
                    .with_message("Floats in unions are not supported")],
                Some("Unsupported type"),
            ),
        }
    }

    fn translate_table_field(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        field: &ast::StructField,
        next_vtable_index: &mut u32,
        max_vtable_size: &mut u32,
        has_id_error: &mut bool,
    ) -> Option<TableField> {
        let type_ = self.translate_type(current_namespace, current_file_id, &field.type_)?;
        self.check_valid_table_field_type(current_file_id, &type_);
        let mut default_value = self.default_value_for_type(&type_);
        let mut explicit_null = false;
        if let Some(assignment) = field.assignment.as_ref() {
            if matches!(assignment.kind, LiteralKind::Null) {
                explicit_null = true;
                default_value = None;
            } else {
                default_value = self.translate_literal(current_file_id, assignment, &type_);
                if let Some(default_value) = &default_value {
                    self.check_valid_default_literal(
                        current_file_id,
                        default_value,
                        assignment.span,
                    );
                }
            }
        };
        let mut required = false;
        let mut deprecated = false;
        let mut vtable_index = *next_vtable_index;

        for m in &field.metadata.values {
            match &m.kind {
                MetadataValueKind::Required => {
                    if type_.kind.is_scalar() {
                        self.ctx.emit_error(
                            ErrorKind::MISC_SEMANTIC_ERROR,
                            [
                                Label::secondary(current_file_id, type_.span).with_message(
                                    "only non-scalar types support the 'required' attribute",
                                ),
                                Label::primary(current_file_id, m.span)
                                    .with_message("required attribute was here"),
                            ],
                            Some("Unsupported required attribute"),
                        );
                    } else if explicit_null {
                        self.ctx.emit_error(
                            ErrorKind::MISC_SEMANTIC_ERROR,
                            [
                                Label::secondary(current_file_id, m.span)
                                    .with_message("field was declared required here"),
                                Label::primary(
                                    current_file_id,
                                    field.assignment.as_ref().unwrap().span,
                                )
                                .with_message("field was declared optional here"),
                            ],
                            Some("Cannot setup field as both required and optional"),
                        );
                    } else {
                        required = true;
                    }
                }
                MetadataValueKind::Deprecated => deprecated = true,
                MetadataValueKind::Id(ast::IntegerLiteral {
                    span,
                    value,
                    is_negative,
                }) => {
                    if let Some(new_index) = self.translate_integer_generic::<u32>(
                        current_file_id,
                        *span,
                        *is_negative,
                        value,
                    ) {
                        if matches!(type_.kind, TypeKind::Union(_)) {
                            if new_index > 0 {
                                // The specification says that the id assignment for unions specifies the second
                                // of the two IDs, while we use the first in our code
                                vtable_index = new_index - 1;
                            } else {
                                self.ctx.emit_error(
                                    ErrorKind::MISC_SEMANTIC_ERROR,
                                    [Label::primary(current_file_id, *span).with_message(
                                        "This attribute implies the key will have an id of -1",
                                    )],
                                    Some("Id assignments for fields of union type specify the id of the value."),
                                );
                                vtable_index = 0;
                            }
                        } else {
                            vtable_index = new_index;
                        }
                    } else {
                        *has_id_error = true;
                    }
                }
                _ => {
                    self.emit_metadata_support_error(
                        current_file_id,
                        m,
                        "table fields",
                        m.kind.accepted_on_table_fields(),
                    );
                }
            }
        }

        if matches!(&type_.kind, TypeKind::Union(_)) {
            *next_vtable_index = vtable_index + 2;
        } else {
            *next_vtable_index = vtable_index + 1;
        }

        *max_vtable_size = (*max_vtable_size).max(2 * *next_vtable_index + 4);

        let assign_mode = match (required, explicit_null, default_value) {
            (true, false, None) => AssignMode::Required,
            (false, false, None) if type_.kind.is_enum() => {
                self.ctx.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    [Label::primary(current_file_id, field.span)],
                    Some("Enums must either be required, have an explicit default value or contain 0 as a variant"),
                );
                AssignMode::Optional
            }
            (false, _, None) => AssignMode::Optional,
            (false, false, Some(default_value)) => AssignMode::HasDefault(default_value),
            (true, true, _) | (true, _, Some(_)) => {
                self.ctx.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    [Label::primary(current_file_id, field.span)],
                    Some("Fields cannot set field as required while having a default value"),
                );
                AssignMode::Optional
            }
            (_, true, Some(_)) => unreachable!(),
        };

        Some(TableField {
            type_,
            span: field.span,
            assign_mode,
            vtable_index,
            object_value_size: u32::MAX,
            object_tag_size: u32::MAX,
            object_alignment_mask: u32::MAX,
            object_alignment: u32::MAX,
            deprecated,
            docstrings: field.docstrings.clone(),
        })
    }

    fn translate_rpc_service(
        &self,
        _current_namespace: &AbsolutePath,
        _current_file_id: FileId,
        _decl: &ast::RpcService,
    ) -> RpcService {
        RpcService {
            methods: Default::default(),
        }
    }

    fn emit_metadata_support_error(
        &self,
        current_file_id: FileId,
        m: &ast::MetadataValue,
        kind: &str,
        works_upstream: bool,
    ) {
        let msg = match (works_upstream, m.kind.is_supported()) {
            (true, true) => format!("Metadata attribute is not currently supported on {kind}"),
            (true, false) => "Metadata attribute is not currently supported".to_string(),
            (false, true) => format!("Metadata attribute does not make sense on {kind}"),
            (false, false) => format!("Metadata attribute does not make sense on {kind} (but is additionally not supported in planus)"),
        };

        self.ctx.emit_error(
            ErrorKind::MISC_SEMANTIC_ERROR,
            [Label::primary(current_file_id, m.span)],
            Some(&msg),
        );
    }

    fn translate_table(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        definition_span: Span,
        decl: &ast::Struct,
    ) -> Table {
        let mut next_vtable_index = 0u32;
        let mut max_vtable_size = 4u32;

        for m in &decl.metadata.values {
            self.emit_metadata_support_error(
                current_file_id,
                m,
                "tables",
                m.kind.accepted_on_tables(),
            );
        }

        let mut has_id_error = false;
        let mut first_with_id = None;
        let mut first_without_id = None;
        for field in decl.fields.values() {
            let mut cur_id = None;
            for m in &field.metadata.values {
                if let MetadataValueKind::Id(_) = &m.kind {
                    if let Some(span) = cur_id {
                        self.ctx.emit_error(
                            ErrorKind::MISC_SEMANTIC_ERROR,
                            [
                                Label::primary(current_file_id, span)
                                    .with_message("First attribute was here"),
                                Label::primary(current_file_id, m.span)
                                    .with_message("Second attribute was here"),
                            ],
                            Some("Field cannot have multiple id attributes"),
                        );
                        has_id_error = true;
                    } else {
                        cur_id = Some(m.span);
                    }
                }
            }

            if cur_id.is_some() {
                first_with_id.get_or_insert(field.span);
            } else {
                first_without_id.get_or_insert(field.span);
            }
        }

        if let (Some(first_with_id), Some(first_without_id)) = (first_with_id, first_without_id) {
            self.ctx.emit_error(
                ErrorKind::MISC_SEMANTIC_ERROR,
                [
                    Label::primary(current_file_id, first_with_id)
                        .with_message("First field with an id assignment was here"),
                    Label::primary(current_file_id, first_without_id)
                        .with_message("First field without an id assignment was here"),
                    Label::secondary(current_file_id, definition_span).with_message("Offending table was here")
                ],
                Some("Table contains both fields with and without the id assignments, which is disallowed"),
            );
        }

        let fields = decl
            .fields
            .iter()
            .filter_map(|(ident, field)| {
                Some((
                    self.ctx.resolve_identifier(*ident),
                    self.translate_table_field(
                        current_namespace,
                        current_file_id,
                        field,
                        &mut next_vtable_index,
                        &mut max_vtable_size,
                        &mut has_id_error,
                    )?,
                ))
            })
            .collect::<IndexMap<_, _>>();

        let mut seen_vtable_ids: BTreeMap<u32, Span> = BTreeMap::new();
        let mut insert = |vtable_index, span| match seen_vtable_ids.entry(vtable_index) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(span);
            }
            btree_map::Entry::Occupied(entry) => {
                if !has_id_error {
                    self.ctx.emit_error(
                        ErrorKind::MISC_SEMANTIC_ERROR,
                        [
                            Label::primary(current_file_id, *entry.get())
                                .with_message("First id assignment was here"),
                            Label::primary(current_file_id, span)
                                .with_message("Second id assignment was here"),
                        ],
                        Some(&format!("Overlapping id assignments for id {vtable_index}")),
                    );
                    has_id_error = true;
                }
            }
        };

        for field in fields.values() {
            insert(field.vtable_index, field.span);
            if matches!(field.type_.kind, TypeKind::Union(_)) {
                insert(field.vtable_index + 1, field.span);
            }
        }

        let mut next_expected_id = 0;
        for &key in seen_vtable_ids.keys() {
            if key != next_expected_id {
                let msg = if key == next_expected_id + 1 {
                    format!("Table contains non-consecutive ids. Missing id {next_expected_id}")
                } else {
                    format!(
                        "Table contains non-consecutive ids. Missing ids {}..{}",
                        next_expected_id,
                        key - 1
                    )
                };
                if !has_id_error {
                    self.ctx.emit_error(
                        ErrorKind::MISC_SEMANTIC_ERROR,
                        [Label::primary(current_file_id, definition_span)],
                        Some(&msg),
                    );
                }
            }
            next_expected_id = key + 1;
        }

        Table {
            fields,
            alignment_order: Vec::new(),
            max_size: u32::MAX,
            max_vtable_size,
            max_alignment: u32::MAX,
        }
    }

    fn translate_enum(&self, current_file_id: FileId, decl: &ast::Enum) -> Enum {
        let alignment = decl.type_.byte_size();
        for m in &decl.metadata.values {
            self.emit_metadata_support_error(
                current_file_id,
                m,
                "enums",
                m.kind.accepted_on_enums(),
            );
        }

        let mut variants: IndexMap<IntegerLiteral, EnumVariant> = IndexMap::new();
        let mut next_value = IntegerLiteral::default_value_from_type(&decl.type_);
        for (ident, variant) in decl.variants.iter() {
            let mut value = next_value;
            let name = self.ctx.resolve_identifier(*ident);
            if let Some(assignment) = &variant.value {
                if let Some(v) = self.translate_integer(
                    current_file_id,
                    assignment.span,
                    assignment.is_negative,
                    &assignment.value,
                    &decl.type_,
                ) {
                    value = v;
                } else {
                    continue;
                };
            }
            match variants.entry(value) {
                Entry::Occupied(entry) => {
                    self.ctx.emit_error(
                        ErrorKind::MISC_SEMANTIC_ERROR,
                        [
                            Label::primary(current_file_id, entry.get().span)
                                .with_message("First variant was here"),
                            Label::primary(current_file_id, variant.span)
                                .with_message("Second variant was here"),
                        ],
                        Some(&format!(
                            "Enum uses the value {value} for multiple variants"
                        )),
                    );
                }
                Entry::Vacant(entry) => {
                    entry.insert(EnumVariant {
                        span: variant.span,
                        name,
                        docstrings: variant.docstrings.clone(),
                    });
                }
            }
            next_value = value.next();
        }
        Enum {
            variants,
            type_: decl.type_,
            alignment,
        }
    }

    pub(crate) fn translate_union(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        decl: &ast::Union,
    ) -> Union {
        for m in &decl.metadata.values {
            self.emit_metadata_support_error(
                current_file_id,
                m,
                "unions",
                m.kind.accepted_on_unions(),
            );
        }

        let variants = decl
            .variants
            .values()
            .filter_map(|variant| {
                let type_ =
                    self.translate_type(current_namespace, current_file_id, &variant.type_)?;
                self.check_valid_union_variant_type(
                    current_file_id,
                    &type_,
                    variant.ident.is_some(),
                );

                let name = if let Some(ident) = variant.ident {
                    self.ctx.resolve_identifier(ident.value)
                } else {
                    // TODO: make sure this is unique
                    match &variant.type_.kind {
                        ast::TypeKind::Path(p) => {
                            self.ctx.resolve_identifier(*p.parts.last().unwrap())
                        }
                        _ => String::default(),
                    }
                };

                Some((
                    name,
                    UnionVariant {
                        type_,
                        docstrings: variant.docstrings.clone(),
                    },
                ))
            })
            .collect();
        Union { variants }
    }

    pub fn resolve_struct_sizes(
        &mut self,
        parents: &mut IndexMap<usize, ()>,
        index: usize,
    ) -> Option<(u32, u32)> {
        match &self.descriptions[index] {
            TypeDescription::Table => return None,
            TypeDescription::Struct { size, .. } if size == &u32::MAX => (),
            TypeDescription::Struct { size, alignment } => return Some((*size, *alignment)),
            TypeDescription::Enum(decl) => return Some((decl.type_.byte_size(), decl.alignment)),
            TypeDescription::Union => return None,
            TypeDescription::RpcService => return None,
        }

        match parents.entry(index) {
            Entry::Occupied(entry) => {
                let entry_index = entry.index();
                self.ctx.emit_error(
                    ErrorKind::MISC_SEMANTIC_ERROR,
                    parents
                        .keys()
                        .skip(entry_index)
                        .cloned()
                        .zip(
                            parents
                                .keys()
                                .skip(entry_index + 1)
                                .cloned()
                                .chain(std::iter::once(index)),
                        )
                        .map(|(a, b)| {
                            Label::secondary(
                                self.ast_declarations[a].file_id,
                                self.ast_declarations[a].definition_span,
                            )
                            .with_message(if a == b { format!(
                                "{} contains itself directly",
                                self.ast_declarations.get_index(a).unwrap().0,
                            )} else {format!(
                                "{} contains itself through {}",
                                self.ast_declarations.get_index(a).unwrap().0,
                                self.ast_declarations.get_index(b).unwrap().0
                            )})
                        }).take(1),
                    Some("Structs are not allowed to contain themselves, as it implies infinite size"),
                );
                return None;
            }
            Entry::Vacant(entry) => {
                entry.insert(());
            }
        }

        macro_rules! get_struct_decl {
            () => {
                if let DeclarationKind::Struct(decl) = &mut self.declarations[index].kind {
                    decl
                } else {
                    panic!("BUG")
                }
            };
        }

        macro_rules! get_ast_decl {
            () => {{
                let ast_decl = &self.ast_declarations[index];
                let ast_kind = if let ast::TypeDeclarationKind::Struct(decl) = &ast_decl.kind {
                    decl
                } else {
                    panic!("BUG")
                };
                (ast_decl, ast_kind)
            }};
        }

        fn round_up(value: u32, alignment: u32) -> u32 {
            value.div_ceil(alignment) * alignment
        }

        let mut offset = 0;
        let mut max_alignment = 1;
        let mut max_alignment_span = None;
        for field_id in 0..get_struct_decl!().fields.len() {
            let (cur_size, cur_alignment) = match &get_struct_decl!().fields[field_id].type_ {
                SimpleType::Struct(index) | SimpleType::Enum(index) => {
                    let index = index.0;
                    match self.resolve_struct_sizes(parents, index) {
                        Some(sizes) => sizes,
                        None => {
                            parents.pop();
                            return None;
                        }
                    }
                }
                SimpleType::Bool
                | SimpleType::Integer(ast::IntegerType::I8)
                | SimpleType::Integer(ast::IntegerType::U8) => (1, 1),
                SimpleType::Integer(ast::IntegerType::I16)
                | SimpleType::Integer(ast::IntegerType::U16) => (2, 2),
                SimpleType::Integer(ast::IntegerType::I32)
                | SimpleType::Integer(ast::IntegerType::U32)
                | SimpleType::Float(FloatType::F32) => (4, 4),
                SimpleType::Integer(ast::IntegerType::I64)
                | SimpleType::Integer(ast::IntegerType::U64)
                | SimpleType::Float(FloatType::F64) => (8, 8),
            };

            let (_ast_decl, ast_kind) = get_ast_decl!();
            offset = round_up(offset, cur_alignment);
            let decl = get_struct_decl!();
            decl.fields[field_id].offset = offset;
            decl.fields[field_id].size = cur_size;
            if field_id > 0 {
                decl.fields[field_id - 1].padding_after_field =
                    offset - decl.fields[field_id - 1].offset - decl.fields[field_id - 1].size;
            }
            offset += cur_size;
            if max_alignment_span.is_none() || max_alignment < cur_alignment {
                max_alignment_span = Some(ast_kind.fields[field_id].type_.span);
                max_alignment = cur_alignment;
            }
        }

        let (ast_decl, ast_kind) = get_ast_decl!();
        for m in &ast_kind.metadata.values {
            if let MetadataValueKind::ForceAlign(n) = &m.kind {
                if let Some(value) = self.translate_alignment(ast_decl.file_id, m.span, n) {
                    if max_alignment <= value {
                        max_alignment = value;
                    } else {
                        self.ctx.emit_error(
                            ErrorKind::MISC_SEMANTIC_ERROR,
                            std::iter::once(Label::primary(ast_decl.file_id, m.span).with_message(
                                "This attribute tries to force the alignment of the struct",
                            ))
                            .chain(
                                max_alignment_span.into_iter().map(|span| {
                                    Label::secondary(ast_decl.file_id, span).with_message(format!(
                                    "However the minimum alignment of this type is {max_alignment}"
                                ))
                                }),
                            ),
                            Some("Alignment of struct cannot be forced to lower"),
                        );
                    }
                }
            }
        }

        offset = round_up(offset, max_alignment);
        let decl = get_struct_decl!();
        if let Some((_, last_field)) = decl.fields.last_mut() {
            last_field.padding_after_field = offset - last_field.offset - last_field.size;
        }
        decl.alignment = max_alignment;
        decl.size = offset;

        self.descriptions[index] = TypeDescription::Struct {
            size: offset,
            alignment: max_alignment,
        };

        parents.pop().unwrap();

        Some((offset, max_alignment))
    }

    pub fn resolve_table_sizes(&mut self) {
        for decl in self.declarations.values_mut() {
            let mut max_size = 4u32;
            let mut max_alignment = 0;
            if let DeclarationKind::Table(decl) = &mut decl.kind {
                for field in decl.fields.values_mut() {
                    let (value_size, tag_size, alignment) = match &field.type_.kind {
                        TypeKind::Table(_) | TypeKind::Vector(_) | TypeKind::String => (4, 0, 4),
                        TypeKind::Union(_) => (4, 1, 4),
                        TypeKind::Array(_, _) => (1, 0, 1), // TODO: Fix this once arrays are supported
                        TypeKind::SimpleType(SimpleType::Struct(index))
                        | TypeKind::SimpleType(SimpleType::Enum(index)) => {
                            match &self.descriptions[index.0] {
                                TypeDescription::Struct { size, alignment } => {
                                    (*size, 0, *alignment)
                                }
                                TypeDescription::Enum(decl) => {
                                    (decl.type_.byte_size(), 0, decl.alignment)
                                }
                                _ => panic!("BUG"),
                            }
                        }
                        TypeKind::SimpleType(SimpleType::Bool)
                        | TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::I8))
                        | TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::U8)) => {
                            (1, 0, 1)
                        }
                        TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::I16))
                        | TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::U16)) => {
                            (2, 0, 2)
                        }
                        TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::I32))
                        | TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::U32))
                        | TypeKind::SimpleType(SimpleType::Float(FloatType::F32)) => (4, 0, 4),
                        TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::I64))
                        | TypeKind::SimpleType(SimpleType::Integer(ast::IntegerType::U64))
                        | TypeKind::SimpleType(SimpleType::Float(FloatType::F64)) => (8, 0, 8),
                    };
                    max_size = max_size.saturating_add(value_size + tag_size);
                    max_alignment = max_alignment.max(alignment);
                    field.object_value_size = value_size;
                    field.object_tag_size = tag_size;
                    field.object_alignment = alignment;
                    field.object_alignment_mask = alignment - 1;
                }
                decl.max_size = max_size;
                decl.max_alignment = max_alignment;
                let mut indices = (0..decl.fields.len()).collect::<Vec<_>>();
                indices.sort_by(|&i, &j| {
                    std::cmp::Ord::cmp(
                        &decl.fields[i].object_alignment,
                        &decl.fields[j].object_alignment,
                    )
                    .reverse()
                });
                decl.alignment_order = indices;
            }
        }
    }

    pub fn finish(mut self) -> Declarations {
        for (id, (path, decl)) in self.ast_declarations.iter().enumerate() {
            if let Some(namespace) = self.namespaces.get(path) {
                self.ctx.emit_error(
                    ErrorKind::TYPE_DEFINED_TWICE,
                    [Label::secondary(decl.file_id, decl.definition_span)
                        .with_message("Declaration was here")]
                    .into_iter()
                    .chain(namespace.spans.iter().filter_map(|(file_id, span)| {
                        let span = (*span)?;
                        Some(
                            Label::secondary(*file_id, span)
                                .with_message("namespace was defined here"),
                        )
                    })),
                    Some("Overlapping declarations"),
                );
            }
            let current_namespace = path.clone_pop();
            self.declarations.insert(
                path.clone(),
                self.translate_decl(
                    id,
                    &current_namespace,
                    NamespaceIndex(self.namespaces.get_index_of(&current_namespace).unwrap()),
                    decl,
                ),
            );
        }
        let mut parents = IndexMap::new();
        for i in 0..self.ast_declarations.len() {
            self.resolve_struct_sizes(&mut parents, i);
            assert!(parents.is_empty());
        }
        self.resolve_table_sizes();

        Declarations::new(self.namespaces, self.declarations)
    }

    pub fn default_value_for_type(&self, type_: &Type) -> Option<Literal> {
        match &type_.kind {
            TypeKind::Table(_)
            | TypeKind::Union(_)
            | TypeKind::Vector(_)
            | TypeKind::Array(_, _)
            | TypeKind::String => None,
            TypeKind::SimpleType(type_) => self.default_value_for_simple_type(type_),
        }
    }

    pub fn default_value_for_simple_type(&self, type_: &SimpleType) -> Option<Literal> {
        match type_ {
            SimpleType::Struct(_) => None,
            SimpleType::Enum(decl) => self.default_value_for_enum(*decl),
            SimpleType::Bool => Some(Literal::Bool(false)),
            SimpleType::Integer(type_) => Some(Literal::Int(match type_ {
                ast::IntegerType::U8 => IntegerLiteral::U8(0),
                ast::IntegerType::U16 => IntegerLiteral::U16(0),
                ast::IntegerType::U32 => IntegerLiteral::U32(0),
                ast::IntegerType::U64 => IntegerLiteral::U64(0),
                ast::IntegerType::I8 => IntegerLiteral::I8(0),
                ast::IntegerType::I16 => IntegerLiteral::I16(0),
                ast::IntegerType::I32 => IntegerLiteral::I32(0),
                ast::IntegerType::I64 => IntegerLiteral::I64(0),
            })),
            SimpleType::Float(type_) => Some(Literal::Float(match type_ {
                ast::FloatType::F32 => FloatLiteral::F32(0.0),
                ast::FloatType::F64 => FloatLiteral::F64(0.0),
            })),
        }
    }

    pub fn default_value_for_enum(&self, declaration_index: DeclarationIndex) -> Option<Literal> {
        match &self.descriptions[declaration_index.0] {
            TypeDescription::Enum(decl) => decl
                .variants
                .iter()
                .enumerate()
                .filter_map(|(variant_index, (k, _v))| {
                    (k.is_zero()).then_some(Literal::EnumTag {
                        variant_index,
                        value: *k,
                    })
                })
                .next(),
            _ => unreachable!(),
        }
    }
}
