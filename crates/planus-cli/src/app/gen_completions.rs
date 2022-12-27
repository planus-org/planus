use clap::{IntoApp, Parser};
use clap_complete::{generate, Shell};

/// Generate shell completion scripts
#[derive(Parser)]
#[clap(arg_required_else_help = true)]
pub struct Command {
    /// Which shell to generate completions for
    #[clap(arg_enum)]
    shell: Shell,
}

impl Command {
    pub fn run(self, _options: super::AppOptions) {
        generate(
            self.shell,
            &mut super::App::command(),
            "planus",
            &mut std::io::stdout(),
        );
    }
}
