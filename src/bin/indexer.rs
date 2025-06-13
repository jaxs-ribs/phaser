use clap::Parser;
use project_x::index::chunker::CodeChunker;
use project_x::index::embedder::EmbeddingGenerator;
use project_x::index::vector_store::VectorDB;
use std::error::Error;
use std::fs;
use std::path::Path;
use tokio;

#[derive(Parser)]
#[clap(name = "indexer")]
#[clap(about = "Code Indexer - Extract and analyze code chunks for RAG system")]
struct Cli {
    /// Directory to scan for Rust files
    #[clap(short, long, default_value = "src")]
    directory: String,
    
    /// Show detailed chunk information
    #[clap(short, long)]
    verbose: bool,
    
    /// Limit number of files to process (for testing)
    #[clap(short, long)]
    limit: Option<usize>,
    
    /// Generate embeddings for code chunks
    #[clap(short, long)]
    embeddings: bool,
    
    /// Embedding dimension
    #[clap(long, default_value = "384")]
    embedding_dimension: usize,
    
    /// Store embeddings in vector database
    #[clap(short, long)]
    store: bool,
    
    /// Vector database path
    #[clap(long, default_value = "index/vecdb")]
    db_path: String,
    
    /// Clear existing vector database before storing
    #[clap(long)]
    clear_db: bool,
    
    /// Search for similar code chunks
    #[clap(long)]
    search: Option<String>,
    
    /// Number of search results to return
    #[clap(long, default_value = "5")]
    top_k: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    
    // Handle search functionality first
    if let Some(search_query) = &cli.search {
        println!("🔍 Searching for: \"{}\"", search_query);
        
        let db = VectorDB::new(&cli.db_path).await?;
        let count = db.count().await?;
        
        if count == 0 {
            println!("❌ Vector database is empty. Please run indexing with --store first.");
            return Ok(());
        }
        
        println!("📊 Searching {} stored chunks...", count);
        
        // Generate embedding for search query
        let mut embedder = EmbeddingGenerator::new(cli.embedding_dimension)?;
        let query_embedding = embedder.generate_embedding(search_query)?;
        
        // Search vector database
        let results = db.search(&query_embedding, cli.top_k).await?;
        
        if results.is_empty() {
            println!("❌ No similar code chunks found.");
        } else {
            println!("\n🎯 Found {} similar code chunks:\n", results.len());
            
            for (i, result) in results.iter().enumerate() {
                println!("{}. [Score: {:.3}] {:?} '{}' in {}", 
                        i + 1, 
                        result.score, 
                        result.chunk.chunk_type, 
                        result.chunk.name,
                        result.chunk.file_path);
                
                println!("   Lines {}-{}: {}", 
                        result.chunk.start_line, 
                        result.chunk.end_line,
                        if result.chunk.code.len() > 100 {
                            format!("{}...", &result.chunk.code[..100])
                        } else {
                            result.chunk.code.clone()
                        });
                println!();
            }
        }
        
        return Ok(());
    }
    
    println!("🔍 Starting code indexing process...");
    println!("📂 Scanning directory: {}", cli.directory);
    
    let chunker = CodeChunker::new()?;
    let mut embedder = if cli.embeddings || cli.store {
        Some(EmbeddingGenerator::new(cli.embedding_dimension)?)
    } else {
        None
    };
    
    // Initialize vector database if storing
    let mut vector_db = if cli.store {
        let mut db = VectorDB::new(&cli.db_path).await?;
        if cli.clear_db {
            println!("🗑️  Clearing existing vector database...");
            db.clear().await?;
        }
        Some(db)
    } else {
        None
    };
    
    let rust_files = find_rust_files(&cli.directory)?;
    
    let files_to_process = if let Some(limit) = cli.limit {
        rust_files.into_iter().take(limit).collect()
    } else {
        rust_files
    };
    
    println!("📄 Found {} Rust files to process", files_to_process.len());
    
    let mut total_chunks = 0;
    let mut total_functions = 0;
    let mut total_structs = 0;
    let mut total_impls = 0;
    let mut total_embeddings = 0;
    let mut total_stored = 0;
    
    for file_path in files_to_process {
        if cli.verbose {
            println!("\n🔍 Processing: {}", file_path);
        }
        
        match chunker.chunk_file(&file_path) {
            Ok(chunks) => {
                total_chunks += chunks.len();
                
                // Generate embeddings if requested
                let embeddings = if let Some(ref mut embedder) = embedder {
                    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.code.clone()).collect();
                    match embedder.generate_embeddings(&chunk_texts) {
                        Ok(embs) => {
                            total_embeddings += embs.len();
                            Some(embs)
                        },
                        Err(e) => {
                            eprintln!("⚠️  Warning: Failed to generate embeddings for {}: {}", file_path, e);
                            None
                        }
                    }
                } else {
                    None
                };
                
                // Store in vector database if requested
                if let (Some(ref mut db), Some(ref embs)) = (&mut vector_db, &embeddings) {
                    match db.add_chunks(&chunks, embs).await {
                        Ok(()) => {
                            total_stored += chunks.len();
                            if cli.verbose {
                                println!("  💾 Stored {} chunks in vector database", chunks.len());
                            }
                        },
                        Err(e) => {
                            eprintln!("⚠️  Warning: Failed to store chunks from {}: {}", file_path, e);
                        }
                    }
                }
                
                for (i, chunk) in chunks.iter().enumerate() {
                    match chunk.chunk_type {
                        project_x::index::chunker::ChunkType::Function => total_functions += 1,
                        project_x::index::chunker::ChunkType::Struct => total_structs += 1,
                        project_x::index::chunker::ChunkType::Impl => total_impls += 1,
                        _ => {}
                    }
                    
                    if cli.verbose {
                        let embedding_info = if let Some(ref embs) = embeddings {
                            if i < embs.len() {
                                format!(" [emb: {}d]", embs[i].len())
                            } else {
                                " [no emb]".to_string()
                            }
                        } else {
                            "".to_string()
                        };
                        
                        println!("  📝 {:?}: {} (lines {}-{}){}", 
                                chunk.chunk_type, 
                                chunk.name, 
                                chunk.start_line, 
                                chunk.end_line,
                                embedding_info);
                    }
                }
                
                if !cli.verbose {
                    print!(".");
                    if total_chunks % 50 == 0 {
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error processing {}: {}", file_path, e);
            }
        }
    }
    
    if !cli.verbose {
        println!();
    }
    
    println!("\n✅ Indexing complete!");
    println!("📊 Summary:");
    println!("  • Total chunks: {}", total_chunks);
    println!("  • Functions: {}", total_functions);
    println!("  • Structs: {}", total_structs);
    println!("  • Impl blocks: {}", total_impls);
    println!("  • Other items: {}", total_chunks - total_functions - total_structs - total_impls);
    
    if embedder.is_some() {
        println!("  • Embeddings generated: {}", total_embeddings);
        println!("  • Embedding dimension: {}", cli.embedding_dimension);
    }
    
    if vector_db.is_some() {
        println!("  • Chunks stored in vector DB: {}", total_stored);
        println!("  • Vector database path: {}", cli.db_path);
    }
    
    println!("\n💡 Next steps:");
    if embedder.is_none() {
        println!("  • Run with --embeddings to generate vector embeddings");
    }
    if vector_db.is_none() {
        println!("  • Run with --store to save chunks to vector database");
    }
    if vector_db.is_some() {
        println!("  • Query the vector database for semantic search");
        println!("  • Integrate with main voice pipeline for RAG");
    }
    
    if embedder.is_some() {
        println!("\n📊 Embedding Statistics:");
        println!("  • Ready for vector store integration");
        println!("  • Can compute similarity between code chunks");
        println!("  • Suitable for semantic search and RAG");
    }
    
    if vector_db.is_some() {
        println!("\n🔍 Vector Database Commands:");
        println!("  • Search: cargo run --bin indexer -- --store --embeddings");
        println!("  • Clear DB: cargo run --bin indexer -- --store --clear-db");
        println!("  • Database location: {}", cli.db_path);
    }
    
    Ok(())
}

fn find_rust_files(directory: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut rust_files = Vec::new();
    visit_dir(Path::new(directory), &mut rust_files)?;
    Ok(rust_files)
}

fn visit_dir(dir: &Path, rust_files: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Skip target directory and other build artifacts
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                if !["target", ".git", "node_modules"].contains(&dir_name) {
                    visit_dir(&path, rust_files)?;
                }
            }
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                if let Some(path_str) = path.to_str() {
                    rust_files.push(path_str.to_string());
                }
            }
        }
    }
    
    Ok(())
}