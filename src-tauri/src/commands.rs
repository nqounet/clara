use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use crate::cli::execute_cli;
use crate::config::{
    get_atoms_dir, get_config_path, get_global_settings_path, init_workspace, load_app_config,
    AppConfig, ClaraConfig,
};
use crate::models::{ClaraAtom, ClaraFrontmatter};
use crate::parser::{
    generate_slug, parse_ai_response, parse_atom_body, parse_skr_results, SkrSearchResult,
};

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    load_app_config().map_err(|e| format!("設定の読み込みに失敗しました: {}", e))
}

#[tauri::command]
pub fn update_root_dir(new_path: String) -> Result<AppConfig, String> {
    let trimmed = new_path.trim();
    if trimmed.is_empty() {
        return Err("Vaultのパスを指定してください。".into());
    }

    let mut config = load_app_config().map_err(|e| format!("設定の読み込みに失敗: {}", e))?;

    let path_buf = PathBuf::from(trimmed);
    config.root_dir = path_buf.clone();

    if !path_buf.exists() {
        fs::create_dir_all(&path_buf).map_err(|e| format!("ディレクトリの作成に失敗: {}", e))?;
    }

    let config_path = get_global_settings_path();
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| format!("設定の保存に失敗: {}", e))?;

    Ok(config)
}

#[tauri::command]
pub fn list_recent_atoms(limit: usize) -> Result<Vec<ClaraFrontmatter>, String> {
    let app_config = load_app_config().map_err(|e| e.to_string())?;
    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    if !atoms_dir.exists() {
        return Ok(vec![]);
    }

    let mut frontmatters = Vec::new();
    let entries = fs::read_dir(atoms_dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "md" {
            if let Ok(content) = fs::read_to_string(&path) {
                let parts: Vec<&str> = content.splitn(3, "---").collect();
                if parts.len() >= 3 {
                    let yaml_str = parts[1].trim();
                    if let Ok(frontmatter) = serde_yaml::from_str::<ClaraFrontmatter>(yaml_str) {
                        frontmatters.push(frontmatter);
                    }
                }
            }
        }
    }

    frontmatters.sort_by(|a, b| b.id.cmp(&a.id));
    frontmatters.truncate(limit);

    Ok(frontmatters)
}

#[tauri::command]
pub fn load_atom(id: String) -> Result<ClaraAtom, String> {
    let app_config = load_app_config().map_err(|e| e.to_string())?;
    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    let entries = fs::read_dir(atoms_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with(&id) && file_name.ends_with(".md") {
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;

                let parts: Vec<&str> = content.splitn(3, "---").collect();
                if parts.len() < 3 {
                    return Err("ファイルフォーマットが不正です".into());
                }

                let frontmatter: ClaraFrontmatter =
                    serde_yaml::from_str(parts[1].trim()).map_err(|e| e.to_string())?;
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

    Err("指定されたIDのファイルが見つかりません".into())
}

#[tauri::command]
pub fn get_clara_config() -> Result<ClaraConfig, String> {
    let (_, clara_config) = init_workspace().map_err(|e| e.to_string())?;
    Ok(clara_config)
}

#[tauri::command]
pub fn update_clara_config(
    cli_command: String,
    model: Option<String>,
    working_dir: Option<String>,
) -> Result<ClaraConfig, String> {
    if cli_command.trim().is_empty() {
        return Err("CLIコマンド名は必須です。空欄にできません。".into());
    }

    let (_, mut clara_config) = init_workspace().map_err(|e| e.to_string())?;

    clara_config.cli_command = cli_command;
    clara_config.model = model.filter(|s| !s.trim().is_empty());
    clara_config.working_dir = working_dir
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from);

    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(&clara_config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| format!("設定の保存に失敗: {}", e))?;

    Ok(clara_config)
}

#[tauri::command]
pub async fn create_and_send_prompt(
    window: tauri::Window,
    description: Option<String>,
    prompt: String,
    parent_id: Option<String>,
    yolo: bool,
) -> Result<ClaraAtom, String> {
    let (app_config, config) = init_workspace().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let id = now.format("%Y%m%d%H%M%S").to_string();

    let system_instruction = "Please generate a title, a short description, a URL-safe slug, and related tags for this request, then provide your answer.\nYou MUST format your output exactly as follows:\n\nTITLE: [Your generated title]\nDESC: [A short summary, or leave empty if not needed]\nSLUG: [lowercase ASCII slug using hyphens only, max 30 chars, e.g. rust-tauri-setup]\nTAGS: [comma-separated tags]\n---\n[Your actual response]\n\n";
    let full_prompt = format!("{}{}", system_instruction, prompt);

    let raw_response = execute_cli(&window, &full_prompt, &config, yolo)?;

    let parsed = parse_ai_response(&raw_response);

    let slug = parsed.slug.unwrap_or_else(|| generate_slug(&parsed.title));
    let full_id = format!("{}-{}", id, slug);

    let frontmatter = ClaraFrontmatter {
        title: parsed.title.clone(),
        description: description.or(parsed.description),
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

    let yaml = serde_yaml::to_string(&frontmatter).map_err(|e| e.to_string())?;

    let markdown_content = format!(
        "---\n{}---\n\n~~~~~~user\n{}\n~~~~~~\n\n~~~~~~ai\n{}\n~~~~~~",
        yaml, prompt, parsed.body
    );

    let mut file_path = get_atoms_dir(&app_config.root_dir);
    file_path.push(format!("{}.md", full_id));
    fs::write(&file_path, markdown_content).map_err(|e| e.to_string())?;

    Ok(ClaraAtom {
        frontmatter,
        prompt,
        response: parsed.body,
    })
}

#[tauri::command]
pub async fn search_skr(
    window: tauri::Window,
    query: String,
) -> Result<Vec<SkrSearchResult>, String> {
    let (app_config, clara_config) = init_workspace().map_err(|e| e.to_string())?;

    let mut config = clara_config.clone();
    config.working_dir = Some(get_atoms_dir(&app_config.root_dir));

    let system_instruction = "Search the Semantic Knowledge Repository (SKR) for the following query.\nYou MUST format each search result exactly as follows:\n\nID: [id (filename without extension)]\nTITLE: [title]\nSCORE: [relevance score]\nSNIPPET: [short snippet]\n\nQuery: ";
    let full_prompt = format!("{}{}", system_instruction, query);

    let raw_output = execute_cli(&window, &full_prompt, &config, false)?;

    Ok(parse_skr_results(&raw_output))
}
