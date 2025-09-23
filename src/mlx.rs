use std::borrow::Cow;
use std::error::Error;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MlxError {
    message: Cow<'static, str>,
}

impl MlxError {
    #[allow(dead_code)]
    pub fn new<M: Into<Cow<'static, str>>>(message: M) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MlxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for MlxError {}

#[cfg(feature = "mlx")]
mod native {
    use super::MlxError;
    use crate::mlx_bridge::ffi;
    use std::sync::OnceLock;

    #[allow(dead_code)]
    static MODEL_PATH: OnceLock<String> = OnceLock::new();

    #[allow(dead_code)]
    pub fn initialize(model_path: &str) -> Result<(), MlxError> {
        if MODEL_PATH.get().is_some() {
            return Ok(());
        }

        ffi::load_model(model_path)
            .map_err(|err| MlxError::new(format!("failed to load MLX model: {}", err)))?;

        MODEL_PATH
            .set(model_path.to_owned())
            .map_err(|_| MlxError::new("MLX model already initialized"))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn generate(diff: &str) -> Result<String, MlxError> {
        if MODEL_PATH.get().is_none() {
            return Err(MlxError::new(
                "MLX model not initialized. Call initialize() before generate().",
            ));
        }

        ffi::generate_commit(diff)
            .map_err(|err| MlxError::new(format!("MLX generation failed: {}", err)))
    }
}

#[cfg(not(feature = "mlx"))]
mod native {
    use super::MlxError;

    #[allow(dead_code)]
    pub fn initialize(_model_path: &str) -> Result<(), MlxError> {
        Err(MlxError::new(
            "The crate was built without the `mlx` feature. Rebuild with `--features mlx` to enable native MLX support.",
        ))
    }

    #[allow(dead_code)]
    pub fn generate(_diff: &str) -> Result<String, MlxError> {
        Err(MlxError::new(
            "The crate was built without the `mlx` feature. Rebuild with `--features mlx` to enable native MLX support.",
        ))
    }
}

#[allow(unused_imports)]
pub use native::{generate, initialize};
