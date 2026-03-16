use crate::parser::SkrSearchResult;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// RRF定数 k。一般的に60が推奨される
const RRF_K: f64 = 60.0;

/// 各検索パス（ベクトル検索・キーワード検索）から取得する上位件数
const TOP_K_PER_SEARCH: usize = 50;

/// 最終的にフロントエンドに返す上位件数
const FINAL_TOP_K: usize = 20;

/// BM25パラメータ: 用語頻度の飽和係数
const BM25_K1: f64 = 1.2;

/// BM25パラメータ: 文書長の正規化係数
const BM25_B: f64 = 0.75;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    modified: u64,
    vector: Vec<f32>,
    title: String,
    snippet: String,
    /// キーワード検索に使用するテキスト本文
    #[serde(default)]
    body: String,
}

#[derive(Serialize, Deserialize, Default)]
struct EmbeddingsCache {
    entries: HashMap<String, CacheEntry>,
}

pub struct SearchState {
    model: Mutex<Option<TextEmbedding>>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            model: Mutex::new(None),
        }
    }

    fn init_model(&self) -> Result<(), String> {
        let mut model_guard = self.model.lock().map_err(|e| e.to_string())?;
        if model_guard.is_none() {
            let mut options = InitOptions::new(EmbeddingModel::BGEM3);
            options.show_download_progress = true;
            if let Some(mut cache_path) = dirs::home_dir() {
                cache_path.push(".clara");
                cache_path.push(".fastembed_cache");
                options.cache_dir = cache_path;
            }

            let model = TextEmbedding::try_new(options)
                .map_err(|e| format!("Failed to initialize embedding model: {}", e))?;
            *model_guard = Some(model);
        }
        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        atoms_dir: &Path,
        cache_path: &Path,
    ) -> Result<Vec<SkrSearchResult>, String> {
        self.init_model()?;

        let mut cache: EmbeddingsCache = if cache_path.exists() {
            let data = fs::read_to_string(cache_path)
                .map_err(|e| format!("Failed to read cache file: {}", e))?;
            if data.is_empty() {
                EmbeddingsCache::default()
            } else {
                serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to parse cache file: {}", e))?
            }
        } else {
            EmbeddingsCache::default()
        };

        let mut changed = false;
        let mut current_files = std::collections::HashSet::new();

        // ベクトル再生成が必要な更新（ファイル変更 or 新規）
        struct PendingEmbedUpdate {
            filename: String,
            modified: u64,
            title: String,
            snippet: String,
            text_to_embed: String,
            body: String,
        }
        // bodyフィールドのみの補完（既存ベクトルは保持、旧キャッシュ互換用）
        struct PendingBodyUpdate {
            filename: String,
            title: String,
            snippet: String,
            body: String,
        }
        let mut pending_embed_updates = Vec::new();
        let mut pending_body_updates = Vec::new();

        if !atoms_dir.exists() {
            return Ok(vec![]);
        }

        for entry in fs::read_dir(atoms_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    let filename = filename.to_string();
                    current_files.insert(filename.clone());

                    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                    let modified = metadata
                        .modified()
                        .map_err(|e| e.to_string())?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| e.to_string())?
                        .as_secs();

                    match cache.entries.get(&filename) {
                        Some(c) if c.modified >= modified && !c.body.is_empty() => {
                            // キャッシュは最新かつbodyあり → 更新不要
                        }
                        Some(c) if c.modified >= modified && c.body.is_empty() => {
                            // ベクトルは最新だがbodyが空 → body補完のみ（再embedding不要）
                            let content =
                                fs::read_to_string(&path).map_err(|e| e.to_string())?;
                            let extracted = extract_text_for_embedding(&content);
                            pending_body_updates.push(PendingBodyUpdate {
                                filename,
                                title: extracted.title,
                                snippet: extracted.snippet,
                                body: extracted.body,
                            });
                        }
                        _ => {
                            // 新規 or ファイル変更 → フルembedding更新
                            let content =
                                fs::read_to_string(&path).map_err(|e| e.to_string())?;
                            let extracted = extract_text_for_embedding(&content);
                            pending_embed_updates.push(PendingEmbedUpdate {
                                filename,
                                modified,
                                title: extracted.title,
                                snippet: extracted.snippet,
                                text_to_embed: extracted.text_to_embed,
                                body: extracted.body,
                            });
                        }
                    }
                }
            }
        }

        // body補完のみの更新（ベクトル再生成なし）
        for update in &pending_body_updates {
            if let Some(entry) = cache.entries.get_mut(&update.filename) {
                entry.title = update.title.clone();
                entry.snippet = update.snippet.clone();
                entry.body = update.body.clone();
            }
        }
        if !pending_body_updates.is_empty() {
            changed = true;
        }

        // フルembedding更新（ベクトル再生成 + body）
        if !pending_embed_updates.is_empty() {
            let mut model_guard = self.model.lock().map_err(|e| e.to_string())?;
            let model = model_guard.as_mut().unwrap();

            for chunk in pending_embed_updates.chunks(32) {
                let texts: Vec<String> = chunk.iter().map(|u| u.text_to_embed.clone()).collect();
                let embeddings = model.embed(texts, None).map_err(|e| e.to_string())?;

                for (update, vector) in chunk.iter().zip(embeddings) {
                    cache.entries.insert(
                        update.filename.clone(),
                        CacheEntry {
                            modified: update.modified,
                            vector,
                            title: update.title.clone(),
                            snippet: update.snippet.clone(),
                            body: update.body.clone(),
                        },
                    );
                }
            }
            changed = true;
        }

        // Remove deleted atoms
        let initial_len = cache.entries.len();
        cache.entries.retain(|key, _| current_files.contains(key));
        if cache.entries.len() != initial_len {
            changed = true;
        }

        if changed {
            if let Some(parent) = cache_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create cache directory: {}", e))?;
            }
            let file = std::fs::File::create(cache_path)
                .map_err(|e| format!("Failed to create cache file: {}", e))?;
            let writer = std::io::BufWriter::new(file);
            serde_json::to_writer(writer, &cache)
                .map_err(|e| format!("Failed to write to cache: {}", e))?;
        }

        if cache.entries.is_empty() {
            return Ok(vec![]);
        }

        // --- ベクトル検索パス ---
        let query_embedding = {
            let mut model_guard = self.model.lock().map_err(|e| e.to_string())?;
            let model = model_guard.as_mut().unwrap();
            let e5_query = format!("query: {}", query);
            let embeddings = model
                .embed(vec![e5_query], None)
                .map_err(|e| e.to_string())?;
            embeddings[0].clone()
        };

        let mut vector_results: Vec<(String, f32)> = cache
            .entries
            .iter()
            .map(|(id, entry)| {
                let score = cosine_similarity(&query_embedding, &entry.vector);
                (id.clone(), score)
            })
            .collect();
        vector_results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        vector_results.truncate(TOP_K_PER_SEARCH);

        // --- キーワード検索パス (BM25) ---
        // タイトルとbodyを結合した検索テキストへの参照マップを構築
        let search_texts: HashMap<&str, String> = cache
            .entries
            .iter()
            .map(|(id, entry)| {
                (id.as_str(), format!("{} {}", entry.title, entry.body))
            })
            .collect();
        let keyword_results = bm25_search(query, &search_texts, TOP_K_PER_SEARCH);

        // --- RRF (Reciprocal Rank Fusion) によるスコア統合 ---
        let merged = rrf_merge(&vector_results, &keyword_results, &cache, FINAL_TOP_K);

        Ok(merged)
    }
}

/// テキスト抽出結果
struct ExtractedText {
    title: String,
    snippet: String,
    text_to_embed: String,
    body: String,
}

fn extract_text_for_embedding(content: &str) -> ExtractedText {
    let mut title = String::from("Untitled");
    let mut text_lines = Vec::new();
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
        } else {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("~~~~~~") {
                text_lines.push(trimmed);
            }
        }
    }
    let body = text_lines.join(" ");
    let snippet = body.chars().take(150).collect::<String>();

    let embed_body = body.chars().take(1500).collect::<String>();
    let text_to_embed = format!("passage: {}\n{}", title, embed_body);
    ExtractedText {
        title,
        snippet,
        text_to_embed,
        body,
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a.sqrt() * norm_b.sqrt())
}

// =============================================================================
// BM25 キーワード検索
// =============================================================================

/// Unicode対応トークナイザ。
///
/// テキストを検索用トークンに分割する。言語によって分割戦略が異なる:
/// - **Latin/ASCII系**: ワード単位で分割（空白・句読点が区切り）
/// - **CJK系（漢字・ひらがな・カタカナ・ハングル）**: ユニグラム（1文字=1トークン）で分割
///
/// 形態素解析器（MeCab等）を使わない軽量実装のため、日本語の複合語はセマンティック検索
/// （ベクトル検索）側で補完される設計。キーワード検索は主に正確な用語一致を担当する。
fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();
    let mut current_word = String::new();

    for ch in lower.chars() {
        if is_cjk(ch) {
            // CJK文字の前に蓄積中のラテン語ワードがあればフラッシュ
            if !current_word.is_empty() {
                tokens.push(std::mem::take(&mut current_word));
            }
            // CJK文字はユニグラムとして個別トークン化
            tokens.push(ch.to_string());
        } else if ch.is_alphanumeric() || ch == '_' {
            current_word.push(ch);
        } else {
            // 空白・句読点等: ワード境界として処理
            if !current_word.is_empty() {
                tokens.push(std::mem::take(&mut current_word));
            }
        }
    }
    if !current_word.is_empty() {
        tokens.push(current_word);
    }
    tokens
}

/// CJK文字の判定。以下のUnicodeブロックをカバーする:
/// - CJK統合漢字 (U+4E00–U+9FFF)
/// - CJK統合漢字拡張A (U+3400–U+4DBF)
/// - ひらがな (U+3040–U+309F)
/// - カタカナ (U+30A0–U+30FF)
/// - CJK互換漢字 (U+F900–U+FAFF)
/// - ハングル音節 (U+AC00–U+D7AF)
///
/// これらの文字はスペース区切りを持たないため、ユニグラムトークン化の対象となる。
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'   // CJK統合漢字
        | '\u{3400}'..='\u{4DBF}' // CJK統合漢字拡張A
        | '\u{3040}'..='\u{309F}' // ひらがな
        | '\u{30A0}'..='\u{30FF}' // カタカナ
        | '\u{F900}'..='\u{FAFF}' // CJK互換漢字
        | '\u{AC00}'..='\u{D7AF}' // ハングル音節
    )
}

/// BM25アルゴリズムによるキーワード検索を実行する。
///
/// `documents`: ドキュメントID → テキスト本文のマッピング
/// `top_k`: 返却する上位件数
///
/// 戻り値: (ドキュメントID, BM25スコア) のベクトル（スコア降順）
fn bm25_search(
    query: &str,
    documents: &HashMap<&str, String>,
    top_k: usize,
) -> Vec<(String, f64)> {
    if documents.is_empty() {
        return vec![];
    }

    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        return vec![];
    }

    // 各文書をトークン化し、用語頻度（TF）を計算
    let mut doc_tokens: HashMap<&str, HashMap<String, usize>> = HashMap::new();
    let mut doc_lengths: HashMap<&str, usize> = HashMap::new();
    let mut total_length: usize = 0;

    for (id, text) in documents {
        let tokens = tokenize(text);
        let len = tokens.len();
        doc_lengths.insert(*id, len);
        total_length += len;

        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0) += 1;
        }
        doc_tokens.insert(*id, tf);
    }

    let num_docs = documents.len() as f64;
    let avg_doc_length = total_length as f64 / num_docs;

    // クエリトークンごとのIDF（逆文書頻度）を計算
    let mut idf_map: HashMap<&str, f64> = HashMap::new();
    for qt in &query_tokens {
        if idf_map.contains_key(qt.as_str()) {
            continue;
        }
        let docs_with_term = doc_tokens
            .values()
            .filter(|tf| tf.contains_key(qt.as_str()))
            .count() as f64;
        let idf = ((num_docs - docs_with_term + 0.5) / (docs_with_term + 0.5) + 1.0).ln();
        idf_map.insert(qt.as_str(), idf);
    }

    // 各文書のBM25スコアを計算
    let mut scores: Vec<(String, f64)> = documents
        .keys()
        .map(|id| {
            let tf_map = &doc_tokens[id];
            let doc_len = *doc_lengths.get(id).unwrap_or(&0) as f64;

            let score: f64 = query_tokens
                .iter()
                .map(|qt| {
                    let idf = idf_map.get(qt.as_str()).copied().unwrap_or(0.0);
                    let tf = tf_map.get(qt.as_str()).copied().unwrap_or(0) as f64;
                    idf * (tf * (BM25_K1 + 1.0))
                        / (tf + BM25_K1 * (1.0 - BM25_B + BM25_B * doc_len / avg_doc_length))
                })
                .sum();

            (id.to_string(), score)
        })
        .collect();

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores.truncate(top_k);

    // スコアが0以下の結果は除外
    scores.retain(|(_id, score)| *score > 0.0);
    scores
}

// =============================================================================
// RRF (Reciprocal Rank Fusion)
// =============================================================================

/// ベクトル検索結果とキーワード検索結果をRRFで統合する。
///
/// RRF_Score = 1/(k + Rank_Vector) + 1/(k + Rank_Keyword)
/// 片方にしか含まれないドキュメントは、もう片方のRankを無限大（スコア加算0）として扱う。
fn rrf_merge(
    vector_results: &[(String, f32)],
    keyword_results: &[(String, f64)],
    cache: &EmbeddingsCache,
    top_k: usize,
) -> Vec<SkrSearchResult> {
    let mut rrf_scores: HashMap<String, f64> = HashMap::new();

    // ベクトル検索結果のRRFスコア（Rank: 1-based）
    for (rank, (id, _)) in vector_results.iter().enumerate() {
        let rrf_component = 1.0 / (RRF_K + (rank as f64 + 1.0));
        *rrf_scores.entry(id.clone()).or_insert(0.0) += rrf_component;
    }

    // キーワード検索結果のRRFスコア（Rank: 1-based）
    for (rank, (id, _)) in keyword_results.iter().enumerate() {
        let rrf_component = 1.0 / (RRF_K + (rank as f64 + 1.0));
        *rrf_scores.entry(id.clone()).or_insert(0.0) += rrf_component;
    }

    // RRFスコア降順でソート
    let mut sorted: Vec<(String, f64)> = rrf_scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(top_k);

    // SkrSearchResult に変換
    // RRFスコアはf64で計算するが、表示用にf32へキャストする。
    // RRFスコアの値域は0〜0.033程度（最大 2/(k+1) ≈ 0.033）のためf32で十分な精度。
    sorted
        .into_iter()
        .filter_map(|(id, rrf_score)| {
            cache.entries.get(&id).map(|entry| SkrSearchResult {
                id,
                title: entry.title.clone(),
                score: rrf_score as f32,
                snippet: entry.snippet.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- tokenize ---

    #[test]
    fn test_tokenize_latin() {
        let tokens = tokenize("Hello World! How are you?");
        assert_eq!(tokens, vec!["hello", "world", "how", "are", "you"]);
    }

    #[test]
    fn test_tokenize_cjk() {
        let tokens = tokenize("Rust開発");
        assert_eq!(tokens, vec!["rust", "開", "発"]);
    }

    #[test]
    fn test_tokenize_hiragana_katakana() {
        let tokens = tokenize("こんにちはカタカナ");
        assert_eq!(
            tokens,
            vec!["こ", "ん", "に", "ち", "は", "カ", "タ", "カ", "ナ"]
        );
    }

    #[test]
    fn test_tokenize_mixed() {
        let tokens = tokenize("Tauri アプリ development 日本語");
        assert_eq!(
            tokens,
            vec!["tauri", "ア", "プ", "リ", "development", "日", "本", "語"]
        );
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_only_punctuation() {
        let tokens = tokenize("!@#$%^&*()");
        assert!(tokens.is_empty());
    }

    // --- is_cjk ---

    #[test]
    fn test_is_cjk() {
        assert!(is_cjk('漢'));
        assert!(is_cjk('あ'));
        assert!(is_cjk('カ'));
        assert!(!is_cjk('a'));
        assert!(!is_cjk('1'));
        assert!(!is_cjk(' '));
    }

    // --- BM25 ---

    #[test]
    fn test_bm25_search_basic() {
        let mut documents: HashMap<&str, String> = HashMap::new();
        documents.insert("doc1", "Rust programming language".to_string());
        documents.insert("doc2", "Python programming language".to_string());
        documents.insert("doc3", "Rust web development with Tauri".to_string());

        let results = bm25_search("Rust programming", &documents, 10);

        // doc1 should score highest (contains both "Rust" and "programming")
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "doc1");
    }

    #[test]
    fn test_bm25_search_japanese() {
        let mut documents: HashMap<&str, String> = HashMap::new();
        documents.insert("doc1", "Rustでの開発環境の構築".to_string());
        documents.insert("doc2", "Pythonでのデータ分析".to_string());
        documents.insert("doc3", "Rustのパフォーマンス最適化".to_string());

        let results = bm25_search("Rust開発", &documents, 10);

        // doc1 contains both "Rust" and "開" and "発"
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "doc1");
    }

    #[test]
    fn test_bm25_search_empty_query() {
        let mut documents: HashMap<&str, String> = HashMap::new();
        documents.insert("doc1", "Some text".to_string());
        let results = bm25_search("", &documents, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_search_empty_documents() {
        let documents: HashMap<&str, String> = HashMap::new();
        let results = bm25_search("query", &documents, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_search_no_match() {
        let mut documents: HashMap<&str, String> = HashMap::new();
        documents.insert("doc1", "Rust programming".to_string());
        let results = bm25_search("Python", &documents, 10);
        // "Python" doesn't appear in any document, so BM25 returns empty
        assert!(results.is_empty());
    }

    #[test]
    fn test_bm25_search_top_k() {
        // 文字列の所有権を保持するVec
        let keys: Vec<String> = (0..20).map(|i| format!("doc{}", i)).collect();
        let values: Vec<String> = (0..20).map(|i| format!("word{} common", i)).collect();
        let documents: HashMap<&str, String> = keys
            .iter()
            .zip(values.into_iter())
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        let results = bm25_search("common", &documents, 5);
        assert!(results.len() <= 5);
    }

    // --- RRF ---

    #[test]
    fn test_rrf_merge_both_lists() {
        let mut cache = EmbeddingsCache::default();
        cache.entries.insert(
            "doc1".to_string(),
            CacheEntry {
                modified: 0,
                vector: vec![],
                title: "Document 1".to_string(),
                snippet: "Snippet 1".to_string(),
                body: "Body 1".to_string(),
            },
        );
        cache.entries.insert(
            "doc2".to_string(),
            CacheEntry {
                modified: 0,
                vector: vec![],
                title: "Document 2".to_string(),
                snippet: "Snippet 2".to_string(),
                body: "Body 2".to_string(),
            },
        );
        cache.entries.insert(
            "doc3".to_string(),
            CacheEntry {
                modified: 0,
                vector: vec![],
                title: "Document 3".to_string(),
                snippet: "Snippet 3".to_string(),
                body: "Body 3".to_string(),
            },
        );

        // doc1: rank 1 in vector, rank 2 in keyword
        // doc2: rank 2 in vector, rank 1 in keyword
        // doc3: rank 3 in vector only
        let vector_results = vec![
            ("doc1".to_string(), 0.9f32),
            ("doc2".to_string(), 0.8f32),
            ("doc3".to_string(), 0.7f32),
        ];
        let keyword_results = vec![
            ("doc2".to_string(), 5.0f64),
            ("doc1".to_string(), 3.0f64),
        ];

        let results = rrf_merge(&vector_results, &keyword_results, &cache, 10);

        assert_eq!(results.len(), 3);

        // doc1: 1/(60+1) + 1/(60+2) = 0.01639 + 0.01613 = 0.03252
        // doc2: 1/(60+2) + 1/(60+1) = 0.01613 + 0.01639 = 0.03252
        // doc3: 1/(60+3) = 0.01587
        // doc1 and doc2 should be tied (same score, depends on sort stability)
        // doc3 should be last
        assert_eq!(results.last().unwrap().id, "doc3");

        // doc1 and doc2 should have significantly higher scores than doc3
        assert!(results[0].score > results[2].score);
    }

    #[test]
    fn test_rrf_merge_one_list_only() {
        let mut cache = EmbeddingsCache::default();
        cache.entries.insert(
            "doc1".to_string(),
            CacheEntry {
                modified: 0,
                vector: vec![],
                title: "Document 1".to_string(),
                snippet: "Snippet 1".to_string(),
                body: "Body 1".to_string(),
            },
        );

        let vector_results = vec![("doc1".to_string(), 0.9f32)];
        let keyword_results: Vec<(String, f64)> = vec![];

        let results = rrf_merge(&vector_results, &keyword_results, &cache, 10);

        assert_eq!(results.len(), 1);
        // RRF score = 1/(60+1) ≈ 0.01639
        assert!((results[0].score - 1.0 / 61.0).abs() < 1e-5);
    }

    #[test]
    fn test_rrf_merge_empty() {
        let cache = EmbeddingsCache::default();
        let results: Vec<SkrSearchResult> = rrf_merge(&[], &[], &cache, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_k_constant() {
        // Verify the RRF_K constant is 60 as per the specification
        assert_eq!(RRF_K, 60.0);
    }

    // --- cosine_similarity ---

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &a);
        assert!((similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let similarity = cosine_similarity(&a, &b);
        assert!(similarity.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![1.0, 2.0];
        let b = vec![0.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
    }

    // --- extract_text_for_embedding ---

    #[test]
    fn test_extract_text_for_embedding() {
        let content = r#"---
title: "Test Title"
id: "test-id"
---

~~~~~~user
Hello world
~~~~~~

~~~~~~ai
This is a response
~~~~~~"#;
        let result = extract_text_for_embedding(content);
        assert_eq!(result.title, "Test Title");
        assert!(!result.snippet.is_empty());
        assert!(!result.body.is_empty());
        assert!(result.text_to_embed.starts_with("passage: Test Title"));
        assert!(result.body.contains("Hello world"));
        assert!(result.body.contains("This is a response"));
    }
}
