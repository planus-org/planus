use std::{env, fs, io::Result, process::Command};

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let flatc_dir = format!("{}/flatc", out_dir);
    fs::create_dir_all(&flatc_dir)?;

    Command::new("flatc")
        .args(&["--rust", "-o", &flatc_dir, "../files/test/conformance.fbs"])
        .output()?;

    // Generate planus files
    let planus_dir = format!("{}/planus", out_dir);
    fs::create_dir_all(&planus_dir)?;
    for file in ["conformance", "enums", "structs", "unions", "vectors"] {
        let input_files = &[format!("../files/test/{}.fbs", file)];
        let output_file = format!("{}/{}_generated.rs", planus_dir, file);
        eprintln!("{:?} {:?}", input_files, output_file);
        planus_cli::codegen::generate_code(input_files, output_file).unwrap();
        println!("cargo:rerun-if-changed=../files/{}.fbs", file);
    }
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
