use chrono::Utc;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            _ => normalized.push(component),
        }
    }
    normalized
}

use crate::cli::execute_cli;
use crate::config::{
    get_atoms_dir, get_config_path, get_global_settings_path, init_workspace, load_app_config,
    AppConfig, ClaraConfig,
};
use crate::models::{ClaraAtom, ClaraFrontmatter};
use crate::parser::{generate_slug, parse_ai_response, parse_atom_body, SkrSearchResult};
use tauri::Manager;

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, crate::error::AppError> {
    load_app_config()
}

#[tauri::command]
pub fn update_root_dir(new_path: String) -> Result<AppConfig, crate::error::AppError> {
    let trimmed = new_path.trim();
    if trimmed.is_empty() {
        return Err("Vaultのパスを指定してください。".into());
    }

    let path_buf = PathBuf::from(trimmed);
    if !path_buf.is_absolute() {
        return Err("Vaultのパスは絶対パスで指定してください。".into());
    }

    let normalized_path = normalize_path(&path_buf);

    if let Some(home) = dirs::home_dir() {
        if !normalized_path.starts_with(&home) {
            return Err(
                "セキュリティ上の理由により、Vaultのパスはホームディレクトリ内に設定してください。"
                    .into(),
            );
        }
    }

    let mut config = load_app_config()?;

    config.root_dir = normalized_path.clone();

    if !normalized_path.exists() {
        fs::create_dir_all(&normalized_path)?;
    }

    let config_path = get_global_settings_path();
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, json)?;

    Ok(config)
}

#[tauri::command]
pub fn list_recent_atoms(limit: usize) -> Result<Vec<ClaraFrontmatter>, crate::error::AppError> {
    let app_config = load_app_config()?;
    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    if !atoms_dir.exists() {
        return Ok(vec![]);
    }

    let mut frontmatters = Vec::new();
    let entries = fs::read_dir(atoms_dir)?;

    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.extension().unwrap_or_default() == "md" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let parts: Vec<&str> = content.splitn(3, "---").collect();
                        if parts.len() >= 3 {
                            let yaml_str = parts[1].trim();
                            if let Ok(frontmatter) =
                                serde_yaml::from_str::<ClaraFrontmatter>(yaml_str)
                            {
                                frontmatters.push(frontmatter);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Failed to read directory entry: {}", e),
        }
    }

    frontmatters.sort_by(|a, b| b.id.cmp(&a.id));
    frontmatters.truncate(limit);

    Ok(frontmatters)
}

#[tauri::command]
pub fn load_atom(id: String) -> Result<ClaraAtom, crate::error::AppError> {
    let app_config = load_app_config()?;
    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    let entries = fs::read_dir(atoms_dir)?;
    for entry_result in entries {
        match entry_result {
            Ok(entry) => {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&id) && file_name.ends_with(".md") {
                        let content = fs::read_to_string(&path)?;

                        let parts: Vec<&str> = content.splitn(3, "---").collect();
                        if parts.len() < 3 {
                            return Err("ファイルフォーマットが不正です".into());
                        }

                        let frontmatter: ClaraFrontmatter = serde_yaml::from_str(parts[1].trim())?;
                        let body = parts[2];

                        let (prompt, response) = parse_atom_body(body);

                        return Ok(ClaraAtom {
                            frontmatter,
                            prompt,
                            response,
                        });
                    }
                }
            }
            Err(e) => eprintln!("Failed to read directory entry: {}", e),
        }
    }

    Err("指定されたIDのファイルが見つかりません".into())
}

#[tauri::command]
pub fn get_clara_config() -> Result<ClaraConfig, crate::error::AppError> {
    let (_, clara_config) = init_workspace()?;
    Ok(clara_config)
}

#[tauri::command]
pub fn update_clara_config(
    cli_command: String,
    model: Option<String>,
    working_dir: Option<String>,
) -> Result<ClaraConfig, crate::error::AppError> {
    if cli_command.trim().is_empty() {
        return Err("CLIコマンド名は必須です。空欄にできません。".into());
    }

    let cmd_path = std::path::Path::new(&cli_command);
    if let Some(name) = cmd_path.file_name().and_then(|n| n.to_str()) {
        let allowed_commands = ["gemini", "gemini-cli", "deba", "agy"];
        if !allowed_commands.contains(&name) {
            let allowed_str = allowed_commands
                .iter()
                .map(|c| format!("'{c}'"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("CLIコマンドは {} のみを許可しています。", allowed_str).into());
        }
    } else {
        return Err("無効なCLIコマンドです。".into());
    }

    let wd_string = working_dir
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Workspaceディレクトリは必須です。空欄にできません。".to_string())?;

    let path_buf = PathBuf::from(&wd_string);
    if !path_buf.is_absolute() {
        return Err("Workspaceディレクトリは絶対パスで指定してください。".into());
    }
    let normalized_path = normalize_path(&path_buf);

    if let Some(home) = dirs::home_dir() {
        if !normalized_path.starts_with(&home) {
            return Err(
                "セキュリティ上の理由により、Workspaceはホームディレクトリ内に設定してください。"
                    .into(),
            );
        }
    }
    let safe_working_dir = Some(normalized_path.clone());

    let (_, mut clara_config) = init_workspace()?;

    clara_config.cli_command = cli_command;
    clara_config.model = model.filter(|s| !s.trim().is_empty());
    clara_config.working_dir = safe_working_dir;

    clara_config.workspace_history.retain(|x| x != &wd_string);
    clara_config
        .workspace_history
        .insert(0, normalized_path.to_string_lossy().into_owned());
    if clara_config.workspace_history.len() > 999 {
        clara_config.workspace_history.truncate(999);
    }

    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(&clara_config)?;
    fs::write(&config_path, json)?;

    Ok(clara_config)
}

#[tauri::command]
pub fn remove_workspace_history(path: String) -> Result<ClaraConfig, crate::error::AppError> {
    let (_, mut clara_config) = init_workspace()?;
    clara_config.workspace_history.retain(|x| x != &path);
    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(&clara_config)?;
    fs::write(&config_path, json)?;
    Ok(clara_config)
}

#[tauri::command]
pub async fn create_and_send_prompt(
    window: tauri::Window,
    prompt: String,
    parent_id: Option<String>,
    yolo: bool,
) -> Result<ClaraAtom, crate::error::AppError> {
    let (app_config, config) = init_workspace()?;

    let now = Utc::now();
    let id = now.format("%Y%m%d%H%M%S").to_string();

    let parent_atom = parent_id.as_ref().and_then(|pid| {
        match load_atom(pid.clone()) {
            Ok(atom) => Some(atom),
            Err(e) => {
                eprintln!("警告: 親Atom ({}) の読み込みに失敗しました: {}", pid, e);
                None
            }
        }
    });

    let system_instruction = "Please generate a title, a short description, a URL-safe slug, and related tags for this request, then provide your answer.\nYou MUST format your output exactly as follows:\n\nTITLE: [Your generated title]\nDESC: [A short summary, or leave empty if not needed]\nSLUG: [lowercase ASCII slug using hyphens only, max 30 chars, e.g. rust-tauri-setup]\nTAGS: [comma-separated tags]\n---\n[Your actual response]\n\n";
    let full_prompt = build_prompt(system_instruction, parent_atom.as_ref(), &prompt);

    let window_clone = window.clone();
    let config_clone = config.clone();
    let prompt_clone = full_prompt.clone();
    let raw_response = tauri::async_runtime::spawn_blocking(move || {
        execute_cli(&window_clone, &prompt_clone, &config_clone, yolo)
    })
    .await
    .map_err(|e| format!("スレッドの実行に失敗: {}", e))??;

    let parsed = parse_ai_response(&raw_response);

    let slug = parsed.slug.unwrap_or_else(|| generate_slug(&parsed.title));
    let full_id = format!("{}-{}", id, slug);

    let frontmatter = ClaraFrontmatter {
        title: parsed.title.clone(),
        description: parsed.description,
        id: full_id.clone(),
        parent_id: parent_id.clone(),
        parent: parent_id.as_ref().map(|pid| format!("[[{}]]", pid)),
        created_at: now,
        tags: parsed.tags,
        cli_command: Some(config.cli_command.clone()),
        model: config.model.clone(),
        workspace: config
            .working_dir
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned()),
        yolo,
    };

    let yaml = serde_yaml::to_string(&frontmatter)?;

    let markdown_content = format!(
        "---\n{}---\n\n~~~~~~user\n{}\n~~~~~~\n\n~~~~~~ai\n{}\n~~~~~~",
        yaml, prompt, parsed.body
    );

    let mut file_path = get_atoms_dir(&app_config.root_dir);
    file_path.push(format!("{}.md", full_id));
    fs::write(&file_path, markdown_content)?;

    Ok(ClaraAtom {
        frontmatter,
        prompt,
        response: parsed.body,
    })
}

#[tauri::command]
pub fn search_skr(
    app_handle: tauri::AppHandle,
    query: String,
) -> Result<Vec<SkrSearchResult>, crate::error::AppError> {
    let (app_config, _) = init_workspace()?;

    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    let state = app_handle.state::<crate::search::SearchState>();

    let state_inner = state.inner();

    let results = state_inner.search(&query, &atoms_dir)?;
    Ok(results)
}

pub fn build_prompt(
    system_instruction: &str,
    parent_atom: Option<&crate::models::ClaraAtom>,
    prompt: &str,
) -> String {
    match parent_atom {
        Some(parent) => {
            format!(
                "{}Here is the context of the previous conversation. Please read this history to understand the background and reply to the new message accordingly.\n\n[Previous Conversation]\nUser: {}\nAssistant: {}\n\n[New Message]\n{}",
                system_instruction, parent.prompt, parent.response, prompt
            )
        }
        None => format!("{}{}", system_instruction, prompt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[test]
    fn test_build_prompt_without_parent() {
        let system_instruction = "System Instruction\n";
        let prompt = "My prompt";
        let result = build_prompt(system_instruction, None, prompt);
        assert_eq!(result, "System Instruction\nMy prompt");
    }

    #[test]
    fn test_build_prompt_with_parent() {
        let system_instruction = "System Instruction\n";
        let prompt = "My follow up prompt";
        
        let parent = crate::models::ClaraAtom {
            frontmatter: crate::models::ClaraFrontmatter {
                title: "Parent Title".to_string(),
                description: None,
                id: "20260706120000-parent".to_string(),
                parent_id: None,
                parent: None,
                created_at: chrono::Utc::now(),
                tags: vec![],
                cli_command: Some("gemini".to_string()),
                model: None,
                workspace: None,
                yolo: false,
            },
            prompt: "Parent prompt".to_string(),
            response: "Parent response".to_string(),
        };

        let result = build_prompt(system_instruction, Some(&parent), prompt);
        
        let expected = format!(
            "System Instruction\nHere is the context of the previous conversation. Please read this history to understand the background and reply to the new message accordingly.\n\n[Previous Conversation]\nUser: Parent prompt\nAssistant: Parent response\n\n[New Message]\nMy follow up prompt"
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_update_root_dir_empty() {
        let result: Result<crate::config::AppConfig, AppError> = update_root_dir("   ".to_string());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Vaultのパスを指定してください。");
    }

    #[test]
    fn test_update_clara_config_empty_command() {
        let result: Result<crate::config::ClaraConfig, AppError> =
            update_clara_config("   ".to_string(), None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "CLIコマンド名は必須です。空欄にできません。"
        );
    }
    #[test]
    fn test_update_clara_config_command_validation() {
        // 許可されていない無効なコマンドはエラーになること
        let result = update_clara_config("invalid-cli".to_string(), None, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "CLIコマンドは 'gemini', 'gemini-cli', 'deba', 'agy' のみを許可しています。"
        );

        // 'agy' も許可されるべきなので、コマンドチェックを通過し、次の Workspace チェックでエラーになること
        let result_agy = update_clara_config("agy".to_string(), None, None);
        assert!(result_agy.is_err());
        assert_eq!(
            result_agy.unwrap_err().to_string(),
            "Workspaceディレクトリは必須です。空欄にできません。"
        );

        // 許可されている "gemini" はコマンドチェックを通過し、次の Workspace チェックでエラーになること
        let result_gemini = update_clara_config("gemini".to_string(), None, None);
        assert!(result_gemini.is_err());
        assert_eq!(
            result_gemini.unwrap_err().to_string(),
            "Workspaceディレクトリは必須です。空欄にできません。"
        );
    }

    #[test]
    fn test_type_signatures() {
        // This test ensures that all these functions return a Result<_, AppError>.
        // If they return Result<_, String>, this test will fail to compile (RED).
        let _f1: fn() -> Result<crate::config::AppConfig, AppError> = get_app_config;
        let _f2: fn(usize) -> Result<Vec<crate::models::ClaraFrontmatter>, AppError> =
            list_recent_atoms;
        let _f3: fn(String) -> Result<crate::models::ClaraAtom, AppError> = load_atom;
        let _f4: fn() -> Result<crate::config::ClaraConfig, AppError> = get_clara_config;
        let _f5: fn(String) -> Result<crate::config::ClaraConfig, AppError> =
            remove_workspace_history;
        // create_and_send_prompt is async, so its type is complex. We'll skip it in this dummy check.
        // search_skr takes tauri::AppHandle which we can't easily dummy, skip.
    }
}
