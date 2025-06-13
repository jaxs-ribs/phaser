# /index

This module handles the core task of context compression via Retrieval-Augmented Generation (RAG). Its goal is to provide the LLM with the most relevant code context while using the fewest possible tokens.

It contains the logic for:
1.  **Chunking**: Breaking down source code into meaningful, semantic units. The initial plan is to use `tree-sitter` for AST-based chunking.
2.  **Embedding**: Converting code chunks into vector embeddings.
3.  **Vector Store**: Storing and retrieving chunk embeddings using a local vector database like `LanceDB`.

This is a critical component for achieving the project's strict budget goals. 