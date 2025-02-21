use std::{env, fmt::Write, fs, process::Command};

use color_eyre::{
    eyre::{bail, eyre, WrapErr},
    Result,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Create API tests
    let planus_api_dir = format!("{out_dir}/planus_api");
    generate_test_code("api_files", &planus_api_dir, None, false)?;

    // Create serialize/deserialize tests
    let planus_test_dir = format!("{out_dir}/planus_test");
    let planus_test_no_flatc_dir = format!("{out_dir}/planus_test_no_flatc");
    let serialize_template = std::fs::read_to_string("src/test_template.rs").ok();
    generate_test_code(
        "test_files",
        &planus_test_dir,
        serialize_template.as_deref(),
        true,
    )?;

    generate_test_code(
        "test_files_no_flatc",
        &planus_test_no_flatc_dir,
        serialize_template.as_deref(),
        false,
    )?;

    Ok(())
}

fn generate_test_code(
    in_dir: &str,
    out_dir: &str,
    template: Option<&str>,
    generate_flatc: bool,
) -> Result<()> {
    fs::create_dir_all(out_dir).wrap_err_with(|| eyre!("Cannot create dir: {}", out_dir))?;

    let mut mod_code = String::new();

    // We want the same generated files as here in rust-build, but not the tests.
    // Symlinking the relevant files and adding this check was the least bad option
    // I could think of, but it's still not pretty.
    let is_main_crate = std::env::var("CARGO_PKG_NAME").unwrap() == "rust-test";

    for entry in std::fs::read_dir(in_dir).wrap_err_with(|| eyre!("Cannot read dir: {}", in_dir))? {
        let entry = entry.wrap_err("Error doing readdir")?;
        let file_path = entry.path();
        if !file_path.is_dir()
            && file_path
                .extension()
                .is_some_and(|extension| extension == "fbs")
        {
            let file_stem = file_path.file_stem().unwrap().to_str().unwrap();

            // Generate planus code
            let generated = format!("{file_stem}_planus_generated.rs");
            let generated_full_path = format!("{out_dir}/{generated}");
            let Some(declarations) = planus_translation::translate_files(&[&file_path]) else {
                bail!("Cannot translate code for {}", file_path.display())
            };
            let code = planus_codegen::generate_rust(&declarations)
                .wrap_err_with(|| eyre!("Cannot codegen for {}", file_path.display()))?;
            std::fs::write(&generated_full_path, code)
                .wrap_err_with(|| eyre!("Cannot write output to {}", generated_full_path))?;

            let flatc_generated = format!("{file_stem}_generated.rs");
            if generate_flatc && is_main_crate {
                assert!(Command::new("flatc")
                    .args(["--rust", "-o", out_dir])
                    .arg(&file_path)
                    .status()
                    .wrap_err("Cannot run flatc")?
                    .success());
            }

            // Generate test module
            let code_module_name = file_stem.to_string();
            let code_file_full_path = format!("{out_dir}/{code_module_name}.rs");
            let mut code = String::new();
            writeln!(code, "#[path = {generated:?}]").unwrap();
            writeln!(code, "#[allow(clippy::module_inception)]").unwrap();
            writeln!(code, "pub mod generated;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use generated::*;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use core::{{fmt::Debug, hash::Hash}};").unwrap();
            if generate_flatc && is_main_crate {
                writeln!(code, "#[path = {flatc_generated:?}]").unwrap();
                writeln!(code, "#[allow(unused_imports, clippy::all)]").unwrap();
                writeln!(code, "pub mod flatc;").unwrap();
            }
            writeln!(code).unwrap();
            writeln!(code, "#[allow(dead_code)]").unwrap();
            writeln!(
                code,
                "const FILE_PATH: &str = \"{in_dir}/{code_module_name}\";"
            )
            .unwrap();
            writeln!(code).unwrap();

            if let Some(template) = template {
                if generate_flatc {
                    code += template;
                } else {
                    let (start, end) = template.split_once("<FLATC>").unwrap();
                    let (_mid, end) = end.split_once("</FLATC>").unwrap();
                    code += start;
                    code += end;
                }
            } else if is_main_crate {
                let mut path = file_path.to_owned();
                path.set_extension("rs");
                if let Ok(test_code) = std::fs::read_to_string(&path) {
                    writeln!(code, "#[test] fn {code_module_name}() {{").unwrap();
                    code += &test_code;
                    writeln!(code, "}}").unwrap();
                }
            }

            std::fs::write(code_file_full_path, code)
                .wrap_err_with(|| eyre!("Cannot write the file {}", generated_full_path))?;

            // Generate glue code
            writeln!(mod_code, "pub mod {code_module_name};").unwrap();
        }
    }

    std::fs::write(format!("{out_dir}/mod.rs"), mod_code)
        .wrap_err("Cannot write the api glue code")?;

    Ok(())
}
