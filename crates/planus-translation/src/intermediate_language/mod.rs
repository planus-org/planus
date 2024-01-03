use std::path::Path;

use planus_types::intermediate::Declarations;

use crate::{ast_convert::ConverterOptions, ctx::Ctx};

pub mod checks;
pub mod translation;

pub mod ast_map;

pub fn translate_files(input_files: &[impl AsRef<Path>]) -> Option<Declarations> {
    translate_files_with_options(input_files, Default::default())
}

pub fn translate_files_with_options(
    input_files: &[impl AsRef<Path>],
    converter_options: ConverterOptions,
) -> Option<Declarations> {
    let mut ctx = Ctx::default();
    let declarations = translate_files_with_context(&mut ctx, input_files, converter_options);
    if !ctx.has_errors() {
        Some(declarations)
    } else {
        None
    }
}

fn translate_files_with_context<P: AsRef<Path>>(
    ctx: &mut crate::ctx::Ctx,
    input_files: &[P],
    converter_options: ConverterOptions,
) -> Declarations {
    let mut ast_map = ast_map::AstMap::new_with_options(converter_options);
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
