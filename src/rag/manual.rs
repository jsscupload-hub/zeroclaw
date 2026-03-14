//! RAG pipeline for home appliance manuals.
//!
//! Loads .md and .txt files from a manuals directory and retrieves relevant
//! chunks based on user queries.

use crate::memory::chunker;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A chunk of manual content.
#[derive(Debug, Clone)]
pub struct ManualChunk {
    /// Source file path.
    pub source: String,
    /// Chunk content.
    pub content: String,
}

/// Manual RAG index.
pub struct ManualRag {
    chunks: Vec<ManualChunk>,
    /// Keywords extracted from filenames for quick relevance checking.
    filenames: HashSet<String>,
}

impl ManualRag {
    /// Load manuals from a directory.
    pub fn load(workspace_dir: &Path, manuals_dir: &str) -> anyhow::Result<Self> {
        let base = workspace_dir.join(manuals_dir);
        if !base.exists() || !base.is_dir() {
            return Ok(Self {
                chunks: Vec::new(),
                filenames: HashSet::new(),
            });
        }

        let mut paths: Vec<PathBuf> = Vec::new();
        Self::collect_paths(&base, &mut paths);

        let mut chunks = Vec::new();
        let mut filenames = HashSet::new();
        let max_tokens = 512;

        for path in paths {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            if content.trim().is_empty() {
                continue;
            }

            let source = path
                .strip_prefix(workspace_dir)
                .unwrap_or(&path)
                .display()
                .to_string();

            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let stem_lower = stem.to_lowercase();
                // Add the full stem as a keyword
                filenames.insert(stem_lower.clone());
                // Also add individual words from the stem
                for word in stem_lower.split(|c: char| !c.is_alphanumeric()) {
                    if word.len() > 1 {
                        filenames.insert(word.to_string());
                    }
                }
            }

            for chunk in chunker::chunk_markdown(&content, max_tokens) {
                chunks.push(ManualChunk {
                    source: source.clone(),
                    content: chunk.content,
                });
            }
        }

        Ok(Self { chunks, filenames })
    }

    fn collect_paths(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_paths(&path, out);
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str());
                if ext == Some("md") || ext == Some("txt") {
                    out.push(path);
                }
            }
        }
    }

    /// Retrieve chunks relevant to the query.
    pub fn retrieve(&self, query: &str, limit: usize) -> Vec<&ManualChunk> {
        if self.chunks.is_empty() || limit == 0 {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();

        let mut scored: Vec<(&ManualChunk, f32)> = Vec::new();
        for chunk in &self.chunks {
            let content_lower = chunk.content.to_lowercase();
            let mut score = 0.0f32;

            for term in &query_terms {
                if content_lower.contains(term) {
                    score += 1.0;
                }
            }

            if score > 0.0 {
                scored.push((chunk, score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored.into_iter().map(|(c, _)| c).collect()
    }

    /// Check if the query is likely about an appliance based on keywords.
    pub fn is_relevant(&self, query: &str, extra_keywords: &[String]) -> bool {
        let query_lower = query.to_lowercase();
        
        // Match against filenames
        for kw in &self.filenames {
            if query_lower.contains(kw) {
                return true;
            }
        }

        // Match against user-provided keywords
        for kw in extra_keywords {
            if query_lower.contains(&kw.to_lowercase()) {
                return true;
            }
        }

        false
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}
