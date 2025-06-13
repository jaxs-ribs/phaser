use sha2::{Sha256, Digest};
use std::error::Error;

pub type Embedding = Vec<f32>;

#[derive(Debug)]
pub struct EmbeddingGenerator {
    dimension: usize,
}

impl EmbeddingGenerator {
    pub fn new(dimension: usize) -> Result<Self, Box<dyn Error>> {
        // Simplified implementation for demonstration
        // TODO: Replace with actual ML model (rust-bert + sentence-transformers)
        if dimension == 0 {
            return Err("Embedding dimension must be greater than 0".into());
        }
        
        Ok(EmbeddingGenerator {
            dimension,
        })
    }

    pub fn generate_embeddings(&mut self, texts: &[String]) -> Result<Vec<Embedding>, Box<dyn Error>> {
        let mut embeddings = Vec::new();
        
        for text in texts {
            let embedding = self.generate_single_embedding(text)?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    pub fn generate_embedding(&mut self, text: &str) -> Result<Embedding, Box<dyn Error>> {
        self.generate_single_embedding(text)
    }

    fn generate_single_embedding(&mut self, text: &str) -> Result<Embedding, Box<dyn Error>> {
        // Simplified hash-based embedding for demonstration
        // In production, this would use a proper sentence transformer model
        
        if text.trim().is_empty() {
            return Ok(vec![0.0; self.dimension]);
        }

        // Normalize text
        let normalized = text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();
        
        // Create deterministic but varied embedding using multiple hash approaches
        let mut embedding = vec![0.0; self.dimension];
        
        // Method 1: Character frequency-based features
        self.add_char_frequency_features(&normalized, &mut embedding);
        
        // Method 2: Word-based features
        self.add_word_features(&normalized, &mut embedding);
        
        // Method 3: Hash-based features for remaining dimensions
        self.add_hash_features(text, &mut embedding);
        
        // Normalize the embedding vector
        self.normalize_embedding(&mut embedding);
        
        Ok(embedding)
    }

    fn add_char_frequency_features(&self, text: &str, embedding: &mut [f32]) {
        let chars_section = (self.dimension / 4).min(26);
        let mut char_counts = [0; 26];
        let mut total_chars = 0;
        
        for c in text.chars() {
            if c.is_ascii_lowercase() {
                let idx = (c as u8 - b'a') as usize;
                if idx < 26 {
                    char_counts[idx] += 1;
                    total_chars += 1;
                }
            }
        }
        
        if total_chars > 0 {
            for i in 0..chars_section {
                embedding[i] = char_counts[i % 26] as f32 / total_chars as f32;
            }
        }
    }

    fn add_word_features(&mut self, text: &str, embedding: &mut [f32]) {
        let start_idx = self.dimension / 4;
        let end_idx = (self.dimension / 2).min(embedding.len());
        
        if start_idx >= end_idx {
            return;
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        
        if word_count == 0 {
            return;
        }
        
        // Average word length
        let avg_word_len = words.iter().map(|w| w.len()).sum::<usize>() as f32 / word_count as f32;
        embedding[start_idx] = (avg_word_len / 20.0).min(1.0); // Normalize to [0,1]
        
        // Vocabulary diversity (unique words / total words)
        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        embedding[start_idx + 1] = unique_words.len() as f32 / word_count as f32;
        
        // Common programming keywords
        let keywords = ["fn", "struct", "impl", "enum", "pub", "use", "let", "mut", "if", "for"];
        for (i, &keyword) in keywords.iter().enumerate() {
            if start_idx + 2 + i < end_idx {
                let count = text.matches(keyword).count() as f32;
                embedding[start_idx + 2 + i] = (count / word_count as f32).min(1.0);
            }
        }
    }

    fn add_hash_features(&self, text: &str, embedding: &mut [f32]) {
        let start_idx = self.dimension / 2;
        
        if start_idx >= embedding.len() {
            return;
        }
        
        // Create multiple hash values using different salts
        for i in start_idx..embedding.len() {
            let salt = format!("salt_{}", i);
            let input = format!("{}{}", text, salt);
            
            let mut hasher = Sha256::new();
            hasher.update(input.as_bytes());
            let hash = hasher.finalize();
            
            // Convert first 4 bytes of hash to f32 in range [-1, 1]
            let byte_idx = (i - start_idx) % 28; // SHA256 produces 32 bytes, use first 28
            if byte_idx < hash.len() {
                let byte_val = hash[byte_idx] as f32;
                embedding[i] = (byte_val / 127.5) - 1.0; // Convert 0-255 to -1.0 to 1.0
            }
        }
    }

    fn normalize_embedding(&self, embedding: &mut [f32]) {
        // L2 normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude > 0.0 {
            for value in embedding.iter_mut() {
                *value /= magnitude;
            }
        }
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    pub fn similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
        if embedding1.len() != embedding2.len() {
            return 0.0;
        }
        
        // Cosine similarity
        let dot_product: f32 = embedding1.iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        dot_product // Embeddings are already normalized
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        EmbeddingGenerator::new(384).expect("Failed to create default embedding generator")
    }
}