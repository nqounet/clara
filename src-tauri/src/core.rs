use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::models::{ClaraFrontmatter, ClaraSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaraConfig {
    pub cli_command: String,
    pub cli_args: Vec<String>,
}

impl Default for ClaraConfig {
    fn default() -> Self {
        Self {
            cli_command: "gemini".to_string(),
            // 必要に応じて "ask" などの引数を追加できるように
            cli_args: vec![],
        }
    }
}

/// ~/.clara のベースパスを取得
fn get_base_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("ホームディレクトリが見つかりません");
    path.push(".clara");
    path
}

/// ~/.clara/atomics のパスを取得
fn get_atomics_dir() -> PathBuf {
    let mut path = get_base_dir();
    path.push("atomics");
    path
}

/// ~/.clara/clara.config.json のパスを取得
fn get_config_path() -> PathBuf {
    let mut path = get_base_dir();
    path.push("clara.config.json");
    path
}

/// ディレクトリ構造と設定ファイルの初期化
pub fn init_workspace() -> std::io::Result<ClaraConfig> {
    let base_dir = get_base_dir();
    let atomics_dir = get_atomics_dir();
    let config_path = get_config_path();

    if !base_dir.exists() {
        fs::create_dir_all(&base_dir)?;
    }
    if !atomics_dir.exists() {
        fs::create_dir_all(&atomics_dir)?;
    }

    let config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let default_config = ClaraConfig::default();
        let json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, json)?;
        default_config
    };

    Ok(config)
}

/// CLIコマンドにプロンプトを標準入力で渡して実行し、結果を文字列で返す
pub fn execute_cli(prompt: &str, config: &ClaraConfig) -> Result<String, String> {
// ... existing execute_cli code remains but we'll insert our parser after it ...
// Actually, let's just insert the parser before `create_and_send_prompt`
    let mut child = Command::new(&config.cli_command)
        .args(&config.cli_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("CLIコマンドの起動に失敗しました: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("標準入力への書き込みに失敗しました: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("コマンドの実行待ちに失敗しました: {}", e))?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("CLIコマンドがエラーを返しました: {}", err))
    }
}

/// パースされたAIの回答
pub struct ParsedAiResponse {
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
}

/// AIの生の回答からタイトル、説明、タグ、本文を抽出する
pub fn parse_ai_response(raw_response: &str) -> ParsedAiResponse {
    let lines: Vec<&str> = raw_response.lines().collect();
    let mut title = "Untitled".to_string();
    let mut description: Option<String> = None;
    let mut tags: Vec<String> = vec![];
    let mut body = raw_response.to_string();

    let mut separator_index = None;

    // ヘッダー部分のパース（区切り線 "---" まで、もしくは最大10行程度を検索）
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            separator_index = Some(i);
            break;
        }

        if trimmed.starts_with("TITLE:") {
            title = trimmed.replace("TITLE:", "").trim().to_string();
        } else if trimmed.starts_with("DESC:") {
            let desc = trimmed.replace("DESC:", "").trim().to_string();
            if !desc.is_empty() {
                description = Some(desc);
            }
        } else if trimmed.starts_with("TAGS:") {
            let tags_str = trimmed.replace("TAGS:", "").trim().to_string();
            if !tags_str.is_empty() {
                tags = tags_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
        }
    }

    if let Some(idx) = separator_index {
        body = lines[idx + 1..].join("\n").trim().to_string();
    } else if title != "Untitled" {
        // 区切り線がないがタイトル等は見つかった場合（雑なフォールバック）
        // 最初の数行をスキップする
        let start = lines.iter().position(|l| !l.starts_with("TITLE:") && !l.starts_with("DESC:") && !l.starts_with("TAGS:") && !l.trim().is_empty()).unwrap_or(1);
        body = lines[start..].join("\n").trim().to_string();
    }

    ParsedAiResponse {
        title,
        description,
        tags,
        body,
    }
}

/// 新しいセットを作成し、Markdownとして保存する
#[tauri::command]
pub async fn create_and_send_prompt(
    description: Option<String>,
    prompt: String,
    parent_id: Option<String>,
) -> Result<ClaraSet, String> {
    let config = init_workspace().map_err(|e| e.to_string())?;

    // 1. IDの生成 (日時ベース: YYYYMMDDHHMMSS)
    let now = Utc::now();
    let id = now.format("%Y%m%d%H%M%S").to_string();

    // 2. プロンプトにメタデータ生成の指示を付与
    let system_instruction = "Please generate a title, a short description, and related tags for this request, then provide your answer.\nYou MUST format your output exactly as follows:\n\nTITLE: [Your generated title]\nDESC: [A short summary, or leave empty if not needed]\nTAGS: [comma-separated tags]\n---\n[Your actual response]\n\n";
    let full_prompt = format!("{}{}", system_instruction, prompt);

    // 3. CLIツールを実行してAIの回答を取得
    let raw_response = execute_cli(&full_prompt, &config)?;

    // 4. 回答からメタデータと本文をパース
    let parsed = parse_ai_response(&raw_response);

    // 5. Frontmatterの構築
    let frontmatter = ClaraFrontmatter {
        title: parsed.title,
        description: description.or(parsed.description), // ユーザー指定があれば優先
        id: id.clone(),
        parent_id,
        created_at: now,
        tags: parsed.tags,
    };

    // 6. Markdownファイルのフォーマット構築
    let yaml = serde_yaml::to_string(&frontmatter).map_err(|e| e.to_string())?;
    
    // Frontmatter + Userブロック + AIブロック (保存するのはユーザーの元のプロンプト)
    let markdown_content = format!(
        "---\n{}---\n\n~~~~~~user\n{}\n~~~~~~\n\n~~~~~~ai\n{}\n~~~~~~\n",
        yaml, prompt, parsed.body
    );

    // 7. ファイルの保存
    let mut file_path = get_atomics_dir();
    file_path.push(format!("{}.md", id));
    fs::write(&file_path, markdown_content).map_err(|e| e.to_string())?;

    // 8. 結果をフロントエンドに返す
    Ok(ClaraSet {
        frontmatter,
        prompt,
        response: parsed.body,
    })
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
        let base_dir = get_base_dir();
        assert!(base_dir.ends_with(".clara"));

        let atomics_dir = get_atomics_dir();
        assert!(atomics_dir.ends_with(".clara/atomics"));

        let config_path = get_config_path();
        assert!(config_path.ends_with(".clara/clara.config.json"));
    }

    #[test]
    fn test_parse_ai_response_with_format() {
        let raw = "TITLE: RustでのTauri開発\nDESC: Rustを使った開発について\nTAGS: rust, tauri, svelte\n---\nこれが回答です。\n2行目。";
        let parsed = parse_ai_response(raw);
        assert_eq!(parsed.title, "RustでのTauri開発");
        assert_eq!(parsed.description.unwrap(), "Rustを使った開発について");
        assert_eq!(parsed.tags, vec!["rust", "tauri", "svelte"]);
        assert_eq!(parsed.body, "これが回答です。\n2行目。");
    }

    #[test]
    fn test_parse_ai_response_without_separator() {
        let raw = "TITLE: タイトルのみで区切り線なし\nこれが回答です。";
        let parsed = parse_ai_response(raw);
        assert_eq!(parsed.title, "タイトルのみで区切り線なし");
        assert_eq!(parsed.body, "これが回答です。");
    }

    #[test]
    fn test_parse_ai_response_no_format() {
        let raw = "こんにちは！普通に回答を始めます。";
        let parsed = parse_ai_response(raw);
        assert_eq!(parsed.title, "Untitled");
        assert_eq!(parsed.body, "こんにちは！普通に回答を始めます。");
    }
}
