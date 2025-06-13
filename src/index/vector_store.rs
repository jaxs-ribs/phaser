use std::error::Error;
use std::path::Path;
use std::fs;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::index::chunker::CodeChunk;
use crate::index::embedder::Embedding;

#[derive(Debug, Serialize, Deserialize)]
struct StoredChunk {
    id: String,
    chunk: CodeChunk,
    embedding: Embedding,
}

#[derive(Debug)]
pub struct VectorDB {
    db_path: String,
    chunks: Vec<StoredChunk>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub chunk: CodeChunk,
    pub score: f32,
    pub embedding: Embedding,
}

impl VectorDB {
    pub async fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut db = VectorDB {
            db_path: db_path.to_string(),
            chunks: Vec::new(),
        };
        
        // Load existing data if file exists
        db.load_from_disk().await?;
        
        Ok(db)
    }

    async fn load_from_disk(&mut self) -> Result<(), Box<dyn Error>> {
        let file_path = format!("{}/chunks.json", self.db_path);
        
        if Path::new(&file_path).exists() {
            let content = fs::read_to_string(&file_path)?;
            if !content.trim().is_empty() {
                self.chunks = serde_json::from_str(&content)?;
                println!("📊 Loaded {} chunks from vector database", self.chunks.len());
            }
        }
        
        Ok(())
    }

    async fn save_to_disk(&self) -> Result<(), Box<dyn Error>> {
        let file_path = format!("{}/chunks.json", self.db_path);
        
        // Ensure directory exists
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&self.chunks)?;
        fs::write(&file_path, content)?;
        
        Ok(())
    }

    pub async fn add_chunks(&mut self, chunks: &[CodeChunk], embeddings: &[Embedding]) -> Result<(), Box<dyn Error>> {
        if chunks.len() != embeddings.len() {
            return Err("Number of chunks and embeddings must match".into());
        }

        if chunks.is_empty() {
            return Ok(());
        }

        // Add chunks to in-memory storage
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            let stored_chunk = StoredChunk {
                id: Uuid::new_v4().to_string(),
                chunk: chunk.clone(),
                embedding: embedding.clone(),
            };
            self.chunks.push(stored_chunk);
        }

        // Persist to disk
        self.save_to_disk().await?;

        Ok(())
    }

    pub async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        if self.chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate similarity scores for all chunks
        let mut scored_chunks: Vec<(f32, &StoredChunk)> = self.chunks
            .iter()
            .map(|stored_chunk| {
                let similarity = self.cosine_similarity(query_embedding, &stored_chunk.embedding);
                (similarity, stored_chunk)
            })
            .collect();

        // Sort by similarity score (descending)
        scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k results
        let results: Vec<SearchResult> = scored_chunks
            .into_iter()
            .take(top_k)
            .map(|(score, stored_chunk)| SearchResult {
                id: stored_chunk.id.clone(),
                chunk: stored_chunk.chunk.clone(),
                score,
                embedding: stored_chunk.embedding.clone(),
            })
            .collect();

        Ok(results)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    pub async fn count(&self) -> Result<usize, Box<dyn Error>> {
        Ok(self.chunks.len())
    }

    pub async fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        self.chunks.clear();
        self.save_to_disk().await?;
        Ok(())
    }
}