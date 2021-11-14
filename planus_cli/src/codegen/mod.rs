use crate::{ast_map::AstMap, ctx::Ctx, intermediate_language::translation::Translator};
use anyhow::{anyhow, Result};
use std::path::Path;

use self::rust2::RustBackend;

pub mod backend;
pub mod backend_translation;
pub mod name_generator;
pub mod rust;
pub mod rust2;

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
        let mut codegen = crate::codegen::rust::Codegen::new(output_file)?;
        codegen.generate_code(&namespace)?;
        Ok(())
    } else {
        Err(anyhow!("could not generate code"))
    }
}

pub fn generate_code2<P: AsRef<Path>>(input_files: &[P]) -> Result<()> {
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

    let declarations = translator.finish();

    let output = backend_translation::run_backend(&mut RustBackend, &declarations);

    if !ctx.has_errors() {
        println!("{:#?}", output);
    }

    Ok(())
}
