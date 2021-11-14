use std::path::Path;

use anyhow::Result;

use crate::{
    ast_map::AstMap, codegen::rust::Codegen, ctx::Ctx,
    intermediate_language::translation::Translator,
};

pub mod name_generator;
pub mod rust;

pub fn generate_code<P: AsRef<Path>>(input_files: &[P], output_file: String) -> Result<()> {
    let mut ctx = Ctx::default();
    let mut ast_map = AstMap::default();
    for file in input_files {
        let file_id = ctx.add_file(&file).unwrap();
        ast_map.add_files_recursively(&mut ctx, file_id);
    }

    let mut translator = Translator::new(&ctx, ast_map.reachability());
    for schema in ast_map.iter() {
        translator.add_schema(schema);
    }

    let namespace = translator.finish();

    if !ctx.has_errors() {
        let mut codegen = Codegen::new(output_file)?;
        codegen.generate_code(&namespace)?;
    }

    Ok(())
}
