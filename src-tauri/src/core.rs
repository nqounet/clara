use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::models::{ClaraFrontmatter, ClaraAtom};

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

pub fn load_app_config() -> std::io::Result<AppConfig> {
    let config_path = get_global_settings_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
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

/// 現在のAppConfig（グローバル設定）を取得する
#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    load_app_config().map_err(|e| format!("設定の読み込みに失敗しました: {}", e))
}

/// Rootディレクトリ（Vaultの場所）を更新する
#[tauri::command]
pub fn update_root_dir(new_path: String) -> Result<AppConfig, String> {
    let trimmed = new_path.trim();
    if trimmed.is_empty() {
        return Err("Vaultのパスを指定してください。".into());
    }

    let mut config = load_app_config().map_err(|e| format!("設定の読み込みに失敗: {}", e))?;
    
    // 新しいパスの設定
    let path_buf = PathBuf::from(trimmed);
    config.root_dir = path_buf.clone();

    // ディレクトリが存在しなければ作成
    if !path_buf.exists() {
        fs::create_dir_all(&path_buf).map_err(|e| format!("ディレクトリの作成に失敗: {}", e))?;
    }

    // 設定ファイルに保存
    let config_path = get_global_settings_path();
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| format!("設定の保存に失敗: {}", e))?;

    Ok(config)
}

/// 最近のAtomのリストを取得する（Frontmatterのみ）
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
                // Frontmatter（最初の "---" と次の "---" の間）を抽出してパース
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

    // ID（日付ベース）の降順でソート
    frontmatters.sort_by(|a, b| b.id.cmp(&a.id));
    frontmatters.truncate(limit);

    Ok(frontmatters)
}

/// IDから特定のAtomを読み込む
#[tauri::command]
pub fn load_atom(id: String) -> Result<ClaraAtom, String> {
    let app_config = load_app_config().map_err(|e| e.to_string())?;
    let atoms_dir = get_atoms_dir(&app_config.root_dir);

    // IDから始まるファイルを検索
    let entries = fs::read_dir(atoms_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with(&id) && file_name.ends_with(".md") {
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                
                // パース処理 (Frontmatter, User, AI)
                let parts: Vec<&str> = content.splitn(3, "---").collect();
                if parts.len() < 3 {
                    return Err("ファイルフォーマットが不正です".into());
                }
                
                let frontmatter: ClaraFrontmatter = serde_yaml::from_str(parts[1].trim()).map_err(|e| e.to_string())?;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaraConfig {
    pub cli_command: String,
    pub cli_args: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
}

impl Default for ClaraConfig {
    fn default() -> Self {
        Self {
            cli_command: "gemini".to_string(),
            cli_args: vec![],
            model: None,
            working_dir: None,
        }
    }
}

/// 現在のClaraConfig（ワークスペース設定）を取得する
#[tauri::command]
pub fn get_clara_config() -> Result<ClaraConfig, String> {
    let (_, clara_config) = init_workspace().map_err(|e| e.to_string())?;
    Ok(clara_config)
}

/// ClaraConfig（ワークスペース設定）を更新する
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
    clara_config.working_dir = working_dir.filter(|s| !s.trim().is_empty()).map(PathBuf::from);

    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(&clara_config).map_err(|e| e.to_string())?;
    fs::write(&config_path, json).map_err(|e| format!("設定の保存に失敗: {}", e))?;

    Ok(clara_config)
}

/// Atomファイルのbody部分をパースし、(prompt, response) のタプルを返す
fn parse_atom_body(body: &str) -> (String, String) {
    let prompt = extract_block(body, "~~~~~~user\n", "\n~~~~~~");
    let response = extract_block(body, "~~~~~~ai\n", "\n~~~~~~");
    (
        prompt.unwrap_or_default(),
        response.unwrap_or_else(|| body.trim().to_string()),
    )
}

/// body内から開始マーカーと終了マーカーの間のテキストを抽出する
/// 注: Atomファイルの構造上、~~~~~~user ブロックが先、~~~~~~ai ブロックが後の順序を前提とする
fn extract_block(body: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start = body.find(start_marker)?;
    let content_start = start + start_marker.len();
    let content_end = body[content_start..]
        .find(end_marker)
        .map(|i| content_start + i)
        .unwrap_or(body.len());
    Some(body[content_start..content_end].trim().to_string())
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
pub fn init_workspace() -> std::io::Result<(AppConfig, ClaraConfig)> {
    let app_config = load_app_config()?;
    let base_dir = &app_config.root_dir;
    // Clara設定ファイルは Vault とは独立して ~/.clara/clara.config.json に保存
    let config_path = get_config_path();

    if !base_dir.exists() {
        fs::create_dir_all(base_dir)?;
    }

    // 設定ファイルの親ディレクトリ（~/.clara）を確保
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let clara_config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let default_config = ClaraConfig::default();
        let json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, json)?;
        default_config
    };

    Ok((app_config, clara_config))
}

/// CLIコマンドにプロンプトを標準入力で渡して実行し、結果を文字列で返す
pub fn execute_cli(prompt: &str, config: &ClaraConfig, yolo: bool) -> Result<String, String> {
    let mut cmd = Command::new(&config.cli_command);
    cmd.args(&config.cli_args);

    if yolo {
        cmd.arg("--yolo");
    }

    if let Some(ref m) = config.model {
        cmd.arg("--model");
        cmd.arg(m);
    }

    if let Some(ref wd) = config.working_dir {
        if wd.exists() {
            cmd.current_dir(wd);
        } else {
            return Err(format!(
                "Workspaceディレクトリが存在しません: {}\n設定を確認してください。",
                wd.display()
            ));
        }
    }

    let mut child = cmd
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

/// タイトルからファイル名に使える安全なASCIIスラッグを生成する
///
/// - ASCII英数字以外はすべてハイフンに変換
/// - 連続ハイフンを1つに圧縮
/// - 先頭・末尾のハイフンを除去
/// - 最大30文字に切り詰め
fn generate_slug(title: &str) -> String {
    // 非ASCII英数字をすべてハイフンに変換（日本語等のUnicode文字を含む）
    let hyphenated: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    // ハイフンで分割→空要素を除去（連続ハイフン・先頭末尾処理）→再結合
    let slug = hyphenated
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    if slug.is_empty() {
        return "untitled".to_string();
    }

    // 最大30文字に切り詰め（末尾のハイフンも除去）
    let max_len = 30;
    if slug.len() > max_len {
        slug.chars()
            .take(max_len)
            .collect::<String>()
            .trim_end_matches('-')
            .to_string()
    } else {
        slug
    }
}

/// パースされたAIの回答
#[derive(Debug)]
pub struct ParsedAiResponse {
    pub title: String,
    pub description: Option<String>,
    /// LLMが生成したURLセーフなslug（英数字・ハイフンのみ）
    pub slug: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
}

/// AIの生の回答からタイトル、説明、slug、タグ、本文を抽出する
pub fn parse_ai_response(raw_response: &str) -> ParsedAiResponse {
    let lines: Vec<&str> = raw_response.lines().collect();
    let mut title = "Untitled".to_string();
    let mut description: Option<String> = None;
    let mut slug: Option<String> = None;
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
        } else if trimmed.starts_with("SLUG:") {
            let s = trimmed.replace("SLUG:", "").trim().to_string();
            if !s.is_empty() {
                let sanitized = generate_slug(&s);
                if sanitized != "untitled" {
                    slug = Some(sanitized);
                }
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
        let start = lines.iter().position(|l| !l.starts_with("TITLE:") && !l.starts_with("DESC:") && !l.starts_with("SLUG:") && !l.starts_with("TAGS:") && !l.trim().is_empty()).unwrap_or(1);
        body = lines[start..].join("\n").trim().to_string();
    }

    ParsedAiResponse {
        title,
        description,
        slug,
        tags,
        body,
    }
}

/// 新しいAtomを作成し、Markdownとして保存する
#[tauri::command]
pub async fn create_and_send_prompt(
    description: Option<String>,
    prompt: String,
    parent_id: Option<String>,
    yolo: bool,
) -> Result<ClaraAtom, String> {
    let (app_config, config) = init_workspace().map_err(|e| e.to_string())?;

    // 1. IDの生成 (日時ベース: YYYYMMDDHHMMSS)
    let now = Utc::now();
    let id = now.format("%Y%m%d%H%M%S").to_string();

    // 2. プロンプトにメタデータ生成の指示を付与
    let system_instruction = "Please generate a title, a short description, a URL-safe slug, and related tags for this request, then provide your answer.\nYou MUST format your output exactly as follows:\n\nTITLE: [Your generated title]\nDESC: [A short summary, or leave empty if not needed]\nSLUG: [lowercase ASCII slug using hyphens only, max 30 chars, e.g. rust-tauri-setup]\nTAGS: [comma-separated tags]\n---\n[Your actual response]\n\n";
    let full_prompt = format!("{}{}", system_instruction, prompt);

    // 3. CLIツールを実行してAIの回答を取得
    let raw_response = execute_cli(&full_prompt, &config, yolo)?;

    // 4. 回答からメタデータと本文をパース
    let parsed = parse_ai_response(&raw_response);

    // 5. Slugの决定: LLMが返したslugを優先し、なければタイトルから生成
    let slug = parsed.slug.unwrap_or_else(|| generate_slug(&parsed.title));
    let full_id = format!("{}-{}", id, slug);

    // 6. Frontmatterの構築
    let frontmatter = ClaraFrontmatter {
        title: parsed.title.clone(),
        description: description.or(parsed.description), // ユーザー指定があれば優先
        id: full_id.clone(),
        parent_id: parent_id.clone(),
        created_at: now,
        tags: parsed.tags,
        cli_command: Some(config.cli_command.clone()),
        model: config.model.clone(),
        workspace: config.working_dir.as_ref().map(|p| p.to_string_lossy().into_owned()),
        yolo,
    };

    // 7. Markdownファイルのフォーマット構築
    let yaml = serde_yaml::to_string(&frontmatter).map_err(|e| e.to_string())?;
    
    // Obsidian用の親リンク（コードフェンスの外に配置する必要がある）
    let parent_link_block = if let Some(ref pid) = parent_id {
        format!("\n\n---\n**Parent:** [[{}]]\n", pid)
    } else {
        String::new()
    };

    // Frontmatter + Userブロック + AIブロック + 親リンク
    let markdown_content = format!(
        "---\n{}---\n\n~~~~~~user\n{}\n~~~~~~\n\n~~~~~~ai\n{}\n~~~~~~{}",
        yaml, prompt, parsed.body, parent_link_block
    );

    // 8. ファイルの保存
    let mut file_path = get_atoms_dir(&app_config.root_dir);
    file_path.push(format!("{}.md", full_id));
    fs::write(&file_path, markdown_content).map_err(|e| e.to_string())?;

    // 9. 結果をフロントエンドに返す
    Ok(ClaraAtom {
        frontmatter,
        prompt,
        response: parsed.body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        // 日本語などの非ASCII文字はハイフンに変換され圧縮される
        assert_eq!(generate_slug("RustでTauriアプリを作る"), "rust-tauri");
        // 末尾の記号起因ハイフンは除去される
        assert_eq!(generate_slug("Hello World!"), "hello-world");
        // 空文字列はフォールバック
        assert_eq!(generate_slug(""), "untitled");
        // 連続する区切り文字は1ハイフンに圧縮される
        assert_eq!(generate_slug("foo  --  bar"), "foo-bar");
        // 30文字を超える場合は切り詰め
        let long = "a".repeat(40);
        assert!(generate_slug(&long).len() <= 30);
    }

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
    fn test_parse_ai_response_with_format() {
        let raw = "TITLE: RustでのTauri開発\nDESC: Rustを使った開発について\nSLUG: rust-tauri-dev\nTAGS: rust, tauri, svelte\n---\nこれが回答です。\n2行目。";
        let parsed = parse_ai_response(raw);
        assert_eq!(parsed.title, "RustでのTauri開発");
        assert_eq!(parsed.description.unwrap(), "Rustを使った開発について");
        assert_eq!(parsed.slug.unwrap(), "rust-tauri-dev");
        assert_eq!(parsed.tags, vec!["rust", "tauri", "svelte"]);
        assert_eq!(parsed.body, "これが回答です。\n2行目。");
    }

    #[test]
    fn test_parse_ai_response_slug_sanitize() {
        // LLMが不正な文字を含むslugを返した場合は無害化される
        let raw = "TITLE: Test\nSLUG: My Slug / Invalid!\n---\nbody";
        let parsed = parse_ai_response(raw);
        // スペース・記号はハイフンに変換され、連続ハイフンは圧縮される
        assert_eq!(parsed.slug.unwrap(), "my-slug-invalid");
    }

    #[test]
    fn test_parse_ai_response_slug_max_length() {
        // LLMが長すぎるslugを返した場合は30文字以内に切り詰められる
        let long_slug = "a".repeat(50);
        let raw = format!("TITLE: Test\nSLUG: {}\n---\nbody", long_slug);
        let parsed = parse_ai_response(&raw);
        assert!(parsed.slug.unwrap().len() <= 30);
    }

    #[test]
    fn test_parse_ai_response_slug_missing() {
        // SLUGがない場合はNone
        let raw = "TITLE: RustでのTauri開発\nTAGS: rust\n---\nbody";
        let parsed = parse_ai_response(raw);
        assert!(parsed.slug.is_none());
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

    #[test]
    fn test_parse_atom_body_normal() {
        let body = "\n\n~~~~~~user\nテストプロンプト\n~~~~~~\n\n~~~~~~ai\nAIの回答\n~~~~~~\n";
        let (prompt, response) = parse_atom_body(body);
        assert_eq!(prompt, "テストプロンプト");
        assert_eq!(response, "AIの回答");
    }

    #[test]
    fn test_parse_atom_body_no_user_block() {
        let body = "\n\n~~~~~~ai\nAIのみ\n~~~~~~\n";
        let (prompt, response) = parse_atom_body(body);
        assert_eq!(prompt, "");
        assert_eq!(response, "AIのみ");
    }

    #[test]
    fn test_parse_atom_body_no_blocks() {
        let body = "\nただのテキスト\n";
        let (prompt, response) = parse_atom_body(body);
        assert_eq!(prompt, "");
        assert_eq!(response, "ただのテキスト");
    }

    #[test]
    fn test_extract_block() {
        let body = "pre\n~~~~~~user\nhello world\n~~~~~~\npost";
        let result = extract_block(body, "~~~~~~user\n", "\n~~~~~~");
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_extract_block_missing() {
        let body = "no markers here";
        let result = extract_block(body, "~~~~~~user\n", "\n~~~~~~");
        assert!(result.is_none());
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
        assert!(path.to_string_lossy().contains("clara") || path.to_string_lossy().contains("Clara"));
        assert!(path.ends_with("settings.json"));
    }

    #[test]
    fn test_frontmatter_yolo_serialization() {
        use crate::models::{is_false, ClaraFrontmatter};

        // is_false ヘルパーの基本動作テスト
        assert!(is_false(&false));
        assert!(!is_false(&true));

        // yolo=false のとき、YAMLに "yolo" が含まれないこと
        let fm = ClaraFrontmatter {
            title: "Test".into(),
            description: None,
            id: "test-id".into(),
            parent_id: None,
            created_at: Utc::now(),
            tags: vec![],
            cli_command: None,
            model: None,
            workspace: None,
            yolo: false,
        };
        let yaml = serde_yaml::to_string(&fm).unwrap();
        assert!(!yaml.contains("yolo"), "yolo=false のときは YAML に含まれるべきではない");

        // yolo=true のとき、YAMLに "yolo: true" が含まれること
        let fm_yolo = ClaraFrontmatter {
            yolo: true,
            ..fm
        };
        let yaml_yolo = serde_yaml::to_string(&fm_yolo).unwrap();
        assert!(yaml_yolo.contains("yolo: true"), "yolo=true のときは YAML に出力されるべき");
    }
}
