# PHASE 1: Voice → Code (Rust Edition)

This phase focuses on building the initial "voice to code" pipeline in Rust. It involves capturing voice input, transcribing it to text, and sending it to the LLM to get a code-related response. All components will be built as part of the main Rust crate.

## Task 1.1: Implement Voice Capture Module

**Goal:** Create a Rust module to capture audio from the system's microphone and save it as a WAV file.

**Crate Modules:** `src/voice/capture.rs`

**Instructions:**

1.  Create a `voice` module: make a `src/voice` directory with a `mod.rs` file.
2.  Create `src/voice/capture.rs`.
3.  Inside `capture.rs`, implement a `VoiceRecorder` struct.
4.  Use a crate like `cpal` to handle cross-platform audio input and `hound` to write the WAV file.
5.  Implement a function `record_audio(output_filename, duration)` that records audio for a specified duration and saves it.
6.  Ensure the WAV file is saved in a format compatible with Whisper (e.g., 16-bit, 16kHz, mono).
7.  Add `cpal` and `hound` to `Cargo.toml`.

**Acceptance Criteria:** A `VoiceRecorder` that can successfully record and save audio. The output file should be a valid WAV file.

---

## Task 1.2: Implement Whisper Transcription Module

**Goal:** Create a module to transcribe an audio file to text using a local Whisper model.

**Crate Modules:** `src/voice/transcribe.rs`

**Instructions:**

1.  Create `src/voice/transcribe.rs`.
2.  Implement a `Transcriber` struct.
3.  Use a crate like `whisper-rs` which provides safe Rust bindings to `whisper.cpp`.
4.  The `Transcriber` should be initialized with a path to a Whisper model, loaded from the `WHISPER_MODEL_PATH` environment variable.
5.  Implement a function `transcribe_audio(input_filename)` that takes a WAV file path.
6.  This function should use the `whisper-rs` API to get the transcribed text.
7.  Handle potential errors from the transcription process.
8.  Add `whisper-rs` to `Cargo.toml`.

**Acceptance Criteria:** A `Transcriber` that can take a path to a WAV file and return the transcribed text.

---

## Task 1.3: Implement Gemini LLM Client

**Goal:** Create a client to communicate with the Google Gemini API.

**Crate Modules:** `src/llm/gemini_client.rs`

**Instructions:**

1.  Create an `llm` module: make a `src/llm` directory with a `mod.rs` file.
2.  Create `src/llm/gemini_client.rs`.
3.  Inside `gemini_client.rs`, implement a `GeminiClient` struct.
4.  Use a crate like `reqwest` for making HTTP requests and `serde_json` for handling JSON data.
5.  The `GeminiClient` should be initialized with the `GEMINI_API_KEY` from an environment variable.
6.  Implement an async function `generate(prompt)` that builds the correct JSON body, sends the request to the Gemini API, and returns the text response.
7.  Add `reqwest`, `serde`, `serde_json`, and `tokio` (for the async runtime) to `Cargo.toml`.

**Acceptance Criteria:** A `GeminiClient` that can connect to the Gemini API and return a response for a given prompt.

---

## Task 1.4: Create a Demo CLI

**Goal:** Update the main binary to demonstrate the end-to-end voice-to-code-suggestion workflow.

**Crate Modules:** `src/main.rs`

**Instructions:**

1.  Modify `src/main.rs` to create a simple command-line application. A crate like `clap` can be used for argument parsing.
2.  The `main` function (which must be `async`) should:
    a.  Define a path for a temporary WAV file.
    b.  Call the `VoiceRecorder` to record 5 seconds of audio.
    c.  Call the `Transcriber` to convert the saved audio to text.
    d.  Call the `GeminiClient` with the transcribed text as the prompt.
    e.  Print the final response from the LLM to the console.
3.  Add `clap` to `Cargo.toml`.

**Acceptance Criteria:** Running `cargo run` should execute the full pipeline: record audio, transcribe it, get a response from Gemini, and print it to the terminal.

---

## Task 1.5: Write Tests

**Goal:** Create unit and integration tests for the new modules.

**Directory:** `tests/`

**Files to Create:**
*   `tests/voice.rs`
*   `tests/llm.rs`

**Instructions:**

1.  Create `tests/voice.rs`:
    *   Write an integration test for the voice module.
    *   Since testing audio hardware is difficult, focus on mocking parts of the process. For the `Transcriber`, use a pre-recorded silent WAV file and test that the transcription call is made correctly. Use `mockall` or similar if needed.
2.  Create `tests/llm.rs`:
    *   Write an integration test for the `GeminiClient`.
    *   Use a mocking library like `wiremock` to mock the Gemini API endpoint.
    *   Ensure the client constructs the correct request and handles a mock API response properly.
    *   Test that the `GEMINI_API_KEY` is being read from the environment.

**Acceptance Criteria:** `cargo test` should run and all tests should pass. 