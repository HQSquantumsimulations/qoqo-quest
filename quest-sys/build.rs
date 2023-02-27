
// use cmake::Config;
use std::fs;
use std::path::PathBuf;
use std::{env, path::Path};

fn main() {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").expect("Cannot find OUT_DIR"));

    let quest_library_path = build_with_cc(out_dir_path);

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

fn build_with_cc(out_dir: PathBuf) -> PathBuf {
    let base_path = Path::new("QuEST").join("QuEST");
    let src_path = base_path.join("src");
    let include_path = base_path.join("include");
    let files = [
        src_path.join("QuEST.c"),
        src_path.join("QuEST_common.c"),
        src_path.join("QuEST_qasm.c"),
        src_path.join("QuEST_validation.c"),
        src_path.join("mt19937ar.c"),
        src_path.join("CPU").join("QuEST_cpu.c"),
        src_path.join("CPU").join("QuEST_cpu_local.c"),
    ];
    let out_path = out_dir.join("build");
    fs::create_dir_all(out_path.clone()).expect("Cannot create directory for x86_64 library");

    #[cfg(target_arch = "x86_64")]
    cc::Build::new()
        .include(src_path.clone())
        .include(include_path.clone())
        .files(files.clone())
        .define("MULTITHREADED", "0")
        .opt_level(2)
        .debug(false)
        .warnings(false)
        .static_flag(true)
        .out_dir(out_path.clone())
        .flag("-std=c99")
        .flag("-mavx")
        .compile("QuEST");
    #[cfg(not(target_arch = "x86_64"))]
    cc::Build::new()
        .include(src_path)
        .include(include_path)
        .files(files)
        .define("MULTITHREADED", "0")
        .opt_level(2)
        .debug(false)
        .warnings(false)
        .static_flag(true)
        .out_dir(out_path.clone())
        .flag("-std=c99")
        .compile("QuEST");
    out_path
}

// fn standard_cmake_build() -> PathBuf {
//     // use CMake to build quest and return path where the static library is placed
//     let partial_quest_path = PathBuf::from("QuEST").join("QuEST");
//     #[cfg(feature = "openmp")]
//     let quest_library_path = Config::new(partial_quest_path)
//         .no_build_target(true)
//         .very_verbose(true)
//         .always_configure(true)
//         // activated openmp mulit-threading
//         .define("MULTITHREADED", "1")
//         // .define("CMAKE_C_COMPILER", "clang")
//         .build()
//         .join("build");

//     #[cfg(not(feature = "openmp"))]
//     let quest_library_path =
//         Config::new(partial_quest_path)
//         .no_build_target(true)
//         .very_verbose(true)
//         .always_configure(true)
//         // .define("CMAKE_OSX_ARCHITECTURES","x86_64;arm64")
//         // deactivates multi-threading
//         .define("MULTITHREADED", "0")
//         .build()
//         .join("build");
//     #[cfg(not( target_os = "windows"))]
//     return quest_library_path;
//     #[cfg( target_os = "windows")]
//     match env::var("PROFILE").expect("Cannot find PROFILE env variable").as_str(){
//         "debug" => {return quest_library_path.join("Debug");},
//         "release" => {return quest_library_path.join("Release");}
//         _ => {panic!("Profile is not debug or release. Correct windows library location not known.")}
//     }

// }
