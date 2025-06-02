use std::process::Command;

fn main() {
    let cpp_files = [
        "antipode.cpp",
        "canon.cpp",
        "vendor/cityhash/src/city.cc",
        "cmdlineops.cpp",
        "cmds.cpp",
        "ffi/ffi_api.cpp",
        "ffi/rust_api.cpp",
        "filtermoves.cpp",
        "generatingset.cpp",
        "god.cpp",
        "index.cpp",
        "parsemoves.cpp",
        "prunetable.cpp",
        "puzdef.cpp",
        "readksolve.cpp",
        "rotations.cpp",
        "solve.cpp",
        "test.cpp",
        "threads.cpp",
        "twsearch.cpp",
        "util.cpp",
        "workchunks.cpp",
    ];

    let mut build = cxx_build::bridge("./rs/main.rs");
    for cpp_file in cpp_files {
        build.file(cpp_file);
    }
    let version = String::from_utf8(
        Command::new("bun")
            .args(["run", "script/print-current-version-description.ts"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    build
        .flag("-std=c++20")
        .flag("-DASLIBRARY")
        .flag(format!("-DTWSEARCH_VERSION={}", version))
        // .flag("-lpthread") # Unneeded?
        .flag("-DUSE_PTHREADS")
        .compile("twsearch-cpp-wrapper");

    for cpp_file in cpp_files {
        println!("cargo:rerun-if-changed={}", cpp_file);
    }
}
