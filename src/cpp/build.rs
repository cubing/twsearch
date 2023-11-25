fn main() {
    let cpp_files = [
        "antipode.cpp",
        "calcsymm.cpp",
        "canon.cpp",
        "cityhash/src/city.cc",
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
    build
        .flag("-std=c++17")
        .flag("-DASLIBRARY")
        .flag("-DTWSEARCH_VERSION=v0.4.2-7-g4a9107fa")
        // .flag("-lpthread") # Unneeded?
        .flag("-DUSE_PTHREADS")
        .compile("twsearch-cpp-wrapper");

    for cpp_file in cpp_files {
        println!("cargo:rerun-if-changed={}", cpp_file);
    }
}
