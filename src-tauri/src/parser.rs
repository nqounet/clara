use serde::{Deserialize, Serialize};

/// Atomファイルのbody部分をパースし、(prompt, response) のタプルを返す
pub fn parse_atom_body(body: &str) -> (String, String) {
    let prompt = extract_block(body, "~~~~~~user\n", "\n~~~~~~");
    let response = extract_block(body, "~~~~~~ai\n", "\n~~~~~~");
    (
        prompt.unwrap_or_default(),
        response.unwrap_or_else(|| body.trim().to_string()),
    )
}

/// body内から開始マーカーと終了マーカーの間のテキストを抽出する
fn extract_block(body: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start = body.find(start_marker)?;
    let content_start = start + start_marker.len();
    let content_end = body[content_start..]
        .find(end_marker)
        .map(|i| content_start + i)
        .unwrap_or(body.len());
    Some(body[content_start..content_end].trim().to_string())
}

/// タイトルからファイル名に使える安全なASCIIスラッグを生成する
pub fn generate_slug(title: &str) -> String {
    let hyphenated: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    let slug = hyphenated
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    if slug.is_empty() {
        return "untitled".to_string();
    }

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

#[derive(Debug)]
pub struct ParsedAiResponse {
    pub title: String,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
}

pub fn parse_ai_response(raw_response: &str) -> ParsedAiResponse {
    let lines: Vec<&str> = raw_response.lines().collect();
    let mut title = "Untitled".to_string();
    let mut description: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut tags: Vec<String> = vec![];
    let mut body = raw_response.to_string();

    let mut separator_index = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            separator_index = Some(i);
            break;
        }

        if let Some(val) = trimmed.strip_prefix("TITLE:") {
            title = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("DESC:") {
            let desc = val.trim().to_string();
            if !desc.is_empty() {
                description = Some(desc);
            }
        } else if let Some(val) = trimmed.strip_prefix("SLUG:") {
            let s = val.trim().to_string();
            if !s.is_empty() {
                let sanitized = generate_slug(&s);
                if sanitized != "untitled" {
                    slug = Some(sanitized);
                }
            }
        } else if let Some(val) = trimmed.strip_prefix("TAGS:") {
            let tags_str = val.trim().to_string();
            if !tags_str.is_empty() {
                tags = tags_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    if let Some(idx) = separator_index {
        body = lines[idx + 1..].join("\n").trim().to_string();
    } else if title != "Untitled" {
        let start = lines
            .iter()
            .position(|l| {
                !l.starts_with("TITLE:")
                    && !l.starts_with("DESC:")
                    && !l.starts_with("SLUG:")
                    && !l.starts_with("TAGS:")
                    && !l.trim().is_empty()
            })
            .unwrap_or(1);
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SkrSearchResult {
    pub id: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
}

pub fn parse_skr_results(output: &str) -> Vec<SkrSearchResult> {
    output
        .trim()
        .split("\n\n")
        .filter_map(|block| {
            let mut id: Option<String> = None;
            let mut title: Option<String> = None;
            let mut score: Option<f32> = None;
            let mut snippet: Option<String> = None;

            for line in block.lines() {
                if let Some(val) = line.strip_prefix("ID:") {
                    id = Some(val.trim().to_string());
                } else if let Some(val) = line.strip_prefix("TITLE:") {
                    title = Some(val.trim().to_string());
                } else if let Some(val) = line.strip_prefix("SCORE:") {
                    score = val.trim().parse().ok();
                } else if let Some(val) = line.strip_prefix("SNIPPET:") {
                    snippet = Some(val.trim().to_string());
                } else if let Some(s) = snippet.as_mut() {
                    s.push('\n');
                    s.push_str(line);
                }
            }

            Some(SkrSearchResult {
                id: id?,
                title: title?,
                score: score?,
                snippet: snippet.unwrap_or_default(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("RustでTauriアプリを作る"), "rust-tauri");
        assert_eq!(generate_slug("Hello World!"), "hello-world");
        assert_eq!(generate_slug(""), "untitled");
        assert_eq!(generate_slug("foo  --  bar"), "foo-bar");
        let long = "a".repeat(40);
        assert!(generate_slug(&long).len() <= 30);
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
        let raw = "TITLE: Test\nSLUG: My Slug / Invalid!\n---\nbody";
        let parsed = parse_ai_response(raw);
        assert_eq!(parsed.slug.unwrap(), "my-slug-invalid");
    }

    #[test]
    fn test_parse_ai_response_slug_max_length() {
        let long_slug = "a".repeat(50);
        let raw = format!("TITLE: Test\nSLUG: {}\n---\nbody", long_slug);
        let parsed = parse_ai_response(&raw);
        assert!(parsed.slug.unwrap().len() <= 30);
    }

    #[test]
    fn test_parse_ai_response_slug_missing() {
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
    fn test_parse_skr_results() {
        let output = r#"
ID: 20240307120000-rust-setup
TITLE: Rust Setup Guide
SCORE: 0.95
SNIPPET: How to setup Rust environment with Tauri.

ID: 20240307130000-clara-intro
TITLE: Introduction to Clara
SCORE: 0.82
SNIPPET: Clara is a CLI-based knowledge management tool.
"#;
        let results = parse_skr_results(output);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "20240307120000-rust-setup");
        assert_eq!(results[0].title, "Rust Setup Guide");
        assert_eq!(results[0].score, 0.95);
        assert_eq!(
            results[0].snippet,
            "How to setup Rust environment with Tauri."
        );
        assert_eq!(results[1].id, "20240307130000-clara-intro");
        assert_eq!(results[1].title, "Introduction to Clara");
        assert_eq!(results[1].score, 0.82);
    }
}
