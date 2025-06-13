# PHASE 2: Tree-sitter RAG for Context Compression (Rust Edition)

This phase focuses on implementing a Retrieval-Augmented Generation (RAG) system in Rust using `tree-sitter` to create intelligent, compressed context for the LLM. This is a key part of the cost-saving strategy.

## Task 2.1: Implement AST Chunker

**Goal:** Create a Rust module that can parse source code into an Abstract Syntax Tree (AST) using `tree-sitter` and extract meaningful code chunks.

**Crate Modules:** `src/index/chunker.rs`

**Instructions:**

1.  Create an `index` module: make a `src/index` directory with a `mod.rs` file.
2.  Create `src/index/chunker.rs`.
3.  Implement a `CodeChunker` struct.
4.  Use the `tree-sitter` crate and the appropriate language grammar crate (e.g., `tree-sitter-rust`).
5.  Implement a `chunk_file(file_path)` function that reads a file, parses it, and traverses the AST.
6.  Extract nodes corresponding to function definitions, struct/enum definitions, and other important items. Each chunk should contain the code text and its start/end line numbers.
7.  Return a `Vec` of these chunk objects.
8.  Add `tree-sitter` and `tree-sitter-rust` to `Cargo.toml`.

**Acceptance Criteria:** A `CodeChunker` that can take a Rust file and return a list of structured code chunks.

---

## Task 2.2: Implement Embedding Generator

**Goal:** Create a module to generate vector embeddings for text chunks using a local model.

**Crate Modules:** `src/index/embedder.rs`

**Instructions:**

1.  Create `src/index/embedder.rs`.
2.  Implement an `EmbeddingGenerator` struct.
3.  Use a crate like `rust-bert` to load a pre-trained sentence-transformer model (e.g., `all-MiniLM-L6-v2`). This provides capabilities similar to the Python `sentence-transformers` library.
4.  Implement a function `generate_embeddings(chunks)` that takes a `Vec` of text chunks and returns a `Vec` of their corresponding vector embeddings.
5.  Add `rust-bert` and its dependencies (like `tch` for the PyTorch backend) to `Cargo.toml`. Note that this may require a local PyTorch installation.

**Acceptance Criteria:** An `EmbeddingGenerator` that can convert a slice of strings into a `Vec` of numerical vectors.

---

## Task 2.3: Implement Vector Store

**Goal:** Create a module to store and retrieve vector embeddings using LanceDB.

**Crate Modules:** `src/index/vector_store.rs`

**Instructions:**

1.  Create `src/index/vector_store.rs`.
2.  Implement a `VectorDB` struct.
3.  Use the `lancedb` crate for Rust.
4.  The constructor should connect to a LanceDB database stored at `index/vecdb/`.
5.  Implement an async `add_chunks(chunks, embeddings)` function to add code chunks and their embeddings to the database. Each entry should store the file path, the code chunk, and the embedding vector.
6.  Implement an async `search(query_embedding, top_k)` function that performs a vector search and returns the `top_k` most similar code chunks.
7.  Add `lancedb` to `Cargo.toml`.

**Acceptance Criteria:** A `VectorDB` struct that can store, manage, and search for code chunks based on vector similarity.

---

## Task 2.4: Create Indexing Binary

**Goal:** Create a separate binary within the crate to orchestrate the full RAG indexing process.

**Crate Modules:** `src/bin/indexer.rs`

**Instructions:**

1.  Create a new binary target by adding `src/bin/indexer.rs`.
2.  Add the binary to `Cargo.toml`:
    ```toml
    [[bin]]
    name = "indexer"
    path = "src/bin/indexer.rs"
    ```
3.  The `main` function in `indexer.rs` should:
    a.  Scan the project repository for all `*.rs` files (e.g., using the `walkdir` crate).
    b.  For each file, use the `CodeChunker` to get code chunks.
    c.  Use the `EmbeddingGenerator` to create embeddings for all chunks.
    d.  Use the `VectorDB` to store all the chunks and their embeddings.

**Acceptance Criteria:** Running `cargo run --bin indexer` successfully creates and populates the `index/vecdb/` LanceDB database.

---

## Task 2.5: Write Tests

**Goal:** Create integration tests for the indexing components.

**Directory:** `tests/`

**File to Create:** `tests/index.rs`

**Instructions:**

1.  Create `tests/index.rs`.
2.  Test the `CodeChunker` with a sample Rust file string, verifying that it extracts the correct function and struct chunks.
3.  Test the `EmbeddingGenerator` by checking the output dimensions of the generated embeddings for some sample text.
4.  Test the `VectorDB` by connecting to an in-memory or temporary LanceDB table, adding sample embeddings, and then searching for them to ensure the correct chunks are returned.

**Acceptance Criteria:** All tests in `tests/index.rs` pass when running `cargo test`.

---

## Task 2.6: Benchmark Token Savings

**Goal:** Create a script to measure the effectiveness of the RAG implementation in reducing token count.

**Directory:** `scripts/`

**File to Create:** `scripts/benchmark_rag.py`

**Instructions:**

1.  Create `scripts/benchmark_rag.py`.
2.  The script should take a user query (e.g., "how do I add a new command to the CLI?").
3.  **RAG approach:**
    a.  Generate an embedding for the query.
    b.  Use the `VectorDB` to retrieve the top 3-5 relevant code chunks.
    c.  Concatenate these chunks to form the context.
    d.  Count the total number of tokens in this context.
4.  **Naive approach:**
    a.  Identify a few potentially relevant files (e.g., `utils/cli.py`, `README.md`).
    b.  Concatenate the entire content of these files.
    c.  Count the total tokens.
5.  The script should print a comparison of the token counts for both approaches.

**Acceptance Criteria:** The script runs and outputs a clear comparison of token usage, demonstrating the savings from the RAG approach. 