use cfg_if::cfg_if;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut manifest_dir = PathBuf::from(manifest_dir);
    manifest_dir.push("native".to_owned());
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86_64" => {
            if os == "windows" {
                "win-x64"
            } else if os == "linux" {
                "linux-x64"
            } else {
                panic!("Unsupported os, currently linux x86_64 and windows arch x86, x86_64 only")
            }
        }
        "x86" => {
            if os == "windows" {
                "win-x86"
            } else {
                panic!("Unsupported os, currently linux x86_64 and windows arch x86, x86_64 only")
            }
        }
        _ => panic!("Unsupported target, currently linux x86_64 and windows arch x86, x86_64 only")
    };

    manifest_dir.push(arch);
    let lib_dir = manifest_dir;
    println!("cargo:rustc-link-search={}", lib_dir.display());
    let mut exe_pth = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    if os == "windows" {
        cfg_if! {
            if #[cfg(feature = "staticlib")] {
                // let is_msvc = std::env::var("CARGO_CFG_TARGET_ENV").map_or(false, |env| env == "msvc");
                // if is_msvc {
                //     println!("cargo:rustc-link-lib=static=egui_extras");
                // } else {
                //     println!("cargo:rustc-link-lib=static=libegui_extras");
                // }
                panic!("staticlib feature currently unsupported due to rust-lld: error: duplicate symbol: rust_eh_personality issue on release mode")
            } else {
                exe_pth.push("../../../egui_extras.dll");
                std::fs::copy(format!("{}/egui_extras.dll", lib_dir.display()), exe_pth.to_str().unwrap()).unwrap();
                println!("cargo:rustc-link-lib=egui_extras.dll")
            }
        }
    } else if os == "linux" {
        cfg_if! {
            if #[cfg(feature = "staticlib")] {
                 panic!("staticlib feature currently unsupported due to rust-lld: error: duplicate symbol: rust_eh_personality issue on release mode");
            } else {
                exe_pth.push("../../../libegui_extras.so");
                std::fs::copy(format!("{}/libegui_extras.so", lib_dir.display()), exe_pth.to_str().unwrap()).unwrap();
                println!("cargo:rustc-link-lib=dylib=egui_extras")            }
        }
    } else {
        panic!("Unsupported target, currently linux x86_64 and windows arch x86, x86_64 only")
    }
}
