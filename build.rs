use std::env;

fn main() {
    // Only build the C++ bridge when the experimental MLX feature is enabled.
    if env::var("CARGO_FEATURE_MLX").is_ok() {
        let mut bridge = cxx_build::bridge("src/mlx_bridge.rs");
        bridge
            .file("src/mlx_bridge.cc")
            .flag_if_supported("-std=c++17")
            .include("include")
            .compile("git_ca_mlx_bridge");

        println!("cargo:rerun-if-changed=src/mlx_bridge.rs");
        println!("cargo:rerun-if-changed=src/mlx_bridge.cc");
        println!("cargo:rerun-if-changed=include/git_ca/mlx_bridge.h");
    }
}
