#[cfg(not(feature = "zig"))]
use cmake::Config;
#[cfg(any(feature = "zig", feature = "rebuild"))]
use std::env;
#[cfg(feature = "zig")]
use std::fs;
use std::path::PathBuf;
#[cfg(any(feature = "zig"))]
use std::process::Command;

fn main() {

    #[cfg(any(feature = "zig", feature = "rebuild"))]
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(not(feature = "zig"))]
    let quest_library_path = standard_cmake_build();
    #[cfg(all(feature = "zig", target_os = "macos"))]
    let quest_library_path = build_with_zig_macos_universal_2(out_dir_path.clone());
    #[cfg(all(feature = "zig", target_os = "linux"))]
    let quest_library_path = build_with_zig_linux(out_dir_path.clone());

    println!(
        "cargo:rustc-link-search=native={}",
        quest_library_path.display()
    );
    println!("cargo:rustc-link-lib=static=QuEST");
    println!("cargo:rerun-if-changed=wrapper.h");

    // list functions for which bindings should be created
    #[cfg(feature = "rebuild")]
    let builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("create.*")
        .allowlist_function("cloneQureg")
        .allowlist_function("destroy.*")
        .allowlist_function("report.*")
        .allowlist_function("get.*")
        .allowlist_function("set.*")
        .allowlist_function("setComplexMatrixN")
        .allowlist_function("initStateFromAmps")
        .allowlist_function("report.*")
        .allowlist_function("rotate.*")
        .allowlist_function("report.*")
        .allowlist_function("controlled.*")
        .allowlist_function("report.*")
        .allowlist_function("multiControlled.*")
        .allowlist_function("report.*")
        .allowlist_function("pauli.*")
        .allowlist_function("report.*")
        .allowlist_function("init.*")
        .allowlist_function("report.*")
        .allowlist_function("mix.*")
        .allowlist_function("sGate")
        .allowlist_function("tGate")
        .allowlist_function("phaseShift")
        .allowlist_function("compactUnitary")
        .allowlist_function("apply.*")
        .allowlist_function("hadamard*")
        .allowlist_function("swapGate")
        .allowlist_function("unitary")
        .allowlist_function("twoQubitUnitary")
        .allowlist_function("multiQubitUnitary")
        .allowlist_function("measure")
        .allowlist_function("statevec_twoQubitUnitary")
        .allowlist_function("calc.*");

    #[cfg(feature = "rebuild")]
    let bindings = builder.generate().expect("Unable to generate bindings");

    #[cfg(feature = "rebuild")]
    bindings
        .write_to_file(out_dir_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(all(feature = "zig", target_os = "macos"))]
fn build_with_zig_macos_universal_2(out_dir: PathBuf) -> PathBuf {
    let zigtarget_aarch = Some("aarch64-macos-none");
    let zigtarget_x86 = Some("x86_64-macos-none");

    let build_zig_path = PathBuf::from("QuEST").join("QuEST").join("build.zig");

    let mut zig_run_aarch = Command::new("zig");
    zig_run_aarch.arg("build");
    zig_run_aarch.arg("--build-file");
    zig_run_aarch.arg(build_zig_path.to_str().unwrap().to_string());
    if let Some(ztarget) = zigtarget_aarch {
        zig_run_aarch.arg(format!("-Dtarget={}", ztarget));
    }

    let mut zig_run_x86 = Command::new("zig");
    zig_run_x86.arg("build");
    zig_run_x86.arg("--build-file");
    zig_run_x86.arg(build_zig_path.to_str().unwrap().to_string());

    if let Some(ztarget) = zigtarget_x86 {
        zig_run_x86.arg(format!("-Dtarget={}", ztarget));
    }

    let quest_library_path = out_dir.clone();
    let quest_library_path_aarch = quest_library_path.join("aarch64");
    let quest_library_path_x86 = quest_library_path.join("x86_64");

    zig_run_aarch.arg("-p");
    zig_run_aarch.arg(format!("{}", quest_library_path_aarch.display()));

    zig_run_x86.arg("-p");
    zig_run_x86.arg(format!("{}", quest_library_path_x86.display()));

    match zig_run_aarch.status() {
        Ok(status) => {
            if !status.success() {
                panic!("Could not build QuEST library for macos x86_64 with zig");
            }
        }
        Err(err) => {
            panic!("Could not build QuEST library for macos x86_64 with zig: {err}");
        }
    };

    match zig_run_x86.status() {
        Ok(status) => {
            if !status.success() {
                panic!("Could not build QuEST library for macos x86_64 with zig");
            }
        }
        Err(err) => {
            panic!("Could not build QuEST library for macos x86_64 with zig: {err}");
        }
    };

    let x86_files = fs::read_dir(quest_library_path_x86.join("lib")).unwrap();
    let x86_file = x86_files.into_iter().next().unwrap().unwrap().path();
    let aarch_files = fs::read_dir(quest_library_path_aarch.join("lib")).unwrap();
    let aarch_file = aarch_files.into_iter().next().unwrap().unwrap().path();
    let mut combination_writer = fat_macho::FatWriter::new();
    if let Err(err) =  combination_writer.add(fs::read(&x86_file).unwrap()) {
        if !matches!(err, fat_macho::Error::InvalidMachO(_)){
            panic!("{err}")
        }
    }
    if let Err(err) =  combination_writer.add(fs::read(&aarch_file).unwrap()) {
        if !matches!(err, fat_macho::Error::InvalidMachO(_)){
            panic!("{err}")
        }
    }
    let quest_library_path = out_dir.join("universal2-apple-darwin").join("lib");

    fs::create_dir_all(quest_library_path.clone()).unwrap();
    combination_writer.write_to_file(quest_library_path.join("libQuEST.a")).unwrap();
    quest_library_path
}

#[cfg(all(feature = "zig", target_os = "linux"))]
fn build_with_zig_linux(out_dir: PathBuf) -> PathBuf {
    let zigtarget: Option<&str> = None;
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let zigtarget_aarch = Some("aarch64-linux-gnu");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let zigtarget = Some("x86_64-linux-gnu");
    let build_zig_path = PathBuf::from("QuEST").join("QuEST").join("build.zig");

    let mut zig_run = Command::new("zig");
    zig_run.arg("build");
    zig_run.arg("--build-file");
    zig_run.arg(build_zig_path.to_str().unwrap().to_string());
    if let Some(ztarget) = zigtarget {
        zig_run.arg(format!("-Dtarget={}", ztarget));
    }
    zig_run.arg("-flto");

    let quest_library_path = out_dir.clone();

    zig_run.arg("-p");
    zig_run.arg(format!("{}", quest_library_path.display()));

    match zig_run.status() {
        Ok(status) => {
            if !status.success() {
                panic!("Could not build QuEST library for linux with zig");
            }
        }
        Err(err) => {
            panic!("Could not build QuEST library for linux with zig: {err}");
        }
    };
    quest_library_path.join("lib")
}

#[cfg(not(feature = "zig"))]
fn standard_cmake_build() -> PathBuf {
    // use CMake to build quest and return path where the static library is placed
    let partial_quest_path = PathBuf::from("QuEST").join("QuEST");
    #[cfg(feature = "openmp")]
    let quest_library_path = Config::new(partial_quest_path)
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        // activated openmp mulit-threading
        .define("MULTITHREADED", "1")
        // .define("CMAKE_C_COMPILER", "clang")
        .build()
        .join("build");

    #[cfg(not(feature = "openmp"))]
    let quest_library_path = 
        Config::new(partial_quest_path)
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        // .define("CMAKE_OSX_ARCHITECTURES","x86_64;arm64")
        // deactivates multi-threading
        .define("MULTITHREADED", "0")
        .build()
        .join("build");
    
    quest_library_path
}
