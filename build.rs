use std::process::Command;
use walkdir::WalkDir;

const ASM: &str = "nasm";

fn build() {
    for entry in WalkDir::new("guest").into_iter().filter_map(Result::ok) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("asm") {
            continue;
        }

        println!("Compiling {}", path.display());

        let input = path.to_str().unwrap();
        let output = path.with_extension("bin");
        let output = output.to_str().unwrap();

        let status = Command::new(ASM)
            .args(["-f", "bin", input, "-o", output])
            .status()
            .expect("failed to run nasm");

        if !status.success() {
            panic!("nasm failed on {}", input);
        }
    }
}

fn main() {
    build(); // MUST exit immediately
}