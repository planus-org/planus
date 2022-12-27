#[macro_use]
extern crate lalrpop_util;

mod grammar_helper;
lalrpop_mod!(#[allow(clippy::all, unused_imports)] pub grammar);

// Multiple representations:
//   * Concrete syntax tree (cst): As close a representation to the written
//     code as possible including the spans and order of everything. Every
//     individual token and comment are included. This is useful for doing
//     source-level manipulations such as formatting.
//   * Abstract syntax tree (ast): An "abstract" representation, i.e. includes
//     a lot less information and cares less about the source-level
//     representation. This means that things like order in the original file
//     no longer matters.

pub mod app;
pub mod ast;
pub mod ast_map;
pub mod codegen;
pub mod cst;
pub mod ctx;
pub mod error;
pub mod intermediate_language;
pub mod lexer;
pub mod util;
