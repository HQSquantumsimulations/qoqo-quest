use cmake::Config;
#[cfg(feature = "openmp")]
use cmake_find_openmp::get_variables_c;
#[cfg(feature = "rebuild")]
use std::env;
#[cfg(feature = "rebuild")]
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "rebuild")]
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    #[cfg(feature = "openmp")]
    let (
        openmp_c_flags,
        _openmp_c_libraries,
        lib_location,
        lib_name,
    )= get_variables_c();
    // use CMake to build quest and return path where the static library is placed
    #[cfg(feature = "openmp")]
    let quest_library_path = Config::new("QuEST/QuEST")
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        // activated openmp mulit-threading
        .define("MULTITHREADED", "1")
        // .define("CMAKE_C_COMPILER", "clang")
        .build()
        .join("build/");

    #[cfg(not(feature = "openmp"))]
    let quest_library_path = Config::new("QuEST/QuEST")
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        // deactivates multi-threading
        .define("MULTITHREADED", "0")
        .build()
        .join("build/");
    println!(
        "cargo:rustc-link-search=native={}",
        quest_library_path.display()
    );
    println!("cargo:rustc-link-lib=static=QuEST");
    // set compiler flags
    #[cfg(feature = "openmp")]
    println!("cargo:flag={}", openmp_c_flags);
    // get compiler flags
    #[cfg(feature = "openmp")]
    println!("cargo:rustc-link-search=native={}", lib_location);
    //get compiler flags
    #[cfg(feature = "openmp")]
    println!("cargo:rustc-link-lib={}", lib_name);
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
