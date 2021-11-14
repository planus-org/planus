use anyhow::Result;
use planus_cli::{ast::print::PrettyPrint, ast_map::AstMap, ctx::Ctx};

#[derive(structopt::StructOpt)]
struct Args {
    files: Vec<String>,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    {
        let mut ctx = Ctx::default();
        let mut ast_map = AstMap::default();
        for file in args.files {
            let file_id = ctx.add_file(&file).unwrap();
            ast_map.add_files_recursively(&mut ctx, file_id);
        }
        for schema in ast_map.iter() {
            schema.print(&ctx);
            println!();
        }
    }

    Ok(())
}
