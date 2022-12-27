lalrpop_mod!(
    #[allow(clippy::all, unused_imports)]
    grammar,
    "/parser/grammar.rs"
);
mod grammar_helper;

pub use grammar::*;
