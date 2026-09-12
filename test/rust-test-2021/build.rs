use std::{env, fmt::Write, fs, process::Command};

use color_eyre::{
    eyre::{bail, eyre, WrapErr},
    Result,
};

// Keep in sync with the pinned `flatbuffers` dep in `test/Cargo.toml` and with
// `flatbuffers` in `flake.nix`. Generated code is version-specific, so the
// locally-installed `flatc` must match the crate we link against.
const EXPECTED_FLATC_VERSION: &str = "25.12.19";

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PLANUS_SKIP_FLATC");
    println!("cargo:rerun-if-env-changed=PLANUS_REQUIRE_FLATC");

    let out_dir = env::var("OUT_DIR").unwrap();

    let require_flatc = env::var_os("PLANUS_REQUIRE_FLATC").is_some();
    let flatc_available = if env::var_os("PLANUS_SKIP_FLATC").is_some() {
        if require_flatc {
            bail!("PLANUS_SKIP_FLATC and PLANUS_REQUIRE_FLATC are both set — pick one.",);
        }
        println!("cargo:warning=PLANUS_SKIP_FLATC is set; skipping flatc-generated tests.");
        false
    } else {
        probe_flatc(require_flatc)?
    };

    // Create API tests. Companions may opt into flatc bindings (for cross-checking
    // against the upstream flatbuffers crate) by wrapping the relevant assertions in
    // `<FLATC>`/`</FLATC>` markers; see `generate_test_code`.
    let planus_api_dir = format!("{out_dir}/planus_api");
    generate_test_code("api_files", &planus_api_dir, None, flatc_available)?;

    // Create serialize/deserialize tests
    let planus_test_dir = format!("{out_dir}/planus_test");
    let planus_test_no_flatc_dir = format!("{out_dir}/planus_test_no_flatc");
    println!("cargo:rerun-if-changed=src/test_template.rs");
    let serialize_template = std::fs::read_to_string("src/test_template.rs").unwrap();
    generate_test_code(
        "test_files",
        &planus_test_dir,
        Some(&serialize_template),
        flatc_available,
    )?;

    generate_test_code(
        "test_files_no_flatc",
        &planus_test_no_flatc_dir,
        Some(&serialize_template),
        false,
    )?;

    Ok(())
}

// Returns `Ok(true)` when a local `flatc` matches `EXPECTED_FLATC_VERSION`. Otherwise, if
// `require_flatc` is `false` (the default), emits a `cargo:warning=` explaining what happened
// and returns `Ok(false)` so downstream test generation transparently falls back to the no-flatc
// path instead of producing generated Rust that fails to compile against the pinned `flatbuffers`
// crate. When `require_flatc` is `true` (set via `PLANUS_REQUIRE_FLATC=1` in CI), returns `Err`
// instead so a version drift fails the build rather than silently reducing coverage.
fn probe_flatc(require_flatc: bool) -> Result<bool> {
    // In lax mode a probe failure becomes a `cargo:warning=` and falls back to no-flatc codegen.
    // In strict mode (CI) the same failure aborts the build.
    let report = |problem: String| -> Result<bool> {
        if require_flatc {
            bail!(
                "{problem} PLANUS_REQUIRE_FLATC is set: install flatc {EXPECTED_FLATC_VERSION} \
                 (see flake.nix) or unset PLANUS_REQUIRE_FLATC.",
            )
        } else {
            println!(
                "cargo:warning={problem} Skipping flatc-generated tests. Install flatc \
                 {EXPECTED_FLATC_VERSION} (see flake.nix) or set PLANUS_SKIP_FLATC=1 to silence \
                 this warning.",
            );
            Ok(false)
        }
    };

    let output = match Command::new("flatc").arg("--version").output() {
        Ok(output) => output,
        Err(err) => return report(format!("Could not run `flatc --version`: {err}.")),
    };
    if !output.status.success() {
        return report(format!("`flatc --version` exited with {}.", output.status));
    }
    // Expected output: "flatc version 25.12.19\n"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(reported) = stdout.split_whitespace().last() else {
        return report(format!(
            "Could not parse `flatc --version` output {stdout:?}."
        ));
    };
    if reported == EXPECTED_FLATC_VERSION {
        return Ok(true);
    }
    report(format!(
        "Local flatc version {reported} does not match the expected {EXPECTED_FLATC_VERSION}.",
    ))
}

fn generate_test_code(
    in_dir: &str,
    out_dir: &str,
    template: Option<&str>,
    generate_flatc: bool,
) -> Result<()> {
    fs::create_dir_all(out_dir).wrap_err_with(|| eyre!("Cannot create dir: {}", out_dir))?;

    let mut mod_code = String::new();

    // Regenerate if a schema or companion is added/removed/renamed in the input directory.
    println!("cargo:rerun-if-changed={in_dir}");

    for entry in std::fs::read_dir(in_dir).wrap_err_with(|| eyre!("Cannot read dir: {}", in_dir))? {
        let entry = entry.wrap_err("Error doing readdir")?;
        let file_path = entry.path();
        if !file_path.is_dir()
            && file_path
                .extension()
                .is_some_and(|extension| extension == "fbs")
        {
            let file_stem = file_path.file_stem().unwrap().to_str().unwrap();

            // Regenerate when this schema changes.
            println!("cargo:rerun-if-changed={}", file_path.display());

            // Generate planus code
            let generated = format!("{file_stem}_planus_generated.rs");
            let generated_full_path = format!("{out_dir}/{generated}");
            let Some(declarations) = planus_translation::translate_files(&[&file_path]) else {
                bail!("Cannot translate code for {}", file_path.display())
            };
            let code = planus_codegen::generate_rust(&declarations, true)
                .wrap_err_with(|| eyre!("Cannot codegen for {}", file_path.display()))?;
            std::fs::write(&generated_full_path, code)
                .wrap_err_with(|| eyre!("Cannot write output to {}", generated_full_path))?;

            // Companion test bodies (the `template == None` case, i.e. `api_files`) live in a
            // sibling `.rs` file. Read it up front so we can decide whether this particular
            // schema needs flatc: only companions that actually use flatc (marked with a
            // `<FLATC>` section) get flatc bindings, so schemas flatc cannot parse — e.g. ones
            // using keywords as identifiers — stay planus-only even when flatc is available.
            let companion = if template.is_none() {
                let mut path = file_path.to_owned();
                path.set_extension("rs");
                println!("cargo:rerun-if-changed={}", path.display());
                std::fs::read_to_string(&path).ok()
            } else {
                None
            };
            let want_flatc = generate_flatc
                && match &companion {
                    Some(companion) => companion.contains("<FLATC>"),
                    None => template.is_some(),
                };

            let flatc_generated = format!("{file_stem}_generated.rs");
            if want_flatc {
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
            writeln!(
                code,
                "use core::{{convert::{{TryFrom, TryInto}}, fmt::Debug, hash::Hash}};"
            )
            .unwrap();
            writeln!(code, "#[allow(unused_imports)]").unwrap();
            writeln!(
                code,
                "use alloc::{{boxed::Box, format, string::String, vec, vec::Vec}};"
            )
            .unwrap();
            if want_flatc {
                writeln!(code, "#[path = {flatc_generated:?}]").unwrap();
                writeln!(
                    code,
                    "#[allow(unsafe_op_in_unsafe_fn, unused_imports, clippy::all)]"
                )
                .unwrap();
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
                code += &strip_flatc_if_needed(template, want_flatc);
            } else if let Some(companion) = companion {
                writeln!(code, "#[test] fn {code_module_name}() {{").unwrap();
                code += &strip_flatc_if_needed(&companion, want_flatc);
                writeln!(code, "}}").unwrap();
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

// Handle the `<FLATC>`/`</FLATC>` markers in a template or companion test body. When flatc code
// is available (`keep`), the markers are left in place as ordinary line comments and the code
// between them is compiled. Otherwise the marked section (which references the `flatc` module)
// is removed so the test still compiles without flatc. Bodies without markers are returned
// unchanged.
fn strip_flatc_if_needed(code: &str, keep: bool) -> String {
    if keep {
        return code.to_string();
    }
    match code.split_once("<FLATC>") {
        Some((start, rest)) => {
            let (_flatc, end) = rest
                .split_once("</FLATC>")
                .expect("`<FLATC>` marker without a matching `</FLATC>`");
            format!("{start}{end}")
        }
        None => code.to_string(),
    }
}
