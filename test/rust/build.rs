use std::{env, fmt::Write, fs, process::Command};

use anyhow::{format_err, Context, Result};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Create API tests
    let planus_api_dir = format!("{}/planus_api", out_dir);
    generate_test_code("api_files", &planus_api_dir, None, false)?;

    // Create serialize/deserialize tests
    let planus_test_dir = format!("{}/planus_test", out_dir);
    let serialize_template = std::fs::read_to_string("src/test_template.rs").ok();
    generate_test_code(
        "test_files",
        &planus_test_dir,
        serialize_template.as_deref(),
        true,
    )?;

    generate_test_code(
        "test_files_no_flatc",
        &planus_test_dir,
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
    fs::create_dir_all(out_dir).with_context(|| format_err!("Cannot create dir: {}", out_dir))?;

    let mut mod_code = String::new();

    // We want the same generated files as here in rust-build, but not the tests.
    // Symlinking the relevant files and adding this check was the least bad option
    // I could think of, but it's still not pretty.
    let is_main_crate = std::env::var("CARGO_PKG_NAME").unwrap() == "rust_test";

    for entry in
        std::fs::read_dir(in_dir).with_context(|| format_err!("Cannot read dir: {}", in_dir))?
    {
        let entry = entry.context("Error doing readdir")?;
        let file_path = entry.path();
        if !file_path.is_dir()
            && file_path
                .extension()
                .map_or(false, |extension| extension == "fbs")
        {
            let file_stem = file_path.file_stem().unwrap().to_str().unwrap();

            // Generate planus code
            let generated = format!("{}_planus_generated.rs", file_stem);
            let generated_full_path = format!("{}/{}", out_dir, generated);
            planus_cli::codegen::rust::generate_code(&[&file_path], &generated_full_path)
                .with_context(|| format_err!("Cannot generate code for {}", file_path.display()))?;

            let flatc_generated = format!("{}_generated.rs", file_stem);
            if generate_flatc && is_main_crate {
                assert!(Command::new("flatc")
                    .args(&["--rust", "-o", out_dir])
                    .arg(&file_path)
                    .status()
                    .context("Cannot run flatc")?
                    .success());
            }

            // Generate test module
            let code_module_name = file_stem.to_string();
            let code_file_full_path = format!("{}/{}.rs", out_dir, code_module_name);
            let mut code =
                "#![allow(unused_unsafe, unused_imports, dead_code, clippy::all)]".to_string();
            writeln!(code, "#[path = {:?}]", generated).unwrap();
            writeln!(code, "mod generated;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use generated::*;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use core::{{fmt::Debug, hash::Hash}};").unwrap();
            if generate_flatc && is_main_crate {
                writeln!(code, "#[path = {:?}]", flatc_generated).unwrap();
                writeln!(code, "mod flatc;").unwrap();
            }
            writeln!(code).unwrap();
            writeln!(code, "#[allow(dead_code)]").unwrap();
            writeln!(
                code,
                "const FILE_PATH: &str = \"{}/{}\";",
                in_dir, code_module_name
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
                    writeln!(code, "#[test] fn {}() {{", code_module_name).unwrap();
                    code += &test_code;
                    writeln!(code, "}}").unwrap();
                }
            }

            std::fs::write(&code_file_full_path, code)
                .with_context(|| format_err!("Cannot write the file {}", generated_full_path))?;

            // Generate glue code
            writeln!(mod_code, "pub mod {};", code_module_name).unwrap();
        }
    }

    std::fs::write(format!("{}/mod.rs", out_dir), mod_code)
        .context("Cannot write the api glue code")?;

    Ok(())
}
