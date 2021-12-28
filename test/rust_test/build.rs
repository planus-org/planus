use std::{env, fmt::Write, fs, process::Command};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let flatc_dir = format!("{}/flatc", out_dir);
    fs::create_dir_all(&flatc_dir).context("Cannot create flatc dir")?;

    Command::new("flatc")
        .args(&["--rust", "-o", &flatc_dir, "../files/test/conformance.fbs"])
        .output()
        .context("Cannot run flatc")?;

    // Planus files for unit tests
    let planus_dir = format!("{}/planus", out_dir);
    fs::create_dir_all(&planus_dir).context("Cannot create planus out dir")?;
    for file in ["conformance", "enums", "structs", "unions", "vectors"] {
        let input_files = &[format!("../files/test/{}.fbs", file)];
        let output_file = format!("{}/{}_generated.rs", planus_dir, file);
        planus_cli::codegen::rust::generate_code(input_files, output_file)
            .with_context(|| anyhow::format_err!("Cannot generate code for {}", file))?;
        println!("cargo:rerun-if-changed=../files/{}.fbs", file);
    }
    println!("cargo:rerun-if-changed=build.rs");

    let planus_api_dir = format!("{}/planus_api", out_dir);
    fs::create_dir_all(&planus_api_dir).context("Cannot create planus api dir")?;

    let mut api_code = String::new();

    for entry in std::fs::read_dir("api_files").context("Cannot read api files")? {
        let entry = entry.context("Error doing readdir for api_files")?;
        if !entry.path().is_dir()
            && entry
                .path()
                .extension()
                .map_or(false, |extension| extension == "fbs")
        {
            let generated = format!(
                "{}_generated.rs",
                entry.path().file_stem().unwrap().to_str().unwrap()
            );
            let generated_full_path = format!("{}/{}", planus_api_dir, generated);
            planus_cli::codegen::rust::generate_code(&[entry.path()], generated_full_path)
                .with_context(|| {
                    anyhow::format_err!("Cannot generate code for {}", entry.path().display())
                })?;
            let api_mod = format!(
                "{}_api",
                entry.path().file_stem().unwrap().to_str().unwrap()
            );
            let api_file_full_path = format!("{}/{}.rs", planus_api_dir, api_mod);
            let mut code = String::new();
            writeln!(code, "#[path = {:?}]", generated).unwrap();
            writeln!(code, "mod generated;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use generated::*;").unwrap();
            writeln!(code).unwrap();
            writeln!(code, "#[test] fn {}() {{", api_mod).unwrap();

            let mut path = entry.path().to_owned();
            path.set_extension("rs");
            code += &std::fs::read_to_string(path)
                .context("Cannot read associated rs file for api test")?;
            writeln!(code, "}}").unwrap();

            std::fs::write(&api_file_full_path, code).with_context(|| {
                anyhow::format_err!("Cannot write the api file {}", api_file_full_path)
            })?;

            writeln!(api_code, "mod {};", api_mod).unwrap();
        }
    }

    std::fs::write(format!("{}/mod.rs", planus_api_dir), api_code)
        .context("Cannot write the api glue code")?;

    // for entry in std::fs::read_dir("test_files").context("Cannot read test_files")? {
    //     let entry = entry.context("Error doing readdir for test_files")?;
    //     let path = entry.path();
    //     if !path.is_dir()
    //         && path
    //             .extension()
    //             .map_or(false, |extension| extension == "fbs")
    //     {

    //     }
    // }

    Ok(())
}
