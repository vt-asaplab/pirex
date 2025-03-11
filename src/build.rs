
use std::env;

fn main()
{
    env::set_var("CXX", "g++");

    let arch = env::consts::ARCH;

    if arch == "x86_64" // server arch
    {
        cc::Build::new()
            .flag("-Wno-unused-function")
            .flag("-Wno-unused-result")
            .flag("-fopenmp")
            .flag("-mavx2")
            .cpp(true)
            .file("utils/helper.cpp")
            .compile("helper.a");

        println!("cargo:rustc-link-lib=gomp");

        let current_dir = env::current_dir().expect("error get directory");
        let secp256k1_lib_path = current_dir.join("secp256k1/.libs");

        // Link with the secp256k1 library and specify the search path
        println!("cargo:rustc-link-lib=static=secp256k1");
        println!("cargo:rustc-link-search=native={}", secp256k1_lib_path.display());
        println!("cargo:warning=secp256k1 library path: {}", secp256k1_lib_path.display());
        // Print the path (for debugging purposes)
    }
    else
    {
        cc::Build::new()
            .flag("-Wno-unused-function")
            .cpp(true)
            .file("utils/helper.cpp")
            .compile("helper.a");

        let current_dir = env::current_dir().expect("error get directory");
        let secp256k1_lib_path = current_dir.join("secp256k1/.libs");

        // Link with the secp256k1 library and specify the search path
        println!("cargo:rustc-link-lib=static=secp256k1");
        println!("cargo:rustc-link-search=native={}", secp256k1_lib_path.display());
        println!("cargo:warning=secp256k1 library path: {}", secp256k1_lib_path.display());
        // Print the path (for debugging purposes)
    }
}