use std::path::Path;
use std::process::Command;

fn main() {
    let soroban_governor_path = Path::new("soroban-governor");
    let output = Command::new("make")
        .current_dir(soroban_governor_path)
        .arg("build")
        .status()
        .expect("Failed to build soroban-governor");
    assert_eq!(
        output.code().expect("Process did not exit correctly"),
        0,
        "Build failed"
    );
    println!("cargo::rerun-if-changed=soroban-governor");
}
