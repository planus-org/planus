use crate::intermediate_language::types::*;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
};

#[derive(Copy, Clone, Debug)]
pub enum Scope {
    Global = 0,
    Namespace = 1,
    Declaration = 2,
}

pub struct ReservedNames {
    keywords: HashSet<&'static str>,
    names: [BTreeMap<&'static str, HashSet<String>>; 3],
}

impl ReservedNames {
    pub fn try_reserve(&mut self, scope: Scope, binding_kind: &'static str, value: &str) -> bool {
        if self.keywords.contains(value) {
            false
        } else {
            let names = self.names[scope as usize].entry(binding_kind).or_default();
            if names.contains(value) {
                false
            } else {
                names.insert(value.to_string());
                true
            }
        }
    }

    pub fn try_reserve_repeat<'a>(
        &mut self,
        scope: Scope,
        binding_kind: &'static str,
        value: Cow<'a, str>,
        padding: char,
    ) -> Cow<'a, str> {
        if self.try_reserve(scope, binding_kind, &value) {
            return value;
        }

        let mut value = format!("{}{}", value, padding);
        while !self.try_reserve(scope, binding_kind, &value) {
            value.push(padding);
        }

        value.into()
    }

    pub fn clear(&mut self, scope: Scope) {
        for bindings in self.names[scope as usize].values_mut() {
            bindings.clear();
        }
    }
}

pub trait NameGenerator {
    type NamespaceInfo;
    type DeclInfo;
    type EntryInfo;
    const KEYWORDS: &'static [&'static str];

    fn generate_namespace(
        &mut self,
        reserved_names: &mut ReservedNames,
        namespace_name: &AbsolutePath,
        namespace: &Namespace,
    ) -> Self::NamespaceInfo;

    fn generate_table(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &Table,
    ) -> Self::DeclInfo;

    fn generate_struct(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &Struct,
    ) -> Self::DeclInfo;

    fn generate_enum(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &Enum,
    ) -> Self::DeclInfo;

    fn generate_union(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &Union,
    ) -> Self::DeclInfo;

    fn generate_rpc_service(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_name: &AbsolutePath,
        decl: &RpcService,
    ) -> Self::DeclInfo;

    fn generate_table_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        decl_name: &AbsolutePath,
        decl: &Table,
        field_name: &str,
        field: &TableField,
    ) -> Self::EntryInfo;

    fn generate_struct_field(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        decl_name: &AbsolutePath,
        decl: &Struct,
        field_name: &str,
        field: &StructField,
    ) -> Self::EntryInfo;

    fn generate_enum_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        decl_name: &AbsolutePath,
        decl: &Enum,
        key: &str,
        value: &IntegerLiteral,
    ) -> Self::EntryInfo;

    fn generate_union_variant(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        decl_name: &AbsolutePath,
        decl: &Union,
        key: &str,
        value: &UnionVariant,
    ) -> Self::EntryInfo;

    fn generate_rpc_method(
        &mut self,
        reserved_names: &mut ReservedNames,
        decl_infos: &[Self::DeclInfo],
        decl_name: &AbsolutePath,
        decl: &RpcService,
        method_name: &str,
        method: &RpcMethod,
    ) -> Self::EntryInfo;
}

#[allow(clippy::too_many_arguments)]
fn run_namespace<G: NameGenerator>(
    g: &mut G,
    declarations: &Declarations,
    reserved_names: &mut ReservedNames,
    namespace_id: NamespaceIndex,
    name: &AbsolutePath,
    namespace: &Namespace,
    namespace_info: &mut Vec<Option<G::NamespaceInfo>>,
    decl_info: &mut Vec<Option<G::DeclInfo>>,
) {
    reserved_names.clear(Scope::Namespace);
    namespace_info[namespace_id.0] = Some(g.generate_namespace(reserved_names, name, namespace));

    let mut ids = (0..namespace.declaration_ids.len()).collect::<Vec<_>>();
    ids.sort_by_key(|i| {
        let name = namespace.declaration_ids.get_index(*i).unwrap().0;
        (name.len(), name)
    });

    for i in ids {
        let declaration_id = namespace.declaration_ids[i];
        let (name, decl) = declarations.get_declaration(declaration_id);
        run_declaration(g, reserved_names, declaration_id, name, decl, decl_info);
    }

    let mut ids = (0..namespace.child_namespaces.len()).collect::<Vec<_>>();
    ids.sort_by_key(|i| {
        let name = namespace.child_namespaces.get_index(*i).unwrap().0;
        (name.len(), name)
    });

    for i in ids {
        let namespace_id = namespace.child_namespaces[i];
        let (name, namespace) = declarations.get_namespace(namespace_id);
        run_namespace(
            g,
            declarations,
            reserved_names,
            namespace_id,
            name,
            namespace,
            namespace_info,
            decl_info,
        );
    }
}

fn run_declaration<G: NameGenerator>(
    g: &mut G,
    reserved_names: &mut ReservedNames,
    declaration_id: DeclarationIndex,
    name: &AbsolutePath,
    decl: &Declaration,
    decl_infos: &mut Vec<Option<G::DeclInfo>>,
) {
    match &decl.kind {
        DeclarationKind::Table(decl) => {
            let decl_info = g.generate_table(reserved_names, name, decl);
            decl_infos[declaration_id.0] = Some(decl_info);
        }
        DeclarationKind::Struct(decl) => {
            let decl_info = g.generate_struct(reserved_names, name, decl);
            decl_infos[declaration_id.0] = Some(decl_info);
        }
        DeclarationKind::Enum(decl) => {
            let decl_info = g.generate_enum(reserved_names, name, decl);
            decl_infos[declaration_id.0] = Some(decl_info);
        }
        DeclarationKind::Union(decl) => {
            let decl_info = g.generate_union(reserved_names, name, decl);
            decl_infos[declaration_id.0] = Some(decl_info);
        }
        DeclarationKind::RpcService(decl) => {
            let decl_info = g.generate_rpc_service(reserved_names, name, decl);
            decl_infos[declaration_id.0] = Some(decl_info);
        }
    }
}
#[allow(clippy::type_complexity)]
pub fn run_name_generator<G: NameGenerator>(
    g: &mut G,
    declarations: &Declarations,
) -> (
    Vec<G::NamespaceInfo>,
    Vec<G::DeclInfo>,
    Vec<Vec<G::EntryInfo>>,
) {
    let mut namespace_info = (0..declarations.namespace_count())
        .map(|_| None)
        .collect::<Vec<_>>();
    let mut decl_info = (0..declarations.declaration_count())
        .map(|_| None)
        .collect::<Vec<_>>();
    let mut reserved_names = ReservedNames {
        keywords: G::KEYWORDS.iter().cloned().collect(),
        names: Default::default(),
    };
    let (namespace_id, namespace) = declarations.get_root_namespace();
    run_namespace(
        g,
        declarations,
        &mut reserved_names,
        namespace_id,
        &AbsolutePath::ROOT_PATH,
        namespace,
        &mut namespace_info,
        &mut decl_info,
    );

    let namespace_info = namespace_info
        .into_iter()
        .map(|info| info.unwrap())
        .collect();
    let decl_infos = decl_info
        .into_iter()
        .map(|info| info.unwrap())
        .collect::<Vec<_>>();
    let mut entry_infos = Vec::with_capacity(declarations.declaration_count());

    for (_decl_id, decl_name, decl) in declarations.iter_declarations() {
        reserved_names.clear(Scope::Declaration);
        // TODO sort fields
        entry_infos.push(match &decl.kind {
            DeclarationKind::Table(decl) => decl
                .fields
                .iter()
                .map(|(field_name, field)| {
                    g.generate_table_field(
                        &mut reserved_names,
                        &decl_infos,
                        decl_name,
                        decl,
                        field_name,
                        field,
                    )
                })
                .collect::<Vec<_>>(),
            DeclarationKind::Struct(decl) => decl
                .fields
                .iter()
                .map(|(field_name, field)| {
                    g.generate_struct_field(
                        &mut reserved_names,
                        &decl_infos,
                        decl_name,
                        decl,
                        field_name,
                        field,
                    )
                })
                .collect::<Vec<_>>(),

            DeclarationKind::Enum(decl) => decl
                .variants
                .iter()
                .map(|(value, key)| {
                    g.generate_enum_variant(
                        &mut reserved_names,
                        &decl_infos,
                        decl_name,
                        decl,
                        key,
                        value,
                    )
                })
                .collect::<Vec<_>>(),

            DeclarationKind::Union(decl) => decl
                .variants
                .iter()
                .map(|(variant_name, variant)| {
                    g.generate_union_variant(
                        &mut reserved_names,
                        &decl_infos,
                        decl_name,
                        decl,
                        variant_name,
                        variant,
                    )
                })
                .collect::<Vec<_>>(),

            DeclarationKind::RpcService(decl) => decl
                .methods
                .iter()
                .map(|(method_name, method)| {
                    g.generate_rpc_method(
                        &mut reserved_names,
                        &decl_infos,
                        decl_name,
                        decl,
                        method_name,
                        method,
                    )
                })
                .collect::<Vec<_>>(),
        });
    }

    (namespace_info, decl_infos, entry_infos)
}
