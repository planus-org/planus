pub mod checks;
pub mod translation;
pub mod types;

pub fn translate_files<P: AsRef<std::path::Path>>(
    ctx: &mut crate::ctx::Ctx,
    input_files: &[P],
) -> types::Declarations {
    let mut ast_map = crate::ast_map::AstMap::default();
    for file in input_files {
        let file_id = ctx.add_file(&file, []).unwrap();
        ast_map.add_files_recursively(ctx, file_id);
    }

    let mut translator = translation::Translator::new(ctx, ast_map.reachability());
    for schema in ast_map.iter() {
        translator.add_schema(schema);
    }

    translator.finish()
}
