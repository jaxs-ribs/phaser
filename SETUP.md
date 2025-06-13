# Phaser Setup Instructions

## Phase 1: Voice → Code Pipeline

### Current Status
✅ **Completed**: Phase 1 implementation with placeholder audio/transcription  
🔄 **Next**: Replace placeholders with production implementations

### Quick Start

1. **Set up Gemini API key**:
   ```bash
   export GEMINI_API_KEY="your-api-key-here"
   ```

2. **Run the pipeline**:
   ```bash
   cargo run
   # or with custom duration:
   cargo run -- --duration 10
   ```

### Current Implementation

**Voice Capture** (`src/voice/capture.rs`):
- ✅ Placeholder: Creates mock WAV files
- 🔄 Production: Integrate `cpal` for real audio recording

**Transcription** (`src/voice/transcribe.rs`):
- ✅ Placeholder: Returns mock transcription
- 🔄 Production: Integrate Whisper API or local model

**LLM Client** (`src/llm/gemini_client.rs`):
- ✅ Production: Full Gemini API integration
- Requires `GEMINI_API_KEY` environment variable

### Resolving Dependency Issues

The current implementation uses placeholders due to Rust/Cargo compatibility issues with audio libraries. To get full functionality:

1. **For Audio Recording**:
   ```toml
   # Add to Cargo.toml when compatibility is resolved:
   cpal = "0.15"
   ```

2. **For Whisper Transcription**:
   ```toml
   # Options:
   whisper-rs = "0.10"  # Local model
   # OR use OpenAI API, Google Speech-to-Text, etc.
   ```

### Testing

```bash
cargo test
```

### Next Steps (Phase 2)

1. Tree-sitter integration for code parsing
2. RAG system for context compression
3. Vector store for code embeddings

### Environment Variables

**Required:**
- `GEMINI_API_KEY`: Required for LLM functionality

**Optional (Spending Protection):**
- `GEMINI_MAX_REQUESTS`: Max requests per session (default: 10)
- `GEMINI_MAX_PROMPT_LENGTH`: Max prompt length (default: 2000 chars)
- `WHISPER_MODEL_PATH`: Will be required for local transcription

### 🔒 Spending Protection Features

Built-in safeguards to prevent runaway API costs:

1. **Request Limits**: Max 10 requests per session (configurable)
2. **Prompt Length Limits**: Max 2000 characters (configurable)  
3. **Input Truncation**: Long transcriptions automatically truncated
4. **Response Limits**: Requests concise responses (max 200 words)
5. **Usage Tracking**: Shows remaining requests after each call

**Commands:**
```bash
# Check current usage
cargo run -- --show-usage

# Configure limits
export GEMINI_MAX_REQUESTS=5
export GEMINI_MAX_PROMPT_LENGTH=1000

# Run with limits
cargo run
```

**Safety Notes:**
- Session limits reset on restart
- Monitor Google Cloud Console for billing
- Consider setting up billing alerts
- Test with small limits first

### Architecture

```
Voice Input → [Capture] → WAV File → [Transcribe] → Text → [LLM] → Code Suggestions
```

Current pipeline demonstrates the full flow with placeholder implementations that can be replaced with production code.