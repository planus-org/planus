use std::{env, fmt::Write, fs, process::Command};

use anyhow::{format_err, Context, Result};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    // Create flatbuffers files
    let out_dir = env::var("OUT_DIR").unwrap();
    let flatc_dir = format!("{}/flatc", out_dir);
    fs::create_dir_all(&flatc_dir).context("Cannot create flatc dir")?;
    Command::new("flatc")
        .args(&["--rust", "-o", &flatc_dir, "../files/test/conformance.fbs"])
        .output()
        .context("Cannot run flatc")?;

    // Create unit tests
    let planus_dir = format!("{}/planus", out_dir);
    fs::create_dir_all(&planus_dir).context("Cannot create planus out dir")?;
    for file in ["conformance", "enums", "structs", "unions", "vectors"] {
        let input_files = &[format!("../files/test/{}.fbs", file)];
        let output_file = format!("{}/{}_generated.rs", planus_dir, file);
        planus_cli::codegen::rust::generate_code(input_files, &output_file)
            .with_context(|| format_err!("Cannot generate code for {}", file))?;
        println!("cargo:rerun-if-changed=../files/{}.fbs", file);
    }

    // Create API tests
    let planus_api_dir = format!("{}/planus_api", out_dir);
    generate_test_code("api_files", &planus_api_dir, None)?;

    // Create serialize/deserialize tests
    let planus_test_dir = format!("{}/planus_test", out_dir);
    let serialize_template = std::fs::read_to_string("src/test_template.rs")
        .context("could not read serialize template")?;
    generate_test_code("test_files", &planus_test_dir, Some(&serialize_template))?;

    Ok(())
}

fn generate_test_code(in_dir: &str, out_dir: &str, template: Option<&str>) -> Result<()> {
    fs::create_dir_all(out_dir).with_context(|| format_err!("Cannot create dir: {}", out_dir))?;

    let mut mod_code = String::new();

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
            let generated = format!("{}_generated.rs", file_stem);
            let generated_full_path = format!("{}/{}", out_dir, generated);
            planus_cli::codegen::rust::generate_code(&[&file_path], &generated_full_path)
                .with_context(|| format_err!("Cannot generate code for {}", file_path.display()))?;

            // Generate test module
            let code_module_name = file_stem.to_string();
            let code_file_full_path = format!("{}/{}.rs", out_dir, code_module_name);
            let mut code = String::new();
            writeln!(code, "#[path = {:?}]", generated).unwrap();
            writeln!(code, "mod generated;").unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(code, "use generated::*;").unwrap();
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
                code += template;
            } else {
                writeln!(code, "#[test] fn {}() {{", code_module_name).unwrap();
                let mut path = file_path.to_owned();
                path.set_extension("rs");
                code += &std::fs::read_to_string(path).context("Cannot read associated rs")?;
                writeln!(code, "}}").unwrap();
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
