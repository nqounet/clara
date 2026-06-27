use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub root_dir: PathBuf,
}

pub fn get_default_root_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("ホームディレクトリが見つかりません");
    path.push(".clara");
    path.push("atoms");
    path
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            root_dir: get_default_root_dir(),
        }
    }
}

pub fn get_global_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap());
    path.push("net.nqou.clara");
    path.push("settings.json");
    path
}

pub fn load_app_config() -> Result<AppConfig, AppError> {
    let config_path = get_global_settings_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let default_config = AppConfig::default();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, json)?;
        Ok(default_config)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaraConfig {
    pub cli_command: String,
    pub cli_args: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
    #[serde(default)]
    pub workspace_history: Vec<String>,
}

impl Default for ClaraConfig {
    fn default() -> Self {
        Self {
            cli_command: "gemini".to_string(),
            cli_args: vec![],
            model: None,
            working_dir: None,
            workspace_history: vec![],
        }
    }
}

/// Vault（Atom保管ディレクトリ）のパスを取得
pub fn get_atoms_dir(base_dir: &Path) -> PathBuf {
    base_dir.to_path_buf()
}

/// ~/.clara/clara.config.json のパスを取得（Vaultとは独立した設定ファイル）
pub fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("ホームディレクトリが見つかりません");
    path.push(".clara");
    path.push("clara.config.json");
    path
}

/// ディレクトリ構造と設定ファイルの初期化
pub fn init_workspace() -> Result<(AppConfig, ClaraConfig), AppError> {
    let app_config = load_app_config()?;
    let base_dir = &app_config.root_dir;
    let config_path = get_config_path();

    if !base_dir.exists() {
        fs::create_dir_all(base_dir)?;
    }

    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let clara_config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content)?
    } else {
        let default_config = ClaraConfig::default();
        let json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, json)?;
        default_config
    };

    Ok((app_config, clara_config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClaraConfig::default();
        assert_eq!(config.cli_command, "gemini");
        assert!(config.cli_args.is_empty());
    }

    #[test]
    fn test_directory_paths() {
        let base_dir = PathBuf::from("/mock/clara/atoms");

        let atoms_dir = get_atoms_dir(&base_dir);
        assert_eq!(atoms_dir, PathBuf::from("/mock/clara/atoms"));

        let config_path = get_config_path();
        assert!(config_path.to_string_lossy().contains(".clara"));
        assert!(config_path.to_string_lossy().ends_with("clara.config.json"));
    }

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();
        let default_root = get_default_root_dir();
        assert_eq!(config.root_dir, default_root);
    }

    #[test]
    fn test_global_settings_path() {
        let path = get_global_settings_path();
        assert!(
            path.to_string_lossy().contains("clara") || path.to_string_lossy().contains("Clara")
        );
        assert!(path.ends_with("settings.json"));
    }
}
