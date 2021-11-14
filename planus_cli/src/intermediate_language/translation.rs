use super::types::*;
use crate::{
    ast::{self, FloatType, NamespacePath},
    ctx::Ctx,
    error::ErrorKind,
    util::sorted_map::{SortedMap, SortedSet},
};
use codespan::{FileId, Span};
use codespan_reporting::diagnostic::Label;
use indexmap::{map::Entry, IndexMap};

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
        let mut namespace_path = if let Some((_span, path)) = &schema.namespace {
            AbsolutePath::from_ctx(self.ctx, &path.parts)
        } else {
            AbsolutePath::ROOT_PATH
        };

        let namespace_entry = self.namespaces.entry(namespace_path.clone());
        let mut namespace_index = namespace_entry.index();
        let namespace = namespace_entry.or_default();

        for decl in schema.type_declarations.values() {
            let name = self.ctx.resolve_identifier(decl.identifier.value);
            match self
                .ast_declarations
                .entry(namespace_path.clone_push(&name))
            {
                Entry::Occupied(entry) => {
                    self.ctx.emit_error(
                        ErrorKind::TYPE_DEFINED_TWICE,
                        std::array::IntoIter::new([
                            Label::secondary(entry.get().file_id, entry.get().definition_span)
                                .with_message("first definition was here"),
                            Label::secondary(schema.file_id, decl.definition_span)
                                .with_message("second definition was here"),
                        ]),
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
                    TypeDescription::Enum(self.translate_enum(decl))
                }
                ast::TypeDeclarationKind::Union(_) => TypeDescription::Union,
                ast::TypeDeclarationKind::RpcService(_) => TypeDescription::RpcService,
            })
        }

        while let Some(last) = namespace_path.pop() {
            match self.namespaces.entry(namespace_path) {
                Entry::Occupied(_) => break,
                Entry::Vacant(entry) => {
                    namespace_path = entry.key().clone();
                    let next_namespace_index = entry.index();
                    entry
                        .insert(Default::default())
                        .child_namespaces
                        .insert(last, NamespaceIndex(namespace_index));
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
        let absolute_path = AbsolutePath::from_ctx(self.ctx, &namespace_path.parts);
        let mut relative_path = current_namespace.clone();
        relative_path.0.extend(absolute_path.0.iter().cloned());
        let mut hints: Vec<Label<FileId>> = Vec::new();
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
                            std::array::IntoIter::new([
                                Label::primary(current_file_id, namespace_path.span)
                                    .with_message("Rpc services cannot be used as types"),
                                Label::secondary(decl.file_id, decl.definition_span)
                                    .with_message("Service was defined here"),
                            ]),
                            Some(&format!("Cannot use the RpcService {} in a type", path)),
                        );
                        return None;
                    }
                };
                if self
                    .reachability
                    .get(&current_file_id)
                    .map_or(false, |reachable| reachable.contains(&decl.file_id))
                {
                    return Some(result);
                } else {
                    hints.push(
                        Label::secondary(decl.file_id, decl.definition_span).with_message(
                            &format!(
                                "Perhaps you meant to include {}",
                                self.ctx.get_filename(decl.file_id).display()
                            ),
                        ),
                    );
                }
            }
        }
        self.ctx.emit_error(
            ErrorKind::UNKNOWN_IDENTIFIER,
            std::iter::once(
                Label::primary(current_file_id, namespace_path.span).with_message("Unknown type"),
            )
            .chain(hints),
            None,
        );
        None
    }

    fn translate_integer(
        &self,
        span: Span,
        is_negative: bool,
        v: &str,
        type_: &ast::IntegerType,
    ) -> Option<IntegerLiteral> {
        use IntegerLiteral::*;

        Some(match type_ {
            ast::IntegerType::U8 => U8(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::U16 => U16(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::U32 => U32(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::U64 => U64(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::I8 => I8(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::I16 => I16(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::I32 => I32(self.translate_integer_generic(span, is_negative, v)?),
            ast::IntegerType::I64 => I64(self.translate_integer_generic(span, is_negative, v)?),
        })
    }

    fn translate_integer_generic<
        T: num_traits::CheckedAdd
            + num_traits::CheckedSub
            + num_traits::CheckedMul
            + num_traits::CheckedNeg
            + num_traits::NumCast,
    >(
        &self,
        _span: Span,
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
                    // TODO error message
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
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        literal: &ast::Literal,
        type_: &Type,
    ) -> Option<Literal> {
        use crate::intermediate_language::types::{SimpleType::*, TypeKind::*};
        use ast::LiteralKind;

        match (&literal.kind, &type_.kind) {
            (LiteralKind::Bool(value), SimpleType(Bool)) => Some(Literal::Bool(*value)),
            (LiteralKind::Integer { is_negative, value }, SimpleType(Integer(type_))) => self
                .translate_integer(literal.span, *is_negative, value, type_)
                .map(Literal::Int),
            (LiteralKind::Float { is_negative, value }, SimpleType(Float(type_))) => self
                .translate_float(literal.span, *is_negative, value, type_)
                .map(Literal::Float),
            (LiteralKind::String(s), String) => Some(Literal::String(s.clone())),
            (LiteralKind::List(literals), Vector(type_)) => {
                let mut out = Vec::new();
                for literal in literals.iter() {
                    out.push(self.translate_literal(
                        current_namespace,
                        current_file_id,
                        literal,
                        &*type_,
                    )?);
                }
                Some(Literal::Vector(out))
            }
            (LiteralKind::List(_), Array(_type_, _size)) => todo!(),
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
                    &*inner_type,
                )?)),
            }),
            ast::TypeKind::Array { inner_type, size } => Some(Type {
                span: type_.span,
                kind: TypeKind::Array(
                    Box::new(self.translate_type(
                        current_namespace,
                        current_file_id,
                        &*inner_type,
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
        let fields = decl
            .fields
            .iter()
            .filter_map(|(ident, field)| {
                let type_ =
                    self.translate_type(current_namespace, current_file_id, &field.type_)?;
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
                match type_.kind {
                    TypeKind::SimpleType(type_) => Some((
                        self.ctx.resolve_identifier(*ident),
                        StructField {
                            type_,
                            offset: u32::MAX,
                            size: u32::MAX,
                        },
                    )),
                    _ => {
                        self.ctx.emit_error(
                            ErrorKind::TYPE_ERROR,
                            std::iter::once(
                                Label::primary(current_file_id, field.type_.span)
                                    .with_message("Only simple types are permitted in struct"),
                            ),
                            None,
                        );
                        None
                    }
                }
            })
            .collect();
        Struct {
            fields,
            size: u32::MAX,
            alignment: u32::MAX,
        }
    }

    fn translate_decl(
        &self,
        id: usize,
        current_namespace: &AbsolutePath,
        decl: &ast::Declaration,
    ) -> Declaration {
        let current_file_id = decl.file_id;
        let kind =
            match &decl.kind {
                ast::TypeDeclarationKind::Table(decl) => DeclarationKind::Table(
                    self.translate_table(current_namespace, current_file_id, decl),
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
                ast::TypeDeclarationKind::RpcService(_decl) => todo!(),
            };

        Declaration {
            definition_span: decl.definition_span,
            file_id: current_file_id,
            kind,
        }
    }

    fn translate_table(
        &self,
        current_namespace: &AbsolutePath,
        current_file_id: FileId,
        decl: &ast::Struct,
    ) -> Table {
        let mut next_vtable_index = 0u32;
        let mut max_vtable_index = 0u32;

        let fields = decl
            .fields
            .iter()
            .filter_map(|(ident, field)| {
                let type_ =
                    self.translate_type(current_namespace, current_file_id, &field.type_)?;
                let assignment = field.assignment.as_ref().and_then(|assignment| {
                    self.translate_literal(current_namespace, current_file_id, assignment, &type_)
                });
                let mut required = false;
                let mut deprecated = false;
                let vtable_index = next_vtable_index;

                for m in &field.metadata {
                    match self.ctx.resolve_identifier(m.key.value).as_str() {
                        "required" => required = true,
                        "deprecated" => deprecated = true,
                        // TODO: allow setting the vtable index here
                        // TODO: also remember to validate it
                        _ => (),
                    }
                }

                max_vtable_index = max_vtable_index.max(vtable_index);

                if matches!(&type_.kind, TypeKind::Union(_)) {
                    next_vtable_index = vtable_index + 2;
                } else {
                    next_vtable_index = vtable_index + 1;
                }

                Some((
                    self.ctx.resolve_identifier(*ident),
                    TableField {
                        type_,
                        assignment,
                        vtable_index,
                        object_value_size: u32::MAX,
                        object_tag_size: u32::MAX,
                        object_alignment_mask: u32::MAX,
                        object_alignment: u32::MAX,
                        required,
                        deprecated,
                    },
                ))
            })
            .collect();
        Table {
            fields,
            alignment_order: Vec::new(),
            max_size: u32::MAX,
            max_vtable_index,
        }
    }

    fn translate_enum(&self, decl: &ast::Enum) -> Enum {
        let mut alignment = decl.type_.byte_size();
        for m in &decl.metadata {
            match self.ctx.resolve_identifier(m.key.value).as_str() {
                "force_align" => {
                    if let Some(meta_value) = &m.value {
                        match &meta_value.kind {
                            ast::LiteralKind::Integer { is_negative, value } => {
                                if let Some(value) = self.translate_integer_generic::<u32>(
                                    meta_value.span,
                                    *is_negative,
                                    value,
                                ) {
                                    if value.is_power_of_two() {
                                        alignment = value;
                                    } else {
                                        // TODO: write an error
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }

        let mut variants = IndexMap::new();
        let mut next_value = decl.type_.default_value();
        for (ident, variant) in decl.variants.iter() {
            let mut value = next_value;
            let name = self.ctx.resolve_identifier(*ident);
            if let Some(assignment) = &variant.value {
                if let Some(v) = self.translate_integer(
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
                Entry::Occupied(_) => panic!(),
                Entry::Vacant(entry) => {
                    entry.insert(name);
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
        let variants = decl
            .variants
            .values()
            .filter_map(|variant| {
                let type_ =
                    self.translate_type(current_namespace, current_file_id, &variant.type_)?;

                // TODO: check that this type is valid in a union context

                let name = if let Some(ident) = variant.ident {
                    self.ctx.resolve_identifier(ident.value)
                } else {
                    // TODO: make sure this is unique
                    match &variant.type_.kind {
                        ast::TypeKind::Path(p) => {
                            self.ctx.resolve_identifier(*p.parts.last().unwrap())
                        }
                        _ => todo!(),
                    }
                };

                Some((name, UnionVariant { type_ }))
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
                            .with_message(&(if a == b { format!(
                                "{} contains itself directly",
                                self.ast_declarations.get_index(a).unwrap().0,
                            )} else {format!(
                                "{} contains itself through {}",
                                self.ast_declarations.get_index(a).unwrap().0,
                                self.ast_declarations.get_index(b).unwrap().0
                            )}))
                        }).take(1),
                    Some("Structs are not allowed to contain themselves, as it implies infinite size"),
                );
                return None;
            }
            Entry::Vacant(entry) => {
                entry.insert(());
            }
        }

        macro_rules! get_field {
            () => {
                if let DeclarationKind::Struct(decl) = &mut self.declarations[index].kind {
                    decl
                } else {
                    panic!("BUG")
                }
            };
        }

        fn round_up(value: u32, alignment: u32) -> u32 {
            ((value + alignment - 1) / alignment) * alignment
        }

        let mut offset = 0;
        let mut max_alignment = 1;
        for field_id in 0..get_field!().fields.len() {
            let (cur_size, cur_alignment) = match &get_field!().fields[field_id].type_ {
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

            offset = round_up(offset, cur_alignment);
            get_field!().fields[field_id].offset = offset;
            get_field!().fields[field_id].size = cur_size;
            offset += cur_size;
            max_alignment = max_alignment.max(cur_alignment);
        }
        offset = round_up(offset, max_alignment);
        get_field!().alignment = max_alignment;
        get_field!().size = offset;

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
            if let DeclarationKind::Table(decl) = &mut decl.kind {
                for field in decl.fields.values_mut() {
                    let (value_size, tag_size, alignment) = match &field.type_.kind {
                        TypeKind::Table(_) | TypeKind::Vector(_) | TypeKind::String => (4, 0, 4),
                        TypeKind::Union(_) => (4, 1, 4),
                        TypeKind::Array(_, _) => todo!(),
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
                    field.object_value_size = value_size;
                    field.object_tag_size = tag_size;
                    field.object_alignment = alignment;
                    field.object_alignment_mask = alignment - 1;
                }
                decl.max_size = max_size;
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
            self.declarations.insert(
                path.clone(),
                self.translate_decl(id, &path.clone_pop(), decl),
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
}

impl AbsolutePath {
    pub fn from_ctx(ctx: &Ctx, parts: &[ast::RawIdentifier]) -> Self {
        let path = parts.iter().map(|&s| ctx.resolve_identifier(s)).collect();
        Self(path)
    }

    pub fn get_relative_to(&self, _other: &AbsolutePath) -> RelativePath {
        todo!()
    }
}

impl<'a> From<&'a ast::BuiltinType> for TypeKind {
    fn from(value: &ast::BuiltinType) -> TypeKind {
        match value {
            ast::BuiltinType::Bool => TypeKind::SimpleType(SimpleType::Bool),
            ast::BuiltinType::Integer(typ) => TypeKind::SimpleType(SimpleType::Integer(*typ)),
            ast::BuiltinType::Float(typ) => TypeKind::SimpleType(SimpleType::Float(*typ)),
            ast::BuiltinType::String => TypeKind::String,
        }
    }
}

impl ast::IntegerType {
    pub fn default_value(&self) -> IntegerLiteral {
        match self {
            ast::IntegerType::U8 => IntegerLiteral::U8(0),
            ast::IntegerType::U16 => IntegerLiteral::U16(0),
            ast::IntegerType::U32 => IntegerLiteral::U32(0),
            ast::IntegerType::U64 => IntegerLiteral::U64(0),
            ast::IntegerType::I8 => IntegerLiteral::I8(0),
            ast::IntegerType::I16 => IntegerLiteral::I16(0),
            ast::IntegerType::I32 => IntegerLiteral::I32(0),
            ast::IntegerType::I64 => IntegerLiteral::I64(0),
        }
    }
}

impl IntegerLiteral {
    pub fn next(&self) -> IntegerLiteral {
        match self {
            IntegerLiteral::U8(n) => IntegerLiteral::U8(n.wrapping_add(1)),
            IntegerLiteral::I8(n) => IntegerLiteral::I8(n.wrapping_add(1)),
            IntegerLiteral::U16(n) => IntegerLiteral::U16(n.wrapping_add(1)),
            IntegerLiteral::I16(n) => IntegerLiteral::I16(n.wrapping_add(1)),
            IntegerLiteral::U32(n) => IntegerLiteral::U32(n.wrapping_add(1)),
            IntegerLiteral::I32(n) => IntegerLiteral::I32(n.wrapping_add(1)),
            IntegerLiteral::U64(n) => IntegerLiteral::U64(n.wrapping_add(1)),
            IntegerLiteral::I64(n) => IntegerLiteral::I64(n.wrapping_add(1)),
        }
    }
}
