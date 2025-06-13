#!/usr/bin/env python3
"""
RAG Token Savings Benchmark Script

This script demonstrates the token savings achieved by using RAG (Retrieval-Augmented Generation)
compared to naive approaches that include entire files in the context.

Usage:
    python scripts/benchmark_rag.py "how do I add a new command to the CLI?"
"""

import sys
import json
import subprocess
import os
import tempfile
from pathlib import Path

def count_tokens(text):
    """
    Simple token counting using whitespace splitting.
    In practice, you'd use a proper tokenizer like tiktoken for OpenAI models.
    """
    return len(text.split())

def run_indexer_search(query, top_k=5):
    """
    Use the Rust indexer binary to search for relevant code chunks.
    """
    try:
        # Determine the project root directory
        script_dir = Path(__file__).parent
        project_root = script_dir.parent
        
        # Run the indexer search command
        result = subprocess.run([
            'cargo', 'run', '--bin', 'indexer', '--', 
            '--search', query, 
            '--top-k', str(top_k)
        ], capture_output=True, text=True, cwd=str(project_root))
        
        if result.returncode != 0:
            print(f"Error running indexer: {result.stderr}")
            return []
        
        # Parse the output to extract code chunks
        # This is a simple parser for the search output format
        chunks = []
        lines = result.stdout.split('\n')
        current_chunk = None
        
        for line in lines:
            if line.strip().startswith(f"1.") or line.strip().startswith(f"2.") or line.strip().startswith(f"3.") or line.strip().startswith(f"4.") or line.strip().startswith(f"5."):
                if "Score:" in line:
                    # Extract the code description
                    parts = line.split("'")
                    if len(parts) >= 2:
                        current_chunk = {
                            'name': parts[1],
                            'file': parts[-1] if 'in ' in line else 'unknown',
                            'code': ''
                        }
            elif line.strip().startswith("Lines") and current_chunk:
                # Extract the code content
                code_start = line.find(": ") + 2
                if code_start > 1:
                    current_chunk['code'] = line[code_start:]
                    chunks.append(current_chunk)
                    current_chunk = None
        
        return chunks
    except Exception as e:
        print(f"Error running search: {e}")
        return []

def get_file_contents(file_paths):
    """
    Read the entire contents of specified files.
    """
    contents = []
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    for file_path in file_paths:
        full_path = project_root / file_path
        try:
            if full_path.exists():
                with open(full_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    contents.append({
                        'file': file_path,
                        'content': content,
                        'size': len(content)
                    })
            else:
                print(f"Warning: File {file_path} not found")
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
    
    return contents

def benchmark_rag_approach(query):
    """
    Benchmark the RAG approach using vector search.
    """
    print(f"🔍 RAG Approach: Searching for '{query}'")
    
    # Get relevant chunks using vector search
    chunks = run_indexer_search(query, top_k=5)
    
    if not chunks:
        print("❌ No chunks found from vector search")
        return 0, ""
    
    # Combine the relevant chunks
    rag_context = f"Query: {query}\n\nRelevant code chunks:\n\n"
    for i, chunk in enumerate(chunks, 1):
        rag_context += f"--- Chunk {i}: {chunk['name']} from {chunk['file']} ---\n"
        rag_context += chunk['code'] + "\n\n"
    
    token_count = count_tokens(rag_context)
    
    print(f"✅ RAG found {len(chunks)} relevant chunks")
    print(f"📊 RAG context tokens: {token_count}")
    
    return token_count, rag_context

def benchmark_naive_approach(query):
    """
    Benchmark the naive approach using entire files.
    """
    print(f"📁 Naive Approach: Including entire relevant files")
    
    # Define potentially relevant files based on the query
    # This is a heuristic approach - in practice you'd analyze the query more intelligently
    relevant_files = []
    
    query_lower = query.lower()
    if any(word in query_lower for word in ['cli', 'command', 'argument', 'clap']):
        relevant_files.extend(['src/main.rs', 'src/bin/indexer.rs'])
    if any(word in query_lower for word in ['voice', 'audio', 'record']):
        relevant_files.extend(['src/voice/mod.rs', 'src/voice/capture.rs'])
    if any(word in query_lower for word in ['llm', 'gemini', 'api']):
        relevant_files.extend(['src/llm/gemini_client.rs'])
    if any(word in query_lower for word in ['index', 'search', 'chunk', 'embed']):
        relevant_files.extend(['src/index/chunker.rs', 'src/index/embedder.rs', 'src/index/vector_store.rs'])
    
    # Default files if no specific keywords found
    if not relevant_files:
        relevant_files = ['src/main.rs', 'Cargo.toml', 'README.md']
    
    # Remove duplicates while preserving order
    relevant_files = list(dict.fromkeys(relevant_files))
    
    # Get file contents
    file_contents = get_file_contents(relevant_files)
    
    # Combine all file contents
    naive_context = f"Query: {query}\n\nEntire file contents:\n\n"
    total_chars = 0
    
    for file_info in file_contents:
        naive_context += f"--- File: {file_info['file']} ---\n"
        naive_context += file_info['content'] + "\n\n"
        total_chars += file_info['size']
    
    token_count = count_tokens(naive_context)
    
    print(f"✅ Naive approach included {len(file_contents)} files")
    print(f"📊 Total file size: {total_chars:,} characters")
    print(f"📊 Naive context tokens: {token_count}")
    
    return token_count, naive_context

def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/benchmark_rag.py \"your query here\"")
        print("Example: python scripts/benchmark_rag.py \"how do I add a new command to the CLI?\"")
        sys.exit(1)
    
    query = sys.argv[1]
    
    print("=" * 60)
    print("🚀 RAG Token Savings Benchmark")
    print("=" * 60)
    print(f"Query: {query}")
    print()
    
    # Benchmark RAG approach
    rag_tokens, rag_context = benchmark_rag_approach(query)
    print()
    
    # Benchmark naive approach
    naive_tokens, naive_context = benchmark_naive_approach(query)
    print()
    
    # Compare results
    print("=" * 60)
    print("📊 COMPARISON RESULTS")
    print("=" * 60)
    
    if rag_tokens > 0 and naive_tokens > 0:
        savings = naive_tokens - rag_tokens
        savings_percent = (savings / naive_tokens) * 100
        
        print(f"RAG approach:   {rag_tokens:,} tokens")
        print(f"Naive approach: {naive_tokens:,} tokens")
        print(f"Token savings:  {savings:,} tokens ({savings_percent:.1f}% reduction)")
        
        if savings > 0:
            print(f"✅ RAG approach saves {savings:,} tokens!")
            efficiency = naive_tokens / rag_tokens if rag_tokens > 0 else float('inf')
            print(f"🔥 RAG is {efficiency:.1f}x more token-efficient")
        else:
            print(f"❌ RAG approach uses {abs(savings):,} more tokens")
    else:
        print("❌ Could not complete comparison - one or both approaches failed")
    
    print()
    print("💡 Note: This uses simple whitespace tokenization. Real LLM tokenizers")
    print("   (like tiktoken) would give different counts, but the relative savings")
    print("   comparison would be similar.")
    
    # Optionally save contexts to files for inspection
    save_contexts = os.getenv('SAVE_CONTEXTS', '').lower() in ['true', '1', 'yes']
    if save_contexts:
        with tempfile.NamedTemporaryFile(mode='w', suffix='_rag_context.txt', delete=False) as f:
            f.write(rag_context)
            print(f"📝 RAG context saved to: {f.name}")
        
        with tempfile.NamedTemporaryFile(mode='w', suffix='_naive_context.txt', delete=False) as f:
            f.write(naive_context)
            print(f"📝 Naive context saved to: {f.name}")

if __name__ == "__main__":
    main()