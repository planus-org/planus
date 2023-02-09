use std::{io, ops::DerefMut, path::PathBuf, process::ExitCode};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use crossterm::{
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::FuzzyMatcher;
use planus_inspector::{run_inspector, Inspector};
use planus_translation::translate_files;
use planus_types::intermediate::{AbsolutePath, DeclarationIndex, DeclarationKind};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser)]
pub struct App {
    #[clap(value_hint = ValueHint::FilePath)]
    data_file: PathBuf,

    root_type: String,

    #[clap(value_hint = ValueHint::FilePath, required = true)]
    schema_files: Vec<PathBuf>,
}

fn main() -> Result<ExitCode> {
    let args = App::parse();
    let buffer = std::fs::read(args.data_file)?;

    let Some(declarations) = translate_files(&args.schema_files)
    else {
        return Ok(ExitCode::FAILURE);
    };

    let root_type = AbsolutePath(args.root_type.split('.').map(|s| s.to_owned()).collect());
    let Some((root_table_index, _, root_declaration)) = declarations.declarations.get_full(&root_type)
    else {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut matching_paths = declarations
            .declarations
            .iter()
            .filter(|(_path, declaration)| matches!(declaration.kind, DeclarationKind::Table(_)))
            .filter_map(|(path, _declaration)| {
                let path = path.to_string();
                Some((
                    std::cmp::Reverse(matcher.fuzzy_match(&path, &args.root_type)?),
                    path,
                ))
            })
            .collect::<Vec<_>>();

        matching_paths.sort();
        if matching_paths.is_empty() {
            println!("Could not find root type {:?}.", args.root_type);
        } else {
            println!(
                "Could not find root type {:?}. These are a few of the closest matching tables:",
                args.root_type
            );
            for (_score, path) in matching_paths.iter().take(5) {
                println!("- {path}");
            }
        }

        return Ok(ExitCode::FAILURE);
    };

    if !matches!(root_declaration.kind, DeclarationKind::Table(_)) {
        println!(
            "Type {} is not a table, but a {}",
            args.root_type,
            root_declaration.kind.kind_as_str()
        );
        return Ok(ExitCode::FAILURE);
    }

    let inspector = Inspector::new(
        planus_buffer_inspection::InspectableFlatbuffer {
            declarations: &declarations,
            buffer: &buffer,
        },
        DeclarationIndex(root_table_index),
    );

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Fix terminal on panic while preserving the message
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |x| {
        let mut stdout = io::stdout();
        cleanup_terminal(&mut stdout).ok();
        hook(x);
    }));

    // create app and run it

    let res = run_inspector(&mut terminal, inspector);

    // Cleanup and display errors if any
    cleanup_terminal(terminal.backend_mut().deref_mut())?;

    if let Err(err) = res {
        println!("{:?}", err);
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cleanup_terminal(stdout: &mut impl std::io::Write) -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;
    Ok(())
}
