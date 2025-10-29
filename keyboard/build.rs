fn main() {
    #[cfg(feature = "csound")]
    csound()
}

#[cfg(feature = "csound")]
fn csound() {
    use std::env;
    use std::path::PathBuf;
    use std::process::Command;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // The cmake file outputs lines prefixed with `!` in a specific order --
    // see `csound-helper/CMakeLists.txt`.
    let output = Command::new("cmake")
        .arg("-S")
        .arg("csound-helper")
        .arg("-B")
        .arg(out_path.join("cmake").as_os_str())
        .output()
        .expect("cmake failed");
    let lines: Vec<_> = String::from_utf8_lossy(&output.stderr)
        .split("\n")
        .filter_map(|x| x.strip_prefix("!"))
        .map(str::to_string)
        .collect();
    if lines.len() != 3 {
        panic!("cmake generated unexpected output: {lines:?}");
    }
    if lines[0] != "TRUE" {
        panic!("cmake did not found csound");
    }
    let include_dir = PathBuf::from(&lines[1]).to_str().unwrap().to_string();
    let full_lib = PathBuf::from(&lines[2]);
    let lib_dir = full_lib
        .parent()
        .expect("lib not in a directory")
        .to_str()
        .unwrap();
    let lib = full_lib.file_name().unwrap().to_str().unwrap();
    let lib = if lib_dir.starts_with("framework=") {
        lib.to_string()
    } else {
        lib.strip_prefix("lib")
            .expect("library didn't start with lib")
            .replace(".so", "")
    };
    println!("cargo:rustc-link-search={lib_dir}");
    println!("cargo:rustc-link-lib={lib}");
    let bindings = bindgen::Builder::default()
        .header("csound-helper/csound_wrapper.h")
        .clang_arg(format!("-I{include_dir}"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_item("FP_ILOGB0")
        .blocklist_item("FP_ILOGBNAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_NORMAL")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_ZERO")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("csound_bindings.rs"))
        .expect("Couldn't write bindings!");
}
