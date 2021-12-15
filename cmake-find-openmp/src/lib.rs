// Copyright © 2021 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

//! # cmake-find-openmp
//! 
//! When using C/C++ libraries that implement multithreading with the help of OpenMP the final executable must also be linked against the correct OpenMP library.
//! If the C/C++ code is build with CMake the correct compiler and linker options can be detected with the find_package(OpenMP) CMake function.
//! 
//! This crate provides helper functions to extract the correct linker and compiler options and apply them to rustc in a build script.
//! 
//! The intended usecase are rust crates that (somewhere in the dependency tree) link against a C/C++ library that uses OpenMP and is build with CMake.
//! The helper functions in this crate are meant to link the final binary against the same OpenMP libraries found by CMake when building the dependencies.
//! 
//! ## Examples
//! 
//! When the fi

use cmake::Config;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;


/// Set openmp linking options for the final binary as discovered by CMake.
/// 
/// This function can be called in the `build.rs` script of the rust crate.
/// It cannot be used when the same `build.rs` script also builds a library with the [cmake] crate.
/// In that case use a combination of [crate::get_variables_c] an setting the flags manually.
/// 
/// # Example
/// 
/// This is an example build.rs
/// 
/// ```rust
/// use cmake_find_openmp::link_openmp;
/// 
/// fn main() {
///     link_openmp()
/// }
/// ```
pub fn link_openmp() {
    let (
        openmp_c_flags,
        _openmp_c_libraries,
        lib_location,
        lib_name,
    ) = get_variables_c();

    // set compiler flags
    
    println!("cargo:flag={}", openmp_c_flags);
    // println!("cargo:rustc-link-search=native={}", openmp_c_library);
    println!("cargo:rustc-link-search=all={}", lib_location);
    // // get compiler flags
    println!("cargo:rustc-link-lib={}",lib_name);
    // set compiler flags
}

/// Type for C variables
///
/// OpenMP_<LANG>_FLAGS
/// OpenMP_<LANG>_LIBRARIES
/// library folder
/// cropped library name
type CmakeOpenMpVariables = (String, String, String, String);

/// Get OpenMP variables from CMakes FindOpenMP function.
/// 
/// Calls CMake to extract the compiler flags, the location of the OpenMP library
/// and the library name in linker compatible form.
/// Linker compatible form means that (assuming a linux system) the fulllibrary name might be `libgomp.so`
/// then the linker compatible form is `omp` so it can be linked with `-lomp`.
///
/// Use this function if you also build the external library in the `build.rs` build script
/// because the order of calls of the [cmake::Config::build] is important
/// 
/// # Example
/// 
/// ```
/// use cmake_find_openmp::get_variables_c;
/// use cmake::Config;
///
/// fn main() {
/// 
/// let (
/// openmp_c_flags,
/// _openmp_c_libraries,
/// lib_location,
/// lib_name,
/// )= get_variables_c();
/// 
/// let path = Config::new("THE_DEPENDENCY_FOLDER_WITH_CMakeLists.txt")
/// .build();
/// 
/// println!("cargo:flag={}", openmp_c_flags);
/// println!("cargo:rustc-link-search=native={}", lib_location);
/// println!("cargo:rustc-link-lib={}", lib_name);
///
/// }
/// ```
pub fn get_variables_c() -> CmakeOpenMpVariables {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    create_cmakelists(&out_dir_path);

    // use CMake to find OpenMP properties and store in temporary files
    let tmp_file_path = Config::new(out_dir_path)
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        .build()
        .join("build/");
    // get compiler flags
    let openmp_c_flags = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_C_FLAGS"))
        .expect("Could not read OpenMP_C_FLAGS");

    let openmp_c_libraries = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_C_LIBRARIES"))
        .expect("Could not read tmp_OpenMP_C_LIBRARIES");
    let _openmp_c_include_dirs = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_C_INCLUDE_DIRS"))
        .expect("Could not read tmp_OpenMP_C_INCLUDE_DIRS");

    let _openmp_c_library = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_C_LIBRARY"))
        .expect("Could not read tmp_OpenMP_C_LIBRARY");
    let _openmp_c_lib_names = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_C_LIB_NAMES"))
        .expect("Could not read tmp_OpenMP_C_LIB_NAMES");
    
        let libraries_path = PathBuf::from(openmp_c_libraries.clone());
        let lib_location = libraries_path.parent().expect("Could not find library folder of OpenMP_C_LIBRARIES");
        let full_lib_name =  PathBuf::from(libraries_path.file_name().expect("Could not find library name OpenMP_C_LIBRARIES"));
        let lib_name = full_lib_name.file_stem().expect("Could not find library name OpenMP_C_LIBRARIES").to_str().expect("Could not find library name OpenMP_C_LIBRARIES").to_string();
        // remove leading "lib" depending on plattform to link with argument
        let prefix: &str = env::consts::DLL_PREFIX;
        let lib_name = lib_name.trim_start_matches(prefix);

    (
        openmp_c_flags,
        openmp_c_libraries,
        lib_location.to_str().expect("Could not display OpenMP library location").to_string(),
        lib_name.to_string() 
    )
}

/// Get OpenMP variables from CMakes FindOpenMP function
///
/// Returns:
///
/// C_VARIABLES
pub fn get_variables_cxx() -> CmakeOpenMpVariables {
    let out_dir_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    create_cmakelists(&out_dir_path);

    // use CMake to find OpenMP properties and store in temporary files
    let tmp_file_path = Config::new(out_dir_path)
        .no_build_target(true)
        .very_verbose(true)
        .always_configure(true)
        .build()
        .join("build/");
    // get compiler flags

    let openmp_cxx_flags = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_CXX_FLAGS"))
        .expect("Could not read OpenMP_CXX_FLAGS");

    let openmp_cxx_libraries = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_CXX_LIBRARIES"))
        .expect("Could not read tmp_OpenMP_CXX_LIBRARIES");
    let _openmp_cxx_include_dirs =
        fs::read_to_string(tmp_file_path.join("tmp_OpenMP_CXX_INCLUDE_DIRS"))
            .expect("Could not read tmp_OpenMP_CXX_INCLUDE_DIRS");

    let _openmp_cxx_library = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_CXX_LIBRARY"))
        .expect("Could not read tmp_OpenMP_CXX_LIBRARY");
    let _openmp_cxx_lib_names = fs::read_to_string(tmp_file_path.join("tmp_OpenMP_CXX_LIB_NAMES"))
        .expect("Could not read tmp_OpenMP_CXX_LIB_NAMES");
    
        let libraries_path = PathBuf::from(openmp_cxx_libraries.clone());
        let lib_location = libraries_path.parent().expect("Could not find library folder of OpenMP_C_LIBRARIES");
        let full_lib_name =  PathBuf::from(libraries_path.file_name().expect("Could not find library name OpenMP_C_LIBRARIES"));
        let lib_name = full_lib_name.file_stem().expect("Could not find library name OpenMP_C_LIBRARIES").to_str().expect("Could not find library name OpenMP_C_LIBRARIES").to_string();
        // remove leading "lib" depending on plattform to link with argument
        let prefix: &str = env::consts::DLL_PREFIX;
        let lib_name = lib_name.trim_start_matches(prefix);

        (
            openmp_cxx_flags,
            openmp_cxx_libraries,
            lib_location.to_str().expect("Could not display OpenMP library location").to_string(),
            lib_name.to_string()
        )
}

pub fn create_cmakelists(outdir: &PathBuf) {
    let cmake_source = r#"
# Simple 

# CMake initialisation.
cmake_minimum_required(VERSION 3.10)

# Project name
project(SuWaves VERSION 1.0.0 LANGUAGES C CXX)

#set C/C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD_REQUIRED True)
set(CMAKE_C_STANDARD_REQUIRED True)

# Find OpenMP


find_package(OpenMP REQUIRED)
message(STATUS "Using OpenMP version ${OpenMP_VERSION}")
set(ENV{OpenMP_VERSION} ${OpenMP_C_VERSION})

message(STATUS "OpenMP_CXX_INCLUDE_DIRS: ${OpenMP_CXX_INCLUDE_DIRS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_INCLUDE_DIRS ${OpenMP_CXX_INCLUDE_DIRS})
message(STATUS "OpenMP_C_INCLUDE_DIRS: ${OpenMP_C_INCLUDE_DIRS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_INCLUDE_DIRS ${OpenMP_C_INCLUDE_DIRS})

message(STATUS "OpenMP_CXX_FLAGS: ${OpenMP_CXX_FLAGS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_FLAGS ${OpenMP_CXX_FLAGS})
message(STATUS "OpenMP_C_FLAGS: ${OpenMP_C_FLAGS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_FLAGS ${OpenMP_C_FLAGS})

message(STATUS "OpenMP_CXX_FLAGS: ${OpenMP_CXX_FLAGS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_FLAGS ${OpenMP_CXX_FLAGS})
message(STATUS "OpenMP_C_FLAGS: ${OpenMP_C_FLAGS}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_FLAGS ${OpenMP_C_FLAGS})

message(STATUS "OpenMP_CXX_LIBRARIES: ${OpenMP_CXX_LIBRARIES}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_LIBRARIES ${OpenMP_CXX_LIBRARIES})
message(STATUS "OpenMP_C_LIBRARIES: ${OpenMP_C_LIBRARIES}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_LIBRARIES ${OpenMP_C_LIBRARIES})

message(STATUS "OpenMP_CXX_LIBRARY: ${OpenMP_CXX_LIBRARY}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_LIBRARY ${OpenMP_CXX_LIBRARY})
message(STATUS "OpenMP_C_LIBRARY: ${OpenMP_C_LIBRARY}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_LIBRARY ${OpenMP_C_LIBRARY})

message(STATUS "OpenMP_CXX_LIB_NAMES: ${OpenMP_CXX_LIB_NAMES}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_CXX_LIB_NAMES ${OpenMP_CXX_LIB_NAMES})
message(STATUS "OpenMP_C_LIB_NAMES: ${OpenMP_C_LIB_NAMES}")
file(WRITE ${CMAKE_BINARY_DIR}/tmp_OpenMP_C_LIB_NAMES ${OpenMP_C_LIB_NAMES})

# Find Threading

find_package(Threads REQUIRED)


add_executable(dummy dummy.c)
# linking libraries
target_link_libraries(dummy PUBLIC OpenMP::OpenMP_C
)
add_executable(dummycpp dummy.cpp)
target_link_libraries(dummycpp PUBLIC OpenMP::OpenMP_CXX
)

target_compile_options(dummy PUBLIC ${OpenMP_C_FLAGS})
target_compile_options(dummycpp PUBLIC ${OpenMP_CXX_FLAGS})
"#;

    let mut cmake_file =
        fs::File::create(outdir.join("CMakeLists.txt")).expect("Could not create CMakesLists.txt");
    write!(cmake_file, "{}", cmake_source).expect("Could not write to CMakeLists.txt");

    let c_source = r#"
int main(){ return 0;}
"#;

    let mut c_file = fs::File::create(outdir.join("dummy.c")).expect("Could not create dummy.c");
    write!(c_file, "{}", c_source).expect("Could not write to dummy.c");

    let mut cpp_file =
        fs::File::create(outdir.join("dummy.cpp")).expect("Could not create dummy.cpp");
    write!(cpp_file, "{}", c_source).expect("Could not write to dummy.cpp");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_openmp() {
        env::set_var("OUT_DIR", "/tmp/test_build");
        env::set_var("TARGET", "aarch64-apple-darwin");
        env::set_var("HOST", "aarch64-apple-darwin");
        env::set_var("PROFILE", "release");
        env::set_var("OPT_LEVEL", "0");
        env::set_var("DEBUG", "0");
        link_openmp();
    }
}
