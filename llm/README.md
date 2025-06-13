# /llm

This directory contains the adapters for interfacing with various Large Language Models (LLMs).

Its key component is an abstract `LLMProvider` interface that defines a common set of methods (e.g., `get_completion`, `get_streaming_completion`). Concrete implementations for different providers (Gemini, OpenAI, Groq, local models via Ollama) will inherit from this base class.

This plug-in architecture is essential for avoiding vendor lock-in and allowing users to choose the best model for their needs and budget.

Each file in this directory should be a client for a specific LLM provider (e.g., `gemini_client.js`, `openai_client.go`). 