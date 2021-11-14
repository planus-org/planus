use std::{env, fs, io::Result, process::Command};

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let flatc_dir = format!("{}/flatc", out_dir);
    fs::create_dir_all(&flatc_dir)?;

    Command::new("flatc")
        .args(&["--rust", "-o", &flatc_dir, "../files/conformance.fbs"])
        .output()?;

    let planus_dir = format!("{}/planus", out_dir);
    fs::create_dir_all(&planus_dir)?;

    planus_cli::codegen::generate_code(
        &["../files/conformance.fbs"],
        format!("{}/conformance_generated.rs", planus_dir),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=../files/conformance.fbs");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
