use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("YAML Error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("{0}")]
    Message(String),
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Message(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Message(s)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert_eq!(err.to_string(), "I/O Error: not found");

        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"I/O Error: not found\"");
    }
}
