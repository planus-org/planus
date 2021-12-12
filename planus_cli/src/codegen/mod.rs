use self::rust::RustBackend;
use crate::{ast_map::AstMap, ctx::Ctx, intermediate_language::translation::Translator};
use anyhow::{anyhow, Result};
use askama::Template;
use std::{io::Write, path::Path};

pub mod backend;
pub mod backend_translation;
pub mod rust;
pub mod templates;

pub fn generate_code<P: AsRef<Path>>(input_files: &[P], output_filename: String) -> Result<()> {
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

    if !ctx.has_errors() {
        let output = backend_translation::run_backend(&mut RustBackend, &declarations);

        let res = templates::rust::Namespace(&output).render().unwrap();
        let mut file = std::fs::File::create(&output_filename)?;
        file.write_all(res.as_bytes())?;
        file.flush()?;

        rust::format_file(&output_filename)?;
        Ok(())
    } else {
        Err(anyhow!("could not generate code"))
    }
}
