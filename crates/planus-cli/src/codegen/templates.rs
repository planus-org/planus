macro_rules! template {
    ($path:literal, $name:ident, $inner:ty) => {
        #[derive(askama::Template)]
        #[template(path = $path, escape = "none")]
        pub struct $name<'a>(pub &'a $inner);

        impl<'a> std::ops::Deref for $name<'a> {
            type Target = $inner;
            fn deref(&self) -> &$inner {
                &self.0
            }
        }
    };
}

macro_rules! template_module {
    ($name:ident, $backend:ty, [$namespace:literal, $struct:literal, $table:literal, $enum:literal, $union:literal, $rpc_service:literal]) => {
        pub mod $name {
            #[allow(unused_imports)]
            use crate::codegen::backend_translation::{
                BackendDeclaration, BackendEnum, BackendNamespace, BackendRpcService,
                BackendStruct, BackendTable, BackendTableFieldType, BackendUnion,
            };

            template!($namespace, Namespace, BackendNamespace<$backend>);
            template!($struct, Struct, BackendStruct<$backend>);
            template!($table, Table, BackendTable<$backend>);
            template!($union, Union, BackendUnion<$backend>);
            template!($enum, Enum, BackendEnum<$backend>);
            template!($rpc_service, RpcService, BackendRpcService<$backend>);
        }
    };
}

template_module!(
    rust,
    crate::codegen::rust::RustBackend,
    [
        "rust/namespace.template",
        "rust/struct.template",
        "rust/table.template",
        "rust/enum.template",
        "rust/union.template",
        "rust/rpc_service.template"
    ]
);

template_module!(
    dot,
    crate::codegen::dot::DotBackend,
    [
        "dot/namespace.template",
        "dot/struct.template",
        "dot/table.template",
        "dot/enum.template",
        "dot/union.template",
        "dot/rpc_service.template"
    ]
);
