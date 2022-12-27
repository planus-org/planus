use std::path::Path;

use crate::ctx::Ctx;

pub mod checks;
pub mod translation;

pub mod ast_map;

pub fn translate_files(
    input_files: &[impl AsRef<Path>],
) -> Option<planus_types::intermediate::Declarations> {
    let mut ctx = Ctx::default();
    let declarations = translate_files_with_context(&mut ctx, input_files);
    if !ctx.has_errors() {
        Some(declarations)
    } else {
        None
    }
}

fn translate_files_with_context<P: AsRef<std::path::Path>>(
    ctx: &mut crate::ctx::Ctx,
    input_files: &[P],
) -> planus_types::intermediate::Declarations {
    let mut ast_map = ast_map::AstMap::default();
    for file in input_files {
        if let Some(file_id) = ctx.add_file(file, []) {
            ast_map.add_files_recursively(ctx, file_id);
        }
    }

    let mut translator = translation::Translator::new(ctx, ast_map.reachability());
    for schema in ast_map.iter() {
        translator.add_schema(schema);
    }

    translator.finish()
}
