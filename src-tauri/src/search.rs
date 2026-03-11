use crate::parser::SkrSearchResult;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    modified: u64,
    vector: Vec<f32>,
    title: String,
    snippet: String,
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
            let mut options = InitOptions::new(EmbeddingModel::MultilingualE5Small);
            options.show_download_progress = true;

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
            let data = fs::read_to_string(cache_path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            EmbeddingsCache::default()
        };

        let mut changed = false;
        let mut current_files = std::collections::HashSet::new();

        struct PendingUpdate {
            filename: String,
            modified: u64,
            title: String,
            snippet: String,
            text_to_embed: String,
        }
        let mut pending_updates = Vec::new();

        // Ensure atoms directory exists
        if !atoms_dir.exists() {
            return Ok(vec![]);
        }

        // Check atoms
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

                    let needs_update = cache
                        .entries
                        .get(&filename)
                        .map(|c| c.modified < modified)
                        .unwrap_or(true);

                    if needs_update {
                        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                        let (title, snippet, text_to_embed) = extract_text_for_embedding(&content);
                        pending_updates.push(PendingUpdate {
                            filename,
                            modified,
                            title,
                            snippet,
                            text_to_embed,
                        });
                    }
                }
            }
        }

        // Process pending updates in batches
        if !pending_updates.is_empty() {
            let mut model_guard = self.model.lock().map_err(|e| e.to_string())?;
            let model = model_guard.as_mut().unwrap();

            // fastembed handles batching internally, but we can pass all strings at once
            // Or we can explicitly chunk it to avoid out-of-memory if there are thousands of files.
            for chunk in pending_updates.chunks(32) {
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
                        },
                    );
                }
            }
            changed = true;
        }

        // Remove deleted atoms
        let keys: Vec<String> = cache.entries.keys().cloned().collect();
        for key in keys {
            if !current_files.contains(&key) {
                cache.entries.remove(&key);
                changed = true;
            }
        }

        if changed {
            if let Some(parent) = cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(file) = std::fs::File::create(cache_path) {
                let writer = std::io::BufWriter::new(file);
                let _ = serde_json::to_writer(writer, &cache);
            }
        }

        if cache.entries.is_empty() {
            return Ok(vec![]);
        }

        // Embed query
        let query_embedding = {
            let mut model_guard = self.model.lock().map_err(|e| e.to_string())?;
            let model = model_guard.as_mut().unwrap();
            let e5_query = format!("query: {}", query);
            let embeddings = model
                .embed(vec![e5_query], None)
                .map_err(|e| e.to_string())?;
            embeddings[0].clone()
        };

        // Compute cosine similarities
        let mut results = Vec::new();
        for (id, entry) in cache.entries.iter() {
            let score = cosine_similarity(&query_embedding, &entry.vector);
            results.push(SkrSearchResult {
                id: id.clone(),
                title: entry.title.clone(),
                score,
                snippet: entry.snippet.clone(),
            });
        }

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(10);

        Ok(results)
    }
}

fn extract_text_for_embedding(content: &str) -> (String, String, String) {
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
            // For YAML with uppercase format used in CLARA
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

    // Prefix `passage: ` for e5 model indexing
    let embed_body = body.chars().take(1500).collect::<String>();
    let text_to_embed = format!("passage: {}\n{}", title, embed_body);
    (title, snippet, text_to_embed)
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
