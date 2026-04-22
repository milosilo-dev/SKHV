use std::process::Command;
use walkdir::WalkDir;

const ASM: &str = "nasm";
const CC: &str = "i686-elf-gcc";
const LD: &str = "i686-elf-ld";
const OBJ: &str = "objcopy";
const FIRMWARE_PATH: &str = "guest/firmware";

fn build_firmware() {
    let asm_input = FIRMWARE_PATH.to_owned() + "/entry.asm";
    let asm_output = FIRMWARE_PATH.to_owned() + "/entry.o";

    let status = Command::new(ASM)
        .args(["-f", "elf32", asm_input.as_str(), "-o", asm_output.as_str()])
        .status()
        .expect("failed to run nasm");

    if !status.success() {
        panic!("nasm failed to assemble firmware entry stub");
    }

    let cc_input = FIRMWARE_PATH.to_owned() + "/main.c";
    let cc_output = FIRMWARE_PATH.to_owned() + "/main.o";

    let status = Command::new(CC)
        .args(["-m32", "-ffreestanding", "-fno-stack-protector", "-nostdlib", "-isystem", "/usr/lib/gcc/x86_64-linux-gnu/13/include", "-O2", "-c", cc_input.as_str(), "-o", cc_output.as_str()])
        .status()
        .expect("failed to run gcc");

    if !status.success() {
        panic!("gcc failed to compile firmware c_main");
    }

    let ld_output = FIRMWARE_PATH.to_owned() + "/out.elf";
    let ld_script = FIRMWARE_PATH.to_owned() + "/linker.ld";

    let status = Command::new(LD)
        .args(["-T", ld_script.as_str(), "-o", ld_output.as_str(), asm_output.as_str(), cc_output.as_str()])
        .status()
        .expect("failed to run ld");

    if !status.success() {
        panic!("ld failed to link firmware");
    }

    let obj_output = FIRMWARE_PATH.to_owned() + "/out.bin";

    let status = Command::new(OBJ)
        .args(["-O", "binary", ld_output.as_str(), obj_output.as_str()])
        .status()
        .expect("failed to run objcopy");

    if !status.success() {
        panic!("objcopy failed to create flat binary");
    }
}

fn build() {
    build_firmware();

    for entry in WalkDir::new("guest/test").into_iter().filter_map(Result::ok) {
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