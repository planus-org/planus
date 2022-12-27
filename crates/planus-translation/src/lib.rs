#[macro_use]
extern crate lalrpop_util;

mod ast_convert;
mod intermediate_language;

mod ctx;
mod error;
mod parser;
mod pretty_print;
mod util;

use std::path::Path;

pub use error::ErrorKind;
pub use intermediate_language::translate_files;

pub fn format_file(path: &impl AsRef<Path>, ignore_errors: bool) -> Option<String> {
    let mut ctx = ctx::Ctx::default();
    let file_id = ctx.add_file(path, []).unwrap();
    if let Some(parsed) = ctx.parse_file(file_id) {
        if ctx.has_errors() && !ignore_errors {
            None
        } else {
            let mut s = String::new();
            if pretty_print::pretty_print(ctx.get_source(file_id), &parsed, &mut s).is_ok() {
                Some(s)
            } else {
                None
            }
        }
    } else {
        None
    }
}
