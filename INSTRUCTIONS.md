# Project-X – AI-First Coding Assistant

Welcome!  This guide walks you through building, testing and running the **Project-X** toolkit as it stands today.

---
## 1. Prerequisites

| Requirement | Why it is needed |
|-------------|-----------------|
| **Rust 1.74+** | Compile the project (`cargo`) |
| **Git** | Many helpers rely on a Git repository; `git2` is linked at build-time |
| **FFmpeg** *(optional)* | Audio capture is done with `ffmpeg` – only required for voice features |
| Google **Gemini API Key** *(optional)* | Needed to reach the Gemini LLM.  Set in `GEMINI_API_KEY`.  If you prefer OpenRouter, set `OPENROUTER_API_KEY` instead |

Install Rust via [rustup.rs](https://rustup.rs/) and ensure `cargo`, `rustc` are on your `PATH`.

---
## 2. Getting the code
```bash
# clone and enter
$ git clone <repo-url>
$ cd project-x
```

---
## 3. Building & running the test-suite
```bash
# fast dev build
$ cargo build

# full test-suite (takes < 5 s on modern hardware)
$ cargo test
```
All 17 unit-tests should report **OK**.

---
## 4. Environment variables
| Variable | Purpose | Example |
|----------|---------|---------|
| `GEMINI_API_KEY` | Google Generative AI key (standard path) | `export GEMINI_API_KEY=ya29.a0…` |
| `OPENROUTER_API_KEY` | Alternative gateway – bypasses the need for Google keys | `export OPENROUTER_API_KEY=orc_…` |
| `GEMINI_MODEL` | Override model name (defaults to `gemini-2.5-pro`) | `export GEMINI_MODEL=gemini-pro` |
| `GEMINI_MAX_REQUESTS` | Safety-limit per session (default `10`) | `export GEMINI_MAX_REQUESTS=25` |
| `GEMINI_MAX_PROMPT_LENGTH` | Prompt length cut-off (default `2000`) | `export GEMINI_MAX_PROMPT_LENGTH=4096` |

> If **neither** `GEMINI_API_KEY` **nor** `OPENROUTER_API_KEY` is set the LLM client will refuse to start and tests that touch it will be skipped.

---
## 5. CLI Usage (main binary)
The main entry point is `project-x` (defined in `src/main.rs`).

```
$ cargo run --release -- --help
```
Key options:

* `--prompt "…"`   Run the **Autonomous TDD Loop** – the assistant edits the repo, runs tests and retries until GREEN.
* `--test-llm "…"`   Send a simple prompt to Gemini and print the raw reply.
* `--duration -d <sec>` / `--output -o <wav>`   Voice-to-code demo (records audio, transcribes, asks Gemini for advice).
* `--sandbox`   Run the autonomous loop in an isolated copy of the repo (`./sandboxes/<timestamp>`).  Add `--keep-sandbox` to keep it afterwards.
* `--dry-run`   Preview diffs without writing files.
* `--skip-tests`   Apply patches without running the cargo test suite.

Examples:
```bash
# Ask the assistant to add a hello_world() fn in a sandbox
$ cargo run --release -- --prompt "Add hello_world to src/lib.rs" --sandbox

# Quick Gemini sanity check
$ cargo run --release -- --test-llm "explain the builder pattern"
```

---
## 6. Other binaries
Project-X ships several small examples – all invokable via `cargo run --bin …`

| Binary | Purpose |
|--------|---------|
| `tdd_demo` | Minimal demonstration of the autonomous TDD loop; hard-coded prompt inside `src/bin/tdd_demo.rs`. |
| `git_demo` | Shows Git helper utilities. |
| `memory_demo` | Demonstrates the conversation-history `MemoryManager`. |
| `indexer` | Simple file-system indexer helper. |

Example:
```bash
$ cargo run --bin tdd_demo
```

---
## 7. Project structure (high-level)
```
├── src
│   ├── orchestrator   – context builder, memory DB, autonomous logic
│   ├── llm            – Gemini / OpenRouter client
│   ├── edit           – patch creation / application helpers
│   ├── hooks          – autotest executor, Git hooks
│   ├── utils          – git client, misc helpers
│   └── voice          – audio capture & Whisper transcription
└── tests              – black-box & integration tests
```

---
## 8. Common issues
* **`GEMINI_API_KEY environment variable not set`** – export the key (or `OPENROUTER_API_KEY`).
* **No audio recorded** – ensure `ffmpeg` is installed and your input device name matches the default; see `voice/capture.rs` for platform notes.
* **Git operations fail inside tests** – make sure git is installed and you have write permissions in `TMPDIR`.

---
Happy hacking! 🎉 