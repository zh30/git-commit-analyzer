#[allow(dead_code)]
#[cxx::bridge(namespace = "git_ca")]
pub mod ffi {
    unsafe extern "C++" {
        include!("git_ca/mlx_bridge.h");

        fn load_model(model_path: &str) -> Result<()>;
        fn generate_commit(diff_text: &str) -> Result<String>;
    }
}
