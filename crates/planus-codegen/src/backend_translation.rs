use planus_types::{
    ast::Docstrings,
    intermediate::{
        AbsolutePath, DeclarationIndex, DeclarationKind, Declarations, NamespaceIndex, SimpleType,
        Type, TypeKind,
    },
};
use vec_map::VecMap;

use super::backend::{
    Backend, DeclarationNames, DeclarationTranslationContext, NamespaceNames, ResolvedType,
};
use crate::backend::{DeclInfo, Keywords, Names, RelativeNamespace};

#[derive(Debug, Clone)]
pub struct BackendNamespace<B: ?Sized + Backend> {
    pub info: B::NamespaceInfo,
    pub children: Vec<BackendNamespace<B>>,
    pub declarations: Vec<BackendDeclaration<B>>,
    pub docstrings: Docstrings,
}

#[derive(Debug, Clone)]
pub enum BackendDeclaration<B: ?Sized + Backend> {
    Table(BackendTable<B>),
    Struct(BackendStruct<B>),
    Enum(BackendEnum<B>),
    Union(BackendUnion<B>),
    RpcService(BackendRpcService<B>),
}

#[derive(Debug, Clone)]
pub struct BackendTable<B: ?Sized + Backend> {
    pub docstrings: Docstrings,
    pub max_vtable_size: u32,
    pub max_size: u32,
    pub max_alignment: u32,
    pub info: B::TableInfo,
    pub fields: BackendTableFields<B::TableFieldInfo>,
}

#[derive(Debug, Clone)]
pub struct BackendStruct<B: ?Sized + Backend> {
    pub docstrings: Docstrings,
    pub size: u32,
    pub alignment: u32,
    pub info: B::StructInfo,
    pub fields: Vec<BackendStructField<B::StructFieldInfo>>,
}

#[derive(Debug, Clone)]
pub struct BackendEnum<B: ?Sized + Backend> {
    pub docstrings: Docstrings,
    pub size: u32,
    pub info: B::EnumInfo,
    pub variants: Vec<BackendVariant<B::EnumVariantInfo>>,
}

#[derive(Debug, Clone)]
pub struct BackendUnion<B: ?Sized + Backend> {
    pub docstrings: Docstrings,
    pub info: B::UnionInfo,
    pub variants: Vec<BackendVariant<B::UnionVariantInfo>>,
}

#[derive(Debug, Clone)]
pub struct BackendRpcService<B: ?Sized + Backend> {
    pub docstrings: Docstrings,
    pub info: B::RpcServiceInfo,
    pub methods: Vec<B::RpcMethodInfo>,
}

#[derive(Debug, Clone)]
pub struct NameAndDocstrings {
    pub original_name: String,
    pub docstrings: Docstrings,
}

#[derive(Debug, Clone)]
pub struct BackendVariant<V> {
    pub name_and_docs: NameAndDocstrings,
    pub variant: V,
}

impl<V> std::ops::Deref for BackendVariant<V> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.variant
    }
}
#[derive(Debug, Clone)]
pub struct BackendTableFields<F> {
    fields: VecMap<(NameAndDocstrings, F)>,
    declaration_order: Vec<(usize, u32, BackendTableFieldType)>,
    alignment_order: Vec<(usize, u32, BackendTableFieldType)>,
}

impl<F> BackendTableFields<F> {
    #[allow(clippy::too_many_arguments)]
    fn new<'a, B: ?Sized + Backend<TableFieldInfo = F>>(
        declarations: &'a Declarations,
        backend: &mut B,
        translation_context: &mut DeclarationTranslationContext<'a, '_, B>,
        full_translated_decls: &'a VecMap<BackendDeclaration<B>>,
        decl: &'a planus_types::intermediate::Table,
        decl_path: &AbsolutePath,
        translated_decl: &B::TableInfo,
    ) -> BackendTableFields<<B as Backend>::TableFieldInfo> {
        let mut declaration_order = Vec::new();

        let fields = decl
            .fields
            .iter()
            .enumerate()
            .filter(|(_index, (_field_name, field))| !field.deprecated)
            .map(|(index, (field_name, field))| {
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

                let translated_type = translate_type(
                    translation_context,
                    declarations,
                    full_translated_decls,
                    &field.type_,
                    &decl_path.clone_pop(),
                );
                (
                    index,
                    (
                        NameAndDocstrings {
                            original_name: field_name.clone(),
                            docstrings: field.docstrings.clone(),
                        },
                        backend.generate_table_field(
                            translation_context,
                            translated_decl,
                            decl,
                            field_name,
                            field,
                            translated_type,
                        ),
                    ),
                )
            })
            .collect();

        let mut alignment_order = declaration_order.clone();
        alignment_order.sort_by_key(|(index, _, field_type)| {
            std::cmp::Reverse(if *field_type == BackendTableFieldType::UnionKey {
                1
            } else {
                decl.fields[*index].object_alignment
            })
        });

        BackendTableFields {
            fields,
            declaration_order,
            alignment_order,
        }
    }

    pub fn declaration_order_with_union_keys(
        &self,
    ) -> impl Iterator<Item = BackendTableField<'_, F>> {
        self.declaration_order
            .iter()
            .map(
                move |&(index, vtable_index, field_type)| BackendTableField {
                    field_type,
                    vtable_index,
                    name_and_docs: &self.fields[index].0,
                    info: &self.fields[index].1,
                },
            )
    }

    pub fn declaration_order(&self) -> impl Iterator<Item = BackendTableField<'_, F>> {
        self.declaration_order
            .iter()
            .filter(|(_, _, field_type)| *field_type != BackendTableFieldType::UnionKey)
            .map(
                move |&(index, vtable_index, field_type)| BackendTableField {
                    field_type,
                    vtable_index,
                    name_and_docs: &self.fields[index].0,
                    info: &self.fields[index].1,
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
                    name_and_docs: &self.fields[index].0,
                    info: &self.fields[index].1,
                },
            )
    }

    pub fn is_empty(&self) -> bool {
        self.declaration_order.is_empty()
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
    pub info: &'a F,
    pub name_and_docs: &'a NameAndDocstrings,
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
    pub padding_after_field: u32,
    pub info: F,
    pub name_and_docs: NameAndDocstrings,
}

fn translate_type_index<'a, B: ?Sized + Backend>(
    translation_context: &DeclarationTranslationContext<'a, '_, B>,
    declarations: &'a Declarations,
    full_translated_decls: &'a VecMap<BackendDeclaration<B>>,
    index: DeclarationIndex,
    current_namespace_path: &AbsolutePath,
) -> ResolvedType<'a, B> {
    let (path, decl) = &translation_context.translated_decls[index.0];
    let relative_path: RelativeNamespace<B> = RelativeNamespace::new(
        current_namespace_path,
        &path.clone_pop(),
        translation_context.translated_namespaces,
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
            if let BackendDeclaration::Enum(BackendEnum { variants, .. }) =
                full_translated_decls.get(index.0).unwrap()
            {
                ResolvedType::Enum(decl, translated_decl, relative_path, variants)
            } else {
                unreachable!()
            }
        }
        DeclInfo::Union(translated_decl, decl) => {
            ResolvedType::Union(decl, translated_decl, relative_path)
        }
        DeclInfo::RpcService(_translated_decl, _decl) => todo!(),
    }
}

fn translate_type<'a, B: ?Sized + Backend>(
    translation_context: &DeclarationTranslationContext<'a, '_, B>,
    declarations: &'a Declarations,
    full_translated_decls: &'a VecMap<BackendDeclaration<B>>,
    type_: &'a Type,
    current_namespace_path: &AbsolutePath,
) -> ResolvedType<'a, B> {
    match &type_.kind {
        TypeKind::Table(index)
        | TypeKind::Union(index)
        | TypeKind::SimpleType(SimpleType::Struct(index))
        | TypeKind::SimpleType(SimpleType::Enum(index)) => translate_type_index(
            translation_context,
            declarations,
            full_translated_decls,
            *index,
            current_namespace_path,
        ),
        TypeKind::SimpleType(type_) => translate_simple_type(
            translation_context,
            declarations,
            full_translated_decls,
            type_,
            current_namespace_path,
        ),
        TypeKind::Vector(type_) => ResolvedType::Vector(Box::new(translate_type(
            translation_context,
            declarations,
            full_translated_decls,
            type_,
            current_namespace_path,
        ))),
        TypeKind::Array(type_, size) => ResolvedType::Array(
            Box::new(translate_type(
                translation_context,
                declarations,
                full_translated_decls,
                type_,
                current_namespace_path,
            )),
            *size,
        ),
        TypeKind::String => ResolvedType::String,
    }
}

fn translate_simple_type<'a, B: ?Sized + Backend>(
    translation_context: &DeclarationTranslationContext<'a, '_, B>,
    declarations: &'a Declarations,
    full_translated_decls: &'a VecMap<BackendDeclaration<B>>,
    type_: &'a SimpleType,
    current_namespace_path: &AbsolutePath,
) -> ResolvedType<'a, B> {
    match type_ {
        SimpleType::Struct(index) | SimpleType::Enum(index) => translate_type_index(
            translation_context,
            declarations,
            full_translated_decls,
            *index,
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
        docstrings: current_namespace.docstrings.clone(),
    }
}

pub fn run_backend<B: ?Sized + Backend>(
    backend: &mut B,
    declarations: &Declarations,
) -> BackendNamespace<B> {
    let keywords: Keywords = B::KEYWORDS.iter().copied().collect();
    let mut global_names = Names::new(&keywords);
    let global_names = &mut global_names;
    let mut namespace_names = (0..declarations.namespaces.len())
        .map(|_| Names::new(&keywords))
        .collect::<Vec<_>>();
    let mut declaration_names = (0..declarations.declarations.len())
        .map(|_| Names::new(&keywords))
        .collect::<Vec<_>>();
    let translated_namespaces = declarations
        .namespaces
        .iter()
        .zip(&mut namespace_names)
        .map(|((namespace_name, namespace), namespace_names)| {
            backend.generate_namespace(
                &mut NamespaceNames {
                    global_names,
                    namespace_names,
                },
                namespace_name,
                namespace,
            )
        })
        .collect::<Vec<_>>();
    let translated_decls: Vec<(AbsolutePath, DeclInfo<B>)> = declarations
        .declarations
        .iter()
        .zip(&mut declaration_names)
        .enumerate()
        .map(|(decl_id, ((decl_name, decl), declaration_names))| {
            let namespace_names = &mut namespace_names[decl.namespace_id.0];
            let decl: DeclInfo<B> = match &decl.kind {
                DeclarationKind::Table(decl) => DeclInfo::Table(
                    backend.generate_table(
                        &mut DeclarationNames {
                            global_names,
                            namespace_names,
                            declaration_names,
                        },
                        &translated_namespaces,
                        DeclarationIndex(decl_id),
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Struct(decl) => DeclInfo::Struct(
                    backend.generate_struct(
                        &mut DeclarationNames {
                            global_names,
                            namespace_names,
                            declaration_names,
                        },
                        &translated_namespaces,
                        DeclarationIndex(decl_id),
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Enum(decl) => DeclInfo::Enum(
                    backend.generate_enum(
                        &mut DeclarationNames {
                            global_names,
                            namespace_names,
                            declaration_names,
                        },
                        &translated_namespaces,
                        DeclarationIndex(decl_id),
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::Union(decl) => DeclInfo::Union(
                    backend.generate_union(
                        &mut DeclarationNames {
                            global_names,
                            namespace_names,
                            declaration_names,
                        },
                        &translated_namespaces,
                        DeclarationIndex(decl_id),
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
                DeclarationKind::RpcService(decl) => DeclInfo::RpcService(
                    backend.generate_rpc_service(
                        &mut DeclarationNames {
                            global_names,
                            namespace_names,
                            declaration_names,
                        },
                        &translated_namespaces,
                        DeclarationIndex(decl_id),
                        decl_name,
                        decl,
                    ),
                    decl,
                ),
            };
            (decl_name.clone(), decl)
        })
        .collect::<Vec<_>>();

    let mut full_translated_decls: VecMap<BackendDeclaration<B>> =
        VecMap::with_capacity(declarations.declarations.len());

    for (i, (((_decl_path, decl), orig_decl), declaration_names)) in translated_decls
        .iter()
        .zip(&declarations.declarations)
        .zip(&mut declaration_names)
        .enumerate()
    {
        if let DeclInfo::Enum(translated_decl, decl) = decl {
            let namespace_names = &mut namespace_names[orig_decl.1.namespace_id.0];
            full_translated_decls.insert(
                i,
                BackendDeclaration::Enum(BackendEnum {
                    size: decl.type_.byte_size(),
                    info: translated_decl.clone(),
                    variants: decl
                        .variants
                        .iter()
                        .map(|(value, variant)| BackendVariant {
                            name_and_docs: NameAndDocstrings {
                                original_name: variant.name.clone(),
                                docstrings: variant.docstrings.clone(),
                            },
                            variant: backend.generate_enum_variant(
                                &mut DeclarationTranslationContext {
                                    declaration_names: DeclarationNames {
                                        global_names,
                                        namespace_names,
                                        declaration_names,
                                    },
                                    translated_namespaces: &translated_namespaces,
                                    translated_decls: &translated_decls,
                                },
                                translated_decl,
                                decl,
                                &variant.name,
                                value,
                            ),
                        })
                        .collect(),
                    docstrings: (orig_decl.1).docstrings.clone(),
                }),
            );
        }
    }

    for (i, (((decl_path, decl), orig_decl), declaration_names)) in translated_decls
        .iter()
        .zip(&declarations.declarations)
        .zip(&mut declaration_names)
        .enumerate()
    {
        let namespace_names = &mut namespace_names[orig_decl.1.namespace_id.0];
        let mut translation_context = DeclarationTranslationContext {
            declaration_names: DeclarationNames {
                global_names,
                namespace_names,
                declaration_names,
            },
            translated_namespaces: &translated_namespaces,
            translated_decls: &translated_decls,
        };
        let decl = match decl {
            DeclInfo::Enum(..) => continue,
            DeclInfo::Table(translated_decl, decl) => BackendDeclaration::Table(BackendTable {
                max_vtable_size: decl.max_vtable_size,
                max_size: decl.max_size,
                max_alignment: decl.max_alignment,
                info: translated_decl.clone(),
                fields: BackendTableFields::new(
                    declarations,
                    backend,
                    &mut translation_context,
                    &full_translated_decls,
                    decl,
                    decl_path,
                    translated_decl,
                ),
                docstrings: (orig_decl.1).docstrings.clone(),
            }),
            DeclInfo::Struct(translated_decl, decl) => BackendDeclaration::Struct(BackendStruct {
                size: decl.size,
                alignment: decl.alignment,
                info: translated_decl.clone(),
                fields: decl
                    .fields
                    .iter()
                    .map(|(field_name, field)| {
                        let translated_type = translate_simple_type(
                            &translation_context,
                            declarations,
                            &full_translated_decls,
                            &field.type_,
                            &decl_path.clone_pop(),
                        );
                        BackendStructField {
                            info: backend.generate_struct_field(
                                &mut translation_context,
                                translated_decl,
                                decl,
                                field_name,
                                field,
                                translated_type,
                            ),
                            offset: field.offset,
                            padding_after_field: field.padding_after_field,
                            size: field.size,
                            name_and_docs: NameAndDocstrings {
                                original_name: field_name.clone(),
                                docstrings: field.docstrings.clone(),
                            },
                        }
                    })
                    .collect(),
                docstrings: (orig_decl.1).docstrings.clone(),
            }),
            DeclInfo::Union(translated_decl, decl) => BackendDeclaration::Union(BackendUnion {
                info: translated_decl.clone(),
                variants: decl
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, (name, variant))| {
                        let translated_type = translate_type(
                            &translation_context,
                            declarations,
                            &full_translated_decls,
                            &variant.type_,
                            &decl_path.clone_pop(),
                        );
                        BackendVariant {
                            name_and_docs: NameAndDocstrings {
                                original_name: name.clone(),
                                docstrings: variant.docstrings.clone(),
                            },
                            variant: backend.generate_union_variant(
                                &mut translation_context,
                                translated_decl,
                                decl,
                                name,
                                index as u8,
                                variant,
                                translated_type,
                            ),
                        }
                    })
                    .collect(),
                docstrings: (orig_decl.1).docstrings.clone(),
            }),
            DeclInfo::RpcService(_translated_decl, _decl) => todo!(),
        };
        full_translated_decls.insert(i, decl);
    }

    let mut translated_namespaces: VecMap<_> =
        translated_namespaces.into_iter().enumerate().collect();

    make_recursive_structure(
        declarations,
        &mut translated_namespaces,
        &mut full_translated_decls,
        declarations.get_root_namespace().0,
    )
}
