use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let dawn_lib_dir =
        env::var("DAWN_LIB_DIR").expect("environment variable `DAWN_LIB_DIR` must be set");
    let build_target = env::var("TARGET").unwrap();
    let dawn_out = Path::new("../../build")
        .join(&build_target)
        .join("dawn")
        .canonicalize()
        .unwrap();

    println!("cargo:rustc-link-search=native={dawn_lib_dir}");

    let common_libs = [
        "absl_base",
        "absl_int128",
        "absl_log_severity",
        "absl_raw_logging_internal",
        "absl_spinlock_wait",
        "absl_str_format_internal",
        "absl_strings_internal",
        "absl_strings",
        "absl_throw_delegate",
        "dawn_common",
        "dawn_headers",
        "dawn_native",
        "dawn_platform",
        "dawn_proc",
        "dawncpp_headers",
        "SPIRV-Tools-opt",
        "SPIRV-Tools",
        "tint_diagnostic_utils",
        "tint",
    ];

    for lib in common_libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    // Additional platform-specific libraries we need to link
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let libs: &[_] = match target_os.as_str() {
        "windows" => &["dxguid"],
        "macos" => &["framework=Foundation", "framework=IOKit"],
        "linux" => &["X11"],
        _ => &[],
    };

    for lib in libs {
        println!("cargo:rustc-link-lib={lib}");
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dawn_src = Path::new("../../external/dawn");

    let webgpu_header = dawn_out.join("gen/include/dawn/webgpu.h");

    // Generate bindings for the webgpu header.
    bindgen::builder()
        .header(webgpu_header.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out.join("webgpu.rs"))
        .expect("failed to write bindings to output file");

    let mut cc = cc::Build::new();

    cc.file("src/lib.cpp")
        .cpp(true)
        .include(dawn_src.join("include"))
        .include(dawn_out.join("gen/include"));

    if build_target == "x86_64-pc-windows-msvc" {
        cc.flag("/std:c++17").flag("/MD");
    } else {
        cc.flag("-std=c++17");
    }

    // Compile and link the c++ wrapper code for dawn initialisation.
    cc.compile("dawn_init");

    println!("cargo:rerun-if-changed=src/lib.cpp");
}
