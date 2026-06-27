use crate::parser::SkrSearchResult;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct SearchState;

impl SearchState {
    pub fn new() -> Self {
        Self
    }

    pub fn search(&self, query: &str, atoms_dir: &Path) -> Result<Vec<SkrSearchResult>, String> {
        let tokens: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            return Ok(vec![]);
        }

        if !atoms_dir.exists() {
            return Ok(vec![]);
        }

        struct IntermediateResult {
            id: String,
            title: String,
            score: f32,
            snippet: String,
            modified: u64,
        }

        let mut matched_results = Vec::new();

        for entry in fs::read_dir(atoms_dir).map_err(|e| e.to_string())? {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let filename = match path.file_stem().and_then(|s| s.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let modified = match metadata.modified() {
                Ok(mod_time) => match mod_time.duration_since(std::time::UNIX_EPOCH) {
                    Ok(dur) => dur.as_secs(),
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let content_lower = content.to_lowercase();

            // AND検索: すべてのトークンが含まれているか
            let is_match = tokens.iter().all(|token| content_lower.contains(token));
            if !is_match {
                continue;
            }

            // タイトル、desc、本文行の抽出
            let (title, desc, body_lines) = extract_title_and_body(&content);

            // スコア（出現回数）の計算
            let mut score = 0.0f32;
            for token in &tokens {
                let mut count = 0;
                let mut start = 0;
                while let Some(pos) = content_lower[start..].find(token) {
                    count += 1;
                    start += pos + token.len();
                }
                score += count as f32;
            }

            // スニペットの作成
            let mut snippet = String::new();
            for line in &body_lines {
                let line_lower = line.to_lowercase();
                if tokens.iter().any(|t| line_lower.contains(t)) {
                    snippet = line.trim().to_string();
                    break;
                }
            }
            if snippet.is_empty() && !desc.is_empty() {
                snippet = desc;
            }
            if snippet.is_empty() {
                let full_body = body_lines.join(" ");
                snippet = full_body.chars().take(150).collect::<String>();
            }

            matched_results.push(IntermediateResult {
                id: filename,
                title,
                score,
                snippet,
                modified,
            });
        }

        // スコアの降順、スコアが同じなら更新日時の降順でソート
        matched_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.modified.cmp(&a.modified))
        });

        let final_results = matched_results
            .into_iter()
            .map(|r| SkrSearchResult {
                id: r.id,
                title: r.title,
                score: r.score,
                snippet: r.snippet,
            })
            .collect();

        Ok(final_results)
    }
}

fn extract_title_and_body(content: &str) -> (String, String, Vec<String>) {
    let mut title = String::from("Untitled");
    let mut desc = String::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = false;

    for line in content.lines() {
        if line.starts_with("---") {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            if let Some(t) = line.strip_prefix("title:") {
                title = t.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            }
            if let Some(t) = line.strip_prefix("TITLE:") {
                title = t.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            }
            if let Some(d) = line.strip_prefix("desc:") {
                desc = d.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            }
            if let Some(d) = line.strip_prefix("DESC:") {
                desc = d.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            }
        } else {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("~~~~~~") {
                body_lines.push(line.to_string());
            }
        }
    }
    (title, desc, body_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path =
                std::env::temp_dir().join(format!("clara_test_{}_{}", std::process::id(), counter));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn write_file(&self, filename: &str, content: &str) {
            let file_path = self.path.join(filename);
            let mut file = File::create(file_path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_grep_search_basic() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-rust.md",
            "---\ntitle: Rust Guide\n---\nThis is a guide about Rust programming.",
        );
        dir.write_file(
            "20260627-python.md",
            "---\ntitle: Python Data\n---\nData analysis with Python.",
        );

        let searcher = SearchState::new();
        let results = searcher.search("Rust", &dir.path).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "20260627-rust");
        assert_eq!(results[0].title, "Rust Guide");
    }

    #[test]
    fn test_grep_search_case_insensitive() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-rust.md",
            "---\ntitle: Rust Guide\n---\nThis is a guide about Rust programming.",
        );

        let searcher = SearchState::new();
        // 小文字の "rust" でも大文字 of "Rust" にマッチすることを確認
        let results = searcher.search("rust", &dir.path).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "20260627-rust");
    }

    #[test]
    fn test_grep_search_and_logic() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-tauri.md",
            "---\ntitle: Tauri App\n---\nRust Tauri application building.",
        );
        dir.write_file(
            "20260627-rust.md",
            "---\ntitle: Rust Guide\n---\nThis is a guide about Rust programming without GUI.",
        );

        let searcher = SearchState::new();
        // "Rust" と "Tauri" の両方が含まれるものだけがマッチする（AND検索）
        let results = searcher.search("Rust Tauri", &dir.path).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "20260627-tauri");
    }

    #[test]
    fn test_grep_search_score() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-many.md",
            "---\ntitle: Many Rust\n---\nRust. Yes, Rust is awesome. Rust is fast.",
        );
        dir.write_file(
            "20260627-one.md",
            "---\ntitle: One Rust\n---\nThis contains Rust once.",
        );

        let searcher = SearchState::new();
        let results = searcher.search("Rust", &dir.path).unwrap();

        assert_eq!(results.len(), 2);
        // 出現回数が多い "20260627-many" が先頭に来ることを確認
        assert_eq!(results[0].id, "20260627-many");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_grep_search_snippet() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-snippet.md",
            "---\ntitle: Snippet Test\n---\nLine 1: Hello\nLine 2: Target keyword here\nLine 3: World",
        );

        let searcher = SearchState::new();
        let results = searcher.search("keyword", &dir.path).unwrap();

        assert_eq!(results.len(), 1);
        // マッチした行がスニペットとして返ることを確認
        assert!(results[0].snippet.contains("Line 2: Target keyword here"));
    }

    #[test]
    fn test_grep_search_empty_query() {
        let dir = TestDir::new();
        dir.write_file(
            "20260627-rust.md",
            "---\ntitle: Rust Guide\n---\nThis is a guide about Rust programming.",
        );

        let searcher = SearchState::new();
        let results = searcher.search("", &dir.path).unwrap();

        assert!(results.is_empty());
    }
}
