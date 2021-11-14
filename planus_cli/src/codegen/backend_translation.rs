use vec_map::VecMap;

use super::backend::{Backend, ResolvedType};
use crate::{
    codegen::backend::{DeclInfo, RelativeNamespace, ReservedNames, Scope},
    intermediate_language::types::{
        AbsolutePath, DeclarationKind, Declarations, NamespaceIndex, SimpleType, Type, TypeKind,
    },
};

#[derive(Debug, Clone)]
pub struct BackendNamespace<B: ?Sized + Backend> {
    pub info: B::NamespaceInfo,
    pub children: Vec<BackendNamespace<B>>,
    pub declarations: Vec<BackendDeclaration<B>>,
}

#[derive(Debug, Clone)]
pub enum BackendDeclaration<B: ?Sized + Backend> {
    Table {
        max_vtable_index: u32,
        max_size: u32,
        max_alignment: u32,
        info: B::TableInfo,
        fields: BackendTableFields<B::TableFieldInfo>,
    },
    Struct {
        size: u32,
        alignment: u32,
        info: B::StructInfo,
        fields: Vec<BackendStructField<B::StructFieldInfo>>,
    },
    Enum {
        size: u32,
        info: B::EnumInfo,
        variants: Vec<B::EnumVariantInfo>,
    },
    Union {
        info: B::UnionInfo,
        variants: Vec<B::UnionVariantInfo>,
    },
    RpcService {
        info: B::RpcServiceInfo,
        methods: Vec<B::RpcMethodInfo>,
    },
}

#[derive(Debug, Clone)]
pub struct BackendTableFields<F> {
    fields: Vec<F>,
    declaration_order: Vec<(usize, u32, BackendTableFieldType)>,
    alignment_order: Vec<(usize, u32, BackendTableFieldType)>,
}

impl<F> BackendTableFields<F> {
    fn new<B: ?Sized + Backend<TableFieldInfo = F>>(
        declarations: &Declarations,
        backend: &mut B,
        reserved_names: &mut ReservedNames,
        translated_namespaces: &[B::NamespaceInfo],
        translated_decls: &[(AbsolutePath, DeclInfo<B>)],
        decl: &crate::intermediate_language::types::Table,
        decl_path: &AbsolutePath,
        translated_decl: &B::TableInfo,
    ) -> BackendTableFields<<B as Backend>::TableFieldInfo> {
        let fields = decl
            .fields
            .iter()
            .map(|(field_name, field)| {
                backend.generate_table_field(
                    reserved_names,
                    &translated_namespaces,
                    &translated_decls,
                    translated_decl,
                    decl,
                    field_name,
                    field,
                    translate_type(
                        declarations,
                        &translated_namespaces,
                        &translated_decls,
                        &field.type_,
                        &decl_path.clone_pop(),
                    ),
                )
            })
            .collect();

        let mut declaration_order = Vec::new();

        for (index, field) in decl.fields.values().enumerate() {
            match field.type_.kind {
                TypeKind::Union(_) => {
                    declaration_order.push((
                        index,
                        field.vtable_index,
                        BackendTableFieldType::UnionKey,
                    ));
                    declaration_order.push((
                        index,
                        field.vtable_index + 1,
                        BackendTableFieldType::UnionValue,
                    ));
                }
                _ => {
                    declaration_order.push((
                        index,
                        field.vtable_index,
                        BackendTableFieldType::Other,
                    ));
                }
            }
        }

        let mut alignment_order = declaration_order.clone();
        alignment_order
            .sort_by_key(|(index, _, _)| std::cmp::Reverse(decl.fields[*index].object_alignment));

        BackendTableFields {
            fields,
            declaration_order,
            alignment_order,
        }
    }

    pub fn declaration_order(&self) -> impl Iterator<Item = BackendTableField<'_, F>> {
        self.declaration_order
            .iter()
            .map(
                move |&(index, vtable_index, field_type)| BackendTableField {
                    field_type,
                    vtable_index,
                    value: &self.fields[index],
                },
            )
    }

    pub fn alignment_order(&self) -> impl Iterator<Item = BackendTableField<'_, F>> {
        self.alignment_order
            .iter()
            .map(
                move |&(index, vtable_index, field_type)| BackendTableField {
                    field_type,
                    vtable_index,
                    value: &self.fields[index],
                },
            )
    }
}

impl<F> Default for BackendTableFields<F> {
    fn default() -> Self {
        Self {
            fields: Default::default(),
            declaration_order: Default::default(),
            alignment_order: Default::default(),
        }
    }
}

pub struct BackendTableField<'a, F> {
    pub field_type: BackendTableFieldType,
    pub vtable_index: u32,
    pub value: &'a F,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendTableFieldType {
    UnionKey,
    UnionValue,
    Other,
}

#[derive(Debug, Clone)]
pub struct BackendStructField<F> {
    pub offset: u32,
    pub size: u32,
    pub value: F,
}

fn translate_type_index<'a, B: ?Sized + Backend>(
    declarations: &'a Declarations,
    translated_namespaces: &'a [B::NamespaceInfo],
    translated_decls: &'a [(AbsolutePath, DeclInfo<B>)],
    index: usize,
    current_namespace_path: &'a AbsolutePath,
) -> ResolvedType<'a, B> {
    let (path, decl) = &translated_decls[index];
    let relative_path: RelativeNamespace<B> = RelativeNamespace::new(
        current_namespace_path,
        &path.clone_pop(),
        translated_namespaces,
        declarations,
    );
    match decl {
        DeclInfo::Table(translated_decl, decl) => {
            ResolvedType::Table(decl, translated_decl, relative_path)
        }
        DeclInfo::Struct(translated_decl, decl) => {
            ResolvedType::Struct(decl, translated_decl, relative_path)
        }
        DeclInfo::Enum(translated_decl, decl) => {
            ResolvedType::Enum(decl, translated_decl, relative_path)
        }
        DeclInfo::Union(translated_decl, decl) => {
            ResolvedType::Union(decl, translated_decl, relative_path)
        }
        DeclInfo::RpcService(_translated_decl, _decl) => todo!(),
    }
}

fn translate_type<'a, B: ?Sized + Backend>(
    declarations: &'a Declarations,
    translated_namespaces: &'a [B::NamespaceInfo],
    translated_decls: &'a [(AbsolutePath, DeclInfo<B>)],
    type_: &'a Type,
    current_namespace_path: &'a AbsolutePath,
) -> ResolvedType<'a, B> {
    match &type_.kind {
        TypeKind::Table(index)
        | TypeKind::Union(index)
        | TypeKind::SimpleType(SimpleType::Struct(index))
        | TypeKind::SimpleType(SimpleType::Enum(index)) => translate_type_index(
            declarations,
            translated_namespaces,
            translated_decls,
            index.0,
            current_namespace_path,
        ),
        TypeKind::SimpleType(typ) => translate_simple_type(
            declarations,
            translated_namespaces,
            translated_decls,
            typ,
            current_namespace_path,
        ),
        TypeKind::Vector(typ) => ResolvedType::Vector(Box::new(translate_type(
            declarations,
            translated_namespaces,
            translated_decls,
            &typ,
            current_namespace_path,
        ))),
        TypeKind::Array(typ, size) => ResolvedType::Array(
            Box::new(translate_type(
                declarations,
                translated_namespaces,
                translated_decls,
                &typ,
                current_namespace_path,
            )),
            *size,
        ),
        TypeKind::String => ResolvedType::String,
    }
}

fn translate_simple_type<'a, B: ?Sized + Backend>(
    declarations: &'a Declarations,
    translated_namespaces: &'a [B::NamespaceInfo],
    translated_decls: &'a [(AbsolutePath, DeclInfo<B>)],
    type_: &'a SimpleType,
    current_namespace_path: &'a AbsolutePath,
) -> ResolvedType<'a, B> {
    match type_ {
        SimpleType::Struct(index) | SimpleType::Enum(index) => translate_type_index(
            declarations,
            translated_namespaces,
            translated_decls,
            index.0,
            current_namespace_path,
        ),
        SimpleType::Bool => ResolvedType::Bool,
        SimpleType::Integer(typ) => ResolvedType::Integer(*typ),
        SimpleType::Float(typ) => ResolvedType::Float(*typ),
    }
}

fn make_recursive_structure<B: ?Sized + Backend>(
    declarations: &Declarations,
    translated_namespaces: &mut VecMap<B::NamespaceInfo>,
    translated_decls: &mut VecMap<BackendDeclaration<B>>,
    current_namespace_index: NamespaceIndex,
) -> BackendNamespace<B> {
    let (_, current_namespace) = declarations.get_namespace(current_namespace_index);
    let current_translated_namespace = translated_namespaces
        .remove(current_namespace_index.0)
        .unwrap();
    let translated_declarations: Vec<BackendDeclaration<B>> = current_namespace
        .declaration_ids
        .values()
        .map(|id| translated_decls.remove(id.0).unwrap())
        .collect();

    let children = current_namespace
        .child_namespaces
        .values()
        .map(|id| {
            make_recursive_structure(declarations, translated_namespaces, translated_decls, *id)
        })
        .collect();

    BackendNamespace {
        info: current_translated_namespace,
        children,
        declarations: translated_declarations,
    }
}

pub fn run_backend<B: ?Sized + Backend>(
    backend: &mut B,
    declarations: &Declarations,
) -> BackendNamespace<B> {
    let mut reserved_names = ReservedNames::new(B::KEYWORDS);
    let translated_namespaces = declarations
        .namespaces
        .iter()
        .map(|(namespace_name, namespace)| {
            reserved_names.clear(Scope::Namespace);
            reserved_names.clear(Scope::Declaration);
            backend.generate_namespace(&mut reserved_names, namespace_name, namespace)
        })
        .collect::<Vec<_>>();
    reserved_names.clear(Scope::Namespace);
    let translated_decls: Vec<(AbsolutePath, DeclInfo<B>)> = declarations
        .declarations
        .iter()
        .map(|(decl_name, decl)| {
            let decl: DeclInfo<B> = match &decl.kind {
                DeclarationKind::Table(decl) => DeclInfo::Table(
                    backend.generate_table(
                        &mut reserved_names,
                        &translated_namespaces,
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Struct(decl) => DeclInfo::Struct(
                    backend.generate_struct(
                        &mut reserved_names,
                        &translated_namespaces,
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Enum(decl) => DeclInfo::Enum(
                    backend.generate_enum(
                        &mut reserved_names,
                        &translated_namespaces,
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Union(decl) => DeclInfo::Union(
                    backend.generate_union(
                        &mut reserved_names,
                        &translated_namespaces,
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::RpcService(decl) => DeclInfo::RpcService(
                    backend.generate_rpc_service(
                        &mut reserved_names,
                        &translated_namespaces,
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
            };
            reserved_names.clear(Scope::Declaration);
            (decl_name.clone(), decl)
        })
        .collect::<Vec<_>>();

    let mut translated_decls: VecMap<BackendDeclaration<B>> = translated_decls
        .iter()
        .map(|(decl_path, decl)| match decl {
            DeclInfo::Table(translated_decl, decl) => BackendDeclaration::Table {
                max_vtable_index: decl.max_vtable_index,
                max_size: decl.max_size,
                max_alignment: decl.max_alignment,
                info: translated_decl.clone(),
                fields: BackendTableFields::new(
                    declarations,
                    backend,
                    &mut reserved_names,
                    &translated_namespaces,
                    &translated_decls,
                    decl,
                    decl_path,
                    translated_decl,
                ),
            },
            DeclInfo::Struct(translated_decl, decl) => BackendDeclaration::Struct {
                size: decl.size,
                alignment: decl.alignment,
                info: translated_decl.clone(),
                fields: decl
                    .fields
                    .iter()
                    .map(|(field_name, field)| BackendStructField {
                        value: backend.generate_struct_field(
                            &mut reserved_names,
                            &translated_namespaces,
                            &translated_decls,
                            translated_decl,
                            decl,
                            field_name,
                            field,
                            translate_simple_type(
                                declarations,
                                &translated_namespaces,
                                &translated_decls,
                                &field.type_,
                                &decl_path.clone_pop(),
                            ),
                        ),
                        offset: field.offset,
                        size: field.size,
                    })
                    .collect(),
            },
            DeclInfo::Enum(translated_decl, decl) => BackendDeclaration::Enum {
                size: decl.type_.byte_size(),
                info: translated_decl.clone(),
                variants: decl
                    .variants
                    .iter()
                    .map(|(value, name)| {
                        backend.generate_enum_variant(
                            &mut reserved_names,
                            &translated_namespaces,
                            &translated_decls,
                            translated_decl,
                            decl,
                            name,
                            value,
                        )
                    })
                    .collect(),
            },
            DeclInfo::Union(translated_decl, decl) => BackendDeclaration::Union {
                info: translated_decl.clone(),
                variants: decl
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, (name, variant))| {
                        backend.generate_union_variant(
                            &mut reserved_names,
                            &translated_namespaces,
                            &translated_decls,
                            translated_decl,
                            decl,
                            name,
                            index as u8,
                            variant,
                            translate_type(
                                declarations,
                                &translated_namespaces,
                                &translated_decls,
                                &variant.type_,
                                &decl_path.clone_pop(),
                            ),
                        )
                    })
                    .collect(),
            },
            DeclInfo::RpcService(_translated_decl, _decl) => todo!(),
        })
        .enumerate()
        .collect();

    let mut translated_namespaces: VecMap<_> =
        translated_namespaces.into_iter().enumerate().collect();

    make_recursive_structure(
        declarations,
        &mut translated_namespaces,
        &mut translated_decls,
        declarations.get_root_namespace().0,
    )
}
