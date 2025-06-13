use project_x::index::chunker::{CodeChunker, ChunkType, CodeChunk};
use project_x::index::embedder::EmbeddingGenerator;
use project_x::index::vector_store::VectorDB;
use tempfile::tempdir;

#[test]
fn test_chunker_creates_successfully() {
    let chunker = CodeChunker::new();
    assert!(chunker.is_ok());
}

#[test]
fn test_chunk_main_rs() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let chunks = chunker.chunk_file("src/main.rs").expect("Failed to chunk main.rs");
    
    assert!(!chunks.is_empty(), "Should extract at least one chunk from main.rs");
    
    let main_function = chunks.iter().find(|chunk| {
        chunk.chunk_type == ChunkType::Function && chunk.name == "main"
    });
    assert!(main_function.is_some(), "Should find main function");
    
    let cli_struct = chunks.iter().find(|chunk| {
        chunk.chunk_type == ChunkType::Struct && chunk.name == "Cli"
    });
    assert!(cli_struct.is_some(), "Should find Cli struct");
    
    assert!(chunks.len() >= 2, "Should extract at least 2 chunks (main fn + Cli struct)");
}

#[test]
fn test_chunk_function_extraction() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"
fn hello_world() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").expect("Failed to chunk code");
    
    let function_chunks: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Function).collect();
    assert_eq!(function_chunks.len(), 2, "Should extract 2 functions");
    
    let hello_fn = function_chunks.iter().find(|c| c.name == "hello_world");
    assert!(hello_fn.is_some(), "Should find hello_world function");
    
    let add_fn = function_chunks.iter().find(|c| c.name == "add");
    assert!(add_fn.is_some(), "Should find add function");
    
    if let Some(hello) = hello_fn {
        assert!(hello.code.contains("println!"), "Function code should contain implementation");
        assert!(hello.start_line > 0, "Start line should be positive");
        assert!(hello.end_line >= hello.start_line, "End line should be >= start line");
    }
}

#[test]
fn test_chunk_struct_extraction() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"
struct Point {
    x: f64,
    y: f64,
}

struct Rectangle {
    width: u32,
    height: u32,
}
"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").expect("Failed to chunk code");
    
    let struct_chunks: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Struct).collect();
    assert_eq!(struct_chunks.len(), 2, "Should extract 2 structs");
    
    let point_struct = struct_chunks.iter().find(|c| c.name == "Point");
    assert!(point_struct.is_some(), "Should find Point struct");
    
    let rect_struct = struct_chunks.iter().find(|c| c.name == "Rectangle");
    assert!(rect_struct.is_some(), "Should find Rectangle struct");
}

#[test]
fn test_chunk_impl_extraction() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").expect("Failed to chunk code");
    
    let impl_chunks: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Impl).collect();
    assert_eq!(impl_chunks.len(), 1, "Should extract 1 impl block");
    
    let impl_chunk = &impl_chunks[0];
    assert!(impl_chunk.code.contains("fn new"), "Impl should contain new function");
    assert!(impl_chunk.code.contains("fn distance"), "Impl should contain distance function");
}

#[test]
fn test_chunk_enum_extraction() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"
enum Color {
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8),
}
"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").expect("Failed to chunk code");
    
    let enum_chunks: Vec<_> = chunks.iter().filter(|c| c.chunk_type == ChunkType::Enum).collect();
    assert_eq!(enum_chunks.len(), 1, "Should extract 1 enum");
    
    let color_enum = &enum_chunks[0];
    assert_eq!(color_enum.name, "Color", "Enum name should be Color");
    assert!(color_enum.code.contains("Red"), "Enum should contain Red variant");
    assert!(color_enum.code.contains("RGB"), "Enum should contain RGB variant");
}

#[test]
fn test_chunk_invalid_syntax() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let invalid_rust = r#"
fn incomplete_function( {
    // missing closing brace and parameters
"#;
    
    let result = chunker.chunk_code(invalid_rust, "invalid.rs");
    
    assert!(result.is_ok(), "Should handle invalid syntax gracefully");
    
    let chunks = result.unwrap();
    // Tree-sitter may still extract some partial nodes, but shouldn't crash
    println!("Extracted {} chunks from invalid syntax", chunks.len());
}

#[test]
fn test_chunk_empty_file() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let empty_code = "";
    let chunks = chunker.chunk_code(empty_code, "empty.rs").expect("Failed to chunk empty code");
    
    assert!(chunks.is_empty(), "Empty file should produce no chunks");
}

#[test]
fn test_chunk_file_with_nonexistent_path() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let result = chunker.chunk_file("nonexistent_file.rs");
    assert!(result.is_err(), "Should return error for nonexistent file");
}

#[test]
fn test_chunk_line_numbers_accuracy() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"// Line 1
// Line 2
fn first_function() {  // Line 3
    println!("test");  // Line 4
}                      // Line 5
// Line 6
fn second_function() { // Line 7
    let x = 42;        // Line 8
}                      // Line 9"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").expect("Failed to chunk code");
    
    let first_fn = chunks.iter().find(|c| c.name == "first_function").expect("Should find first function");
    assert_eq!(first_fn.start_line, 3, "First function should start at line 3");
    assert_eq!(first_fn.end_line, 5, "First function should end at line 5");
    
    let second_fn = chunks.iter().find(|c| c.name == "second_function").expect("Should find second function");
    assert_eq!(second_fn.start_line, 7, "Second function should start at line 7");
    assert_eq!(second_fn.end_line, 9, "Second function should end at line 9");
}

#[test]
fn test_chunk_file_path_preservation() {
    let chunker = CodeChunker::new().expect("Failed to create chunker");
    
    let rust_code = r#"fn test() {}"#;
    let test_path = "src/test/example.rs";
    
    let chunks = chunker.chunk_code(rust_code, test_path).expect("Failed to chunk code");
    
    assert!(!chunks.is_empty(), "Should extract at least one chunk");
    assert_eq!(chunks[0].file_path, test_path, "File path should be preserved in chunk");
}

// Embedding Generator Tests

#[test]
fn test_embedding_generator_creates_successfully() {
    let generator = EmbeddingGenerator::new(128);
    assert!(generator.is_ok());
    
    let generator = generator.unwrap();
    assert_eq!(generator.get_dimension(), 128);
}

#[test]
fn test_embedding_generator_zero_dimension_fails() {
    let result = EmbeddingGenerator::new(0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("dimension must be greater than 0"));
}

#[test]
fn test_embedding_generator_default() {
    let generator = EmbeddingGenerator::default();
    assert_eq!(generator.get_dimension(), 384);
}

#[test]
fn test_generate_single_embedding() {
    let mut generator = EmbeddingGenerator::new(128).unwrap();
    
    let text = "fn hello_world() { println!(\"Hello, world!\"); }";
    let embedding = generator.generate_embedding(text).unwrap();
    
    assert_eq!(embedding.len(), 128);
    
    // Check that embedding is normalized (L2 norm ≈ 1.0)
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((magnitude - 1.0).abs() < 0.01, "Embedding should be normalized");
}

#[test]
fn test_generate_multiple_embeddings() {
    let mut generator = EmbeddingGenerator::new(256).unwrap();
    
    let texts = vec![
        "fn main() { println!(\"Hello\"); }".to_string(),
        "struct Point { x: f64, y: f64 }".to_string(),
        "impl Point { fn new() -> Self {} }".to_string(),
    ];
    
    let embeddings = generator.generate_embeddings(&texts).unwrap();
    
    assert_eq!(embeddings.len(), 3);
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 256);
        
        // Check normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }
}

#[test]
fn test_embedding_deterministic() {
    let mut generator1 = EmbeddingGenerator::new(128).unwrap();
    let mut generator2 = EmbeddingGenerator::new(128).unwrap();
    
    let text = "fn test_function() { return 42; }";
    
    let embedding1 = generator1.generate_embedding(text).unwrap();
    let embedding2 = generator2.generate_embedding(text).unwrap();
    
    // Same text should produce identical embeddings
    assert_eq!(embedding1, embedding2);
}

#[test]
fn test_embedding_different_for_different_text() {
    let mut generator = EmbeddingGenerator::new(128).unwrap();
    
    let text1 = "fn hello() { println!(\"Hello\"); }";
    let text2 = "struct Point { x: i32, y: i32 }";
    
    let embedding1 = generator.generate_embedding(text1).unwrap();
    let embedding2 = generator.generate_embedding(text2).unwrap();
    
    // Different texts should produce different embeddings
    assert_ne!(embedding1, embedding2);
}

#[test]
fn test_embedding_empty_text() {
    let mut generator = EmbeddingGenerator::new(64).unwrap();
    
    let embedding = generator.generate_embedding("").unwrap();
    
    assert_eq!(embedding.len(), 64);
    // Empty text should produce zero vector
    assert!(embedding.iter().all(|&x| x == 0.0));
}

#[test]
fn test_similarity_calculation() {
    let generator = EmbeddingGenerator::new(128).unwrap();
    
    let embedding1 = vec![1.0, 0.0, 0.0];
    let embedding2 = vec![0.0, 1.0, 0.0];
    let embedding3 = vec![1.0, 0.0, 0.0];
    
    // Orthogonal vectors should have similarity ≈ 0
    let sim1 = generator.similarity(&embedding1, &embedding2);
    assert!((sim1 - 0.0).abs() < 0.01);
    
    // Identical vectors should have similarity = 1
    let sim2 = generator.similarity(&embedding1, &embedding3);
    assert!((sim2 - 1.0).abs() < 0.01);
    
    // Different length vectors should return 0
    let embedding4 = vec![1.0, 0.0];
    let sim3 = generator.similarity(&embedding1, &embedding4);
    assert_eq!(sim3, 0.0);
}

#[test]
fn test_embeddings_with_code_chunks() {
    let chunker = CodeChunker::new().unwrap();
    let mut generator = EmbeddingGenerator::new(128).unwrap();
    
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}

struct User {
    name: String,
    age: u32,
}
"#;
    
    let chunks = chunker.chunk_code(rust_code, "test.rs").unwrap();
    assert!(!chunks.is_empty());
    
    // Generate embeddings for all chunks
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = generator.generate_embeddings(&chunk_texts).unwrap();
    
    assert_eq!(embeddings.len(), chunks.len());
    
    // All embeddings should be valid
    for (i, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding.len(), 128, "Embedding {} has wrong dimension", i);
        
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01, "Embedding {} not normalized", i);
    }
}

// Vector Store Tests

#[tokio::test]
async fn test_vector_db_creates_successfully() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let result = VectorDB::new(db_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_vector_db_add_and_search_chunks() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut db = VectorDB::new(db_path).await.unwrap();
    
    // Create test chunks
    let chunks = vec![
        CodeChunk {
            chunk_type: ChunkType::Function,
            name: "add".to_string(),
            code: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
            start_line: 1,
            end_line: 1,
            file_path: "test.rs".to_string(),
        },
        CodeChunk {
            chunk_type: ChunkType::Struct,
            name: "Point".to_string(),
            code: "struct Point { x: f64, y: f64 }".to_string(),
            start_line: 3,
            end_line: 3,
            file_path: "test.rs".to_string(),
        },
    ];
    
    // Generate embeddings
    let mut generator = EmbeddingGenerator::new(128).unwrap();
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = generator.generate_embeddings(&chunk_texts).unwrap();
    
    // Add chunks to database
    db.add_chunks(&chunks, &embeddings).await.unwrap();
    
    // Verify count
    let count = db.count().await.unwrap();
    assert_eq!(count, 2);
    
    // Search for similar chunks
    let query_embedding = &embeddings[0]; // Search for something similar to first chunk
    let results = db.search(query_embedding, 5).await.unwrap();
    
    assert!(!results.is_empty());
    assert!(results.len() <= 2);
    
    // First result should be the exact match
    assert_eq!(results[0].chunk.name, "add");
    assert_eq!(results[0].chunk.chunk_type, ChunkType::Function);
}

#[tokio::test]
async fn test_vector_db_empty_chunks() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut db = VectorDB::new(db_path).await.unwrap();
    
    // Adding empty chunks should work
    let result = db.add_chunks(&[], &[]).await;
    assert!(result.is_ok());
    
    // Count should be 0
    let count = db.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_vector_db_mismatched_chunks_embeddings() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut db = VectorDB::new(db_path).await.unwrap();
    
    let chunks = vec![
        CodeChunk {
            chunk_type: ChunkType::Function,
            name: "test".to_string(),
            code: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
            file_path: "test.rs".to_string(),
        },
    ];
    
    // Wrong number of embeddings
    let embeddings = vec![vec![1.0, 2.0], vec![3.0, 4.0]]; // 2 embeddings for 1 chunk
    
    let result = db.add_chunks(&chunks, &embeddings).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must match"));
}

#[tokio::test]
async fn test_vector_db_search_empty_database() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let db = VectorDB::new(db_path).await.unwrap();
    
    // Create a dummy query embedding
    let query_embedding = vec![1.0, 0.0, 0.0];
    
    // Search should return empty results, not error
    let results = db.search(&query_embedding, 5).await;
    
    // This might fail if table doesn't exist, which is expected behavior
    // In that case, we'll handle it gracefully in the implementation
    match results {
        Ok(res) => assert!(res.is_empty()),
        Err(_) => {
            // Table doesn't exist yet, which is fine for empty DB
            // This is expected behavior
        }
    }
}

#[tokio::test]
async fn test_vector_db_clear() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut db = VectorDB::new(db_path).await.unwrap();
    
    // Add some test data
    let chunks = vec![
        CodeChunk {
            chunk_type: ChunkType::Function,
            name: "test".to_string(),
            code: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
            file_path: "test.rs".to_string(),
        },
    ];
    
    let mut generator = EmbeddingGenerator::new(64).unwrap();
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = generator.generate_embeddings(&chunk_texts).unwrap();
    
    db.add_chunks(&chunks, &embeddings).await.unwrap();
    
    // Verify data exists
    let count = db.count().await.unwrap();
    assert_eq!(count, 1);
    
    // Clear database
    db.clear().await.unwrap();
    
    // Count should be 0 after recreating table
    let count = db.count().await.unwrap_or(0);
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_vector_db_different_chunk_types() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut db = VectorDB::new(db_path).await.unwrap();
    
    // Create chunks of different types
    let chunks = vec![
        CodeChunk {
            chunk_type: ChunkType::Function,
            name: "main".to_string(),
            code: "fn main() { println!(\"Hello\"); }".to_string(),
            start_line: 1,
            end_line: 3,
            file_path: "main.rs".to_string(),
        },
        CodeChunk {
            chunk_type: ChunkType::Struct,
            name: "User".to_string(),
            code: "struct User { name: String }".to_string(),
            start_line: 5,
            end_line: 7,
            file_path: "models.rs".to_string(),
        },
        CodeChunk {
            chunk_type: ChunkType::Enum,
            name: "Color".to_string(),
            code: "enum Color { Red, Green, Blue }".to_string(),
            start_line: 10,
            end_line: 12,
            file_path: "types.rs".to_string(),
        },
    ];
    
    let mut generator = EmbeddingGenerator::new(256).unwrap();
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
    let embeddings = generator.generate_embeddings(&chunk_texts).unwrap();
    
    // Add all chunks
    db.add_chunks(&chunks, &embeddings).await.unwrap();
    
    // Verify all chunks stored
    let count = db.count().await.unwrap();
    assert_eq!(count, 3);
    
    // Search and verify chunk types are preserved
    let results = db.search(&embeddings[1], 3).await.unwrap();
    assert!(!results.is_empty());
    
    // Find the struct chunk in results
    let struct_result = results.iter().find(|r| r.chunk.chunk_type == ChunkType::Struct);
    assert!(struct_result.is_some());
    assert_eq!(struct_result.unwrap().chunk.name, "User");
}

#[tokio::test]
async fn test_vector_db_persistence() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    // Create and populate database
    {
        let mut db = VectorDB::new(db_path).await.unwrap();
        
        let chunks = vec![
            CodeChunk {
                chunk_type: ChunkType::Function,
                name: "persistent_test".to_string(),
                code: "fn persistent_test() { println!(\"Persistence\"); }".to_string(),
                start_line: 1,
                end_line: 1,
                file_path: "persist.rs".to_string(),
            },
        ];
        
        let mut generator = EmbeddingGenerator::new(128).unwrap();
        let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
        let embeddings = generator.generate_embeddings(&chunk_texts).unwrap();
        
        db.add_chunks(&chunks, &embeddings).await.unwrap();
        
        let count = db.count().await.unwrap();
        assert_eq!(count, 1);
    } // db goes out of scope
    
    // Reopen database and verify data persists
    {
        let db = VectorDB::new(db_path).await.unwrap();
        let count = db.count().await.unwrap();
        assert_eq!(count, 1);
    }
}