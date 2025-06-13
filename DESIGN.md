# Project X • AI-First Coding Assistant

> **DESIGN SPEC – living document**
> **Version 1.0 (final draft)** (**2025-06-13**)

*Maintainer-in-charge: `@your-handle` – edit responsibly; squash all comments after approval.*

---

## 0 Table of Contents

1. Mission & Non-Goals
2. Core Pillars
3. Architecture Overview
4. Vertical-Slice Roadmap & Timeline
5. Prompt Cookbook
6. Cost & Budget Guardrails
7. Security & Safety Model
8. Directory Layout
9. Environment Variables
10. Contributing & Coding Standards
11. Success Metrics
12. Open Questions / Parking Lot
13. Implementation Wisdom & Recipes
14. Phased & Parallelized Implementation Plan

---

## 1 Mission & Non-Goals

### 1.1 Mission

Deliver an **open-source, Gemini-centric replacement for Claude Code** that:

*   Runs on **Gemini 2.5 Pro** by default, with plug-in LLM adapters to support Groq, OpenAI, and local models.
*   Interfaces **hands-free** via a 'walk-and-code' workflow, primarily through a laptop mic, then a phone app (iOS/Android) and e-ink devices (Daylight Computer).
*   Generates, tests, and debugs code *autonomously* under a strict, observable Test-Driven Development (TDD) discipline.
*   Operates on a **≤ $200 USD / month** API budget for a heavy solo user by aggressively minimizing token usage.
*   Is developed via **intensive dogfooding**—used to build itself, as well as complex projects like renderers, physics engines, and websites.
*   Provides a **self-hosted, observable environment** that overcomes the limitations of sandboxed tools (e.g., no GPU access, no display rendering).

### 1.2 Explicit Non-Goals (MVP-slice)

*   IDE/editor plugins (VS Code, JetBrains) – Phase > 3.
*   Multi-user pair-programming (Urbit-style networking is a post-MVP idea) – Phase 5.
*   Private fine-tuning or RLHF – out of scope (API only).
*   Non-x86 build targets – ARM/Daylight specific packaging post-M3.

---

## 2 Core Pillars

|  #  | Pillar                 | Rationale                                       | MVP Deliverable                    |
| :-: | ---------------------- | ----------------------------------------------- | ---------------------------------- |
|  P1 | Voice-first UX         | Keep user mobile and hands-free                 | Mic → Whisper transcript           |
|  P2 | Local shell toolbelt   | Use free CPU cycles to save expensive API tokens | `ripgrep` • `patch` • `pytest`     |
|  P3 | Context compression    | Stay under 8k average input tokens to save cost | chunk → embed → top-k RAG          |
|  P4 | Autonomous TDD loop    | Close bugfix loop without human approval        | `pytest` + 3-retry auto-fix loop   |
|  P5 | Plug-in LLMs           | Avoid vendor lock-in, allow for user choice     | Adapter interface for LLM clients  |
|  P6 | Observability feedback | Enable visual debugging for UIs and games       | Playwright screenshot + `ffmpeg` stub |

The pillars are designed to work in concert. **P2** (Local toolbelt) and **P3** (Context compression) are the primary levers for achieving our aggressive cost-saving goals. By using fast, local tools like `ripgrep` and `tree-sitter` to intelligently filter and rank code context, we can construct highly relevant, token-efficient prompts. This avoids naive, expensive context-stuffing and makes a sub-$200 monthly budget feasible even for heavy use. **P4** and **P6** combine to create a powerful autonomous loop where the agent doesn't just fix code based on text traces, but eventually on visual feedback from the applications it builds.

---

## 3 Architecture Overview

```
           ┌───────────────┐
Mic/Phone →│ Voice Capture  │──► Whisper.cpp ─┐
           └───────────────┘                  │ transcript ∿ 1 k tok
                                              ▼
 ┌──────────────────┐     repo chunks    ┌────────────────────────────┐
 │   Router CLI     │────────────┐──────▶│   Orchestrator (12 k ctx) │
 └──────────────────┘            │       ├────────────────────────────┤
     ▲        ▲                 │       │ Prompt Builder  │ Budget  │
     │        │                 │       │ History Trimmer │ Meter   │
User cmd   Retry loop           │       └────────┬──────────────────┘
     │        │                 │                │
     │        │        LLM call │ IN ≤ 8 k tok   │
     │        └─────────┬───────┴────────────────┘
     │                  ▼
     │          ┌───────────────┐
     └──────────▶│ Gemini 2.5   │
                └───────────────┘
                         │ diff (≤2 k)
                         ▼
                ┌──────────────┐
                │ Patcher (FS) │
                └──────────────┘
                         ▲
                ┌────────┴────────┐
                │ autotest (pytest)|
                └──────────────────┘
```

### 3.1 Data stores

| Store                      | File/Location            | Purpose                |
| -------------------------- | ------------------------ | ---------------------- |
| **Embeddings**             | `index/vecdb/` (LanceDB) | semantic search chunks |
| **Conversation summaries** | `memory.sqlite`          | long-term recall       |
| **Budget log**             | `~/.projectx/budget.log` | monthly spend audit    |

### 3.2 Inter-device Protocol (M3-preview)

*   **Transport:** WebSocket (`/voice` endpoint).
*   **Auth:** JWT in `Sec-WebSocket-Protocol` header.
*   **Message:** JSON `{pcm: <bytes b64>, model: 'base.en'}` or `{text:'…'}`.
*   **Rate:** 16 kHz mono chunks every 250 ms, sent from the mobile client to the host machine.

### 3.3 Autonomous TDD & Observability Loop

The core of the agent's autonomy comes from a tight feedback loop.
1.  **Generate**: The LLM generates a code change (diff).
2.  **Apply**: The `Patcher` applies the change to the filesystem.
3.  **Test**: The `autotest` hook triggers `pytest`.
4.  **Observe**:
    *   **Text**: `stdout` and `stderr` from the test run are captured. If tests fail, the traceback is fed back into the orchestrator's context for the next cycle.
    *   **Visual (M4+)**: For UI/graphics work, the test can trigger a screenshot or a short video recording (`ffmpeg`). This visual output is then processed (e.g., OCR, captioning, or direct analysis by a multimodal model) and the result is fed back into the context.
5.  **Repeat**: The loop continues, with the LLM attempting to fix the code based on the observed output, until all tests pass or a retry limit (e.g., 3 attempts) is reached. This entire process is autonomous, requiring no "Apply this diff? [y/n]" confirmations from the user.

---

## 4 Vertical-Slice Roadmap & Timeline (Gantt-style)

The project will be built in distinct, testable milestones.

| Week  | Milestone               | Owner  | Key deliverables                                                                                                  |
| ----- | ----------------------- | ------ | ----------------------------------------------------------------------------------------------------------------- |
| 0 – 1 | **M0 Bootstrap**        | @alice | Repo creation, CI (`black`, `ruff`), `pyproject.toml`, `.gitignore`, and an empty `tests/` suite.                     |
| 1 – 3 | **M1 Voice → Code**     | @bob   | `voice/` module (`PyAudio` + `whisper.cpp`), `llm/gemini_client.py`, `demo_voice_cli.sh` that passes green tests.      |
| 4 – 5 | **M2 Tree-sitter RAG**  | @carol | `index/` module with `ast_chunker.py` using `tree-sitter`. Benchmark token savings against naive chunking.          |
| 6 – 7 | **M3 Mobile companion** | @dave  | React Native POC for voice chat; `mobile/` dir with WebSocket protocol spec.                                        |
| 8 – 9 | **M4 Observability**    | @eve   | `hooks/screenshot.py` using `Playwright` + `ffmpeg` for GIF capture. OCR loop for visual feedback.                  |
| 10 +  | **M5 Memory & PR**      | team   | `memory.sqlite` for conversation history, `utils/git_client.py` for PR summarization, `tmux` session launcher.        |

> *Adjust as capacity shifts – timeline is aspirational.*

Progress tracked in `github.com/project-x/roadmap` project board.

---

## 5 Prompt Cookbook

A collection of well-tested prompt templates is crucial for reliable and cost-effective operation.

| Filename                 | Template hash (sha256) | Upstream         | Notes                                                              |
| ------------------------ | ---------------------- | ---------------- | ------------------------------------------------------------------ |
| `system_diff.txt`        | `b31c…9af`             | Aider 0.84       | The core diff-only prompt with file allow-list and NO_CHANGES sentinel. |
| `system_tdd.txt`         | `aa8e…513`             | Ready-Set-Cloud  | Instructions for the test-retry loop, with a hard limit of 3 retries.   |
| `repo_map_header.txt`    | `44fd…8c6`             | Cursor demo      | A compact file manifest with path, size, and exported symbols.         |
| `screenshot_analyse.txt` | `f10b…0ce`             | Open-Interpreter | A prompt to make a vision model describe errors or UI state in an image. |

> **Any change** to a template requires bumping `TEMPLATE_SCHEMA` const in `orchestrator.py`.

### 5.1 Example compiled prompt (Gemini)

```text
<system>
(SUMMARY) You are a senior developer… Rules: respond only with unified diff.
</system>
<assistant repo_map>
# size path symbols
1 189 src/cli.py : main, parse_args
2  90 tests/test_cli.py : TestMain
</assistant>
<assistant memory>
• 2025-06-11 we agreed to use Markdown → HTML library 'markdown2'
</assistant>
<user>
Add CLI option --output-dir and update tests.
</user>
```

### 5.2 Prompting Philosophy

Our prompt strategy is built on principles learned from successful open-source tools, optimized for low cost and high reliability.
*   **Diff-only Responses**: For code modifications, the LLM is instructed to respond *only* with a unified diff. This dramatically reduces output token count and cost, compared to verbose explanations. Narrative can be requested in a separate, cheaper follow-up query.
*   **Repo Map Context**: Instead of dumping entire files into the prompt (expensive), we provide a `repo_map`—a compact, tree-sitter-generated summary of the repository structure and key symbols. This gives the LLM a bird's-eye view for navigation without bloating the context.
*   **`NO_CHANGES` Sentinel**: To prevent the LLM from generating conversational filler when no code changes are needed (e.g., "It looks like the code is already correct!"), we instruct it to reply with the exact string `NO_CHANGES`. This results in a near-zero token output for those cycles.
*   **Explicit Test-Retry Loop**: The system prompt for TDD explicitly tells the model its role in the test-fix-retest loop, including the maximum number of retries. This structures the autonomous debugging process.

---

## 6 Cost & Budget Guardrails

Our budget is the project's most critical constraint. The entire architecture is optimized to stay within the monthly cap.

| Action              | Tokens (avg)      | \$ at Gemini 2.5 Pro            |
| ------------------- | ----------------- | ------------------------------- |
| Single prompt cycle | IN 5 k, OUT 1.2 k | \$0.006 + \$0.012 = **\$0.018** |
| 150 cycles / month  |                   | **≈ \$2.70**                    |

Margin absorbed by occasional big contexts, image prompts. Hard cap enforced by `utils/budget.py`.

*Policy:* When 90 % of cap reached session becomes **read-only** (explain/test only). This prevents budget overruns while keeping the assistant useful for queries.

---

## 7 Security & Safety Model

| Threat                  | Mitigation                                       | Status           |
| ----------------------- | ------------------------------------------------ | ---------------- |
| Arbitrary shell exec    | diff allow-list; `safe_exec` sandbox for tests (e.g. Docker) | ready in M1      |
| Token leak via code     | `strip_secrets()` before prompt, redaction regex | stub implemented |
| LLM prompt injection    | escape HTML tags in repo excerpts                | done             |
| Remote WebSocket hijack | JWT + origin check; rotate token                 | M3               |
| CPU runaway tests       | `pytest --timeout=15`; process killed after 3x   | M1               |

---

## 8 Directory Layout (definitive)

```
project-x/
  prompt/            ← text templates
  voice/             ← capture + whisper
  index/             ← chunker, embeddings, vecdb/
  llm/               ← provider_base.py, gemini_client.py
  orchestrator/      ← build_prompt.py, budget.py, history.py
  hooks/             ← auto_test.py, screenshot.py
  edit/              ← apply_patch.py, diff_utils.py
  utils/             ← strip_secrets.py, cli.py
  demo/              ← demo_voice_cli.sh, sample_wav/
  tests/             ← pytest suite
  docs/              ← ws.md, api.md
  third_party/       ← upstream prompt files
```

---

## 9 Environment Variables

| Var                  | Default                          | Description                               |
| -------------------- | -------------------------------- | ----------------------------------------- |
| `GEMINI_API_KEY`     | –                                | Required for Gemini 2.5 Pro.              |
| `GEMINI_MODEL`       | `gemini-2.5-pro`                 | Override model name for different providers. |
| `WHISPER_MODEL_PATH` | `~/.cache/ggml-base.en.q5_0.bin` | Path to whisper.cpp GGML model file.      |
| `BUDGET_CAP`         | `200`                            | USD monthly ceiling for API spend.        |
| `JWT_SECRET`         | random                           | Secret for signing WebSocket auth tokens. |
| `PROJECTX_DEBUG`     | `0`                              | Set to `1` for verbose logging.           |

---

## 10 Contributing & Coding Standards

*   **Black 23.12**, line length 100.
*   **Ruff** – select `ALL`, ignore `C901`, `PLR0913`.
*   **Tests** – every module ≥ 90 % coverage (`pytest –cov`).
*   **Commits** – Conventional Commits; squash-merge PRs.
*   **Docs** – update `CHANGELOG.md` + bump semver.

---

## 11 Success Metrics (MVP)

| Metric                                        | Target                 |
| --------------------------------------------- | ---------------------- |
| Demo script passes **green tests** unattended | 100 %                  |
| Average tokens / prompt                       | IN ≤ 8 k, OUT ≤ 2 k    |
| API spend in CI run                           | ≤ \$0.10               |
| Bugs fixed by auto-retry w/out user           | ≥ 80 % of failing runs |
| End-to-end latency (voice → diff commit)      | ≤ 15 s                 |

---

## 12 Open Questions / Parking Lot

1.  Function-calling spec for Gemini (ETA?). Is it more token-efficient than our structured text approach?
2.  Storing binary artefacts (videos, screenshots) – S3 vs local? Local is simpler for MVP.
3.  GPU path for local `llama.cpp` fallback? How to manage CUDA dependencies?
4.  Plugin marketplace for retrieval tools (e.g., custom database connectors)?
5.  Optimal local vector DB for our use case: `LanceDB` vs. `Qdrant` vs. `Chroma`?
6.  Best strategy for video stream analysis: frame-by-frame OCR vs. future multi-modal video models?
7.  How to structure long-term memory (Honcho-style) for effective retrieval without bloating context?

> Add ideas + names; prune monthly.

---

## 13. Implementation Wisdom & Recipes

To avoid reinventing the wheel and to stand on the shoulders of giants, we will directly adopt proven patterns from successful open-source tools. This is our "cheat sheet" for building complex components quickly.

| Component                  | Recipe to Fork                                                               | Why It's the Right Choice                                                                                                   |
| -------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **AST Indexing**           | `Aider`'s `aider/coders/repo_map.py`                                          | A concise (~600 LOC), pure Python implementation of a `tree-sitter` walker that produces a token-efficient repository map.       |
| **RAG Pipeline**           | `Cursor`'s open-source blog posts on `LanceDB`                               | Provides a complete, working example of plugging `tree-sitter` AST chunks into a fast, local vector search database.      |
| **Prompt Engineering**     | `Aider` & `Claude Code` documentation                                        | They publish the exact, battle-tested prompt wording for diff-only responses and safety guardrails. We copy it verbatim.   |
| **Autonomous TDD Loop**    | `Ready-Set-Cloud`'s TDD-with-AI article                                      | Demonstrates the simple but powerful shell pattern of `run test -> on fail, feed traceback to LLM -> repeat`.            |
| **GUI/OS Control** (M4+)   | `Open Interpreter`'s OS Mode (`oi/os_mode/*`)                                | A minimal Python layer for capturing screenshots and dispatching clicks, serving as a scaffold for our observability hook. |
| **Spec-First Prompting**   | `gpt-engineer`'s `prompts/` directory                                        | Contains excellent examples of breaking down a high-level goal into a spec, and then using that spec to generate code.    |

---

## 14. Phased & Parallelized Implementation Plan

This section reframes the project milestones into a dependency graph, identifying a **Core Thread** of essential, serial tasks and multiple **Parallel Tracks** that can be developed concurrently by independent agents to minimize merge conflicts.

### Milestone 0: Bootstrap (Serial Foundation)

This entire phase is serial and foundational. All subsequent work depends on it.

-   \[ ] **Task 0.1**: Initialize `git`, `README.md`, `DESIGN.md`, `.gitignore`.
-   \[ ] **Task 0.2**: Set up `pyproject.toml` with `ruff`, `black`, `pytest`.
-   \[ ] **Task 0.3**: Create the directory structure from Section 8 with `__init__.py` files.
-   \[ ] **Task 0.4**: Implement the basic CI workflow in `.github/workflows/ci.yml` with linting and a passing dummy test.

---

### Milestone 1: Voice → Code (First Vertical Slice)

Once M0 is complete, M1 can be broken down into parallel tracks that converge at the end.

*   **Core Thread (Orchestration & Integration)**
    -   \[ ] **Task 1.C1**: Define the abstract base classes and interfaces in `llm/provider_base.py` and for the voice/test hooks. This unblocks the parallel tracks.
    -   \[ ] **Task 1.C2**: Implement the main CLI entry point and orchestration logic in `orchestrator/` that pieces together the components.
    -   \[ ] **Task 1.C3**: Integrate the deliverables from the parallel tracks (`gemini_client`, `voice`, `autotest`, `patch`).
    -   \[ ] **Task 1.C4**: Create the end-to-end `demo_voice_cli.sh` script to test the fully integrated slice.

*   **Parallel Track 1.1: LLM Client** (Works in `llm/`, `utils/`)
    -   \[ ] **Task 1.P1.1**: Implement `llm/gemini_client.py` against the `LLMProvider` interface.
    -   \[ ] **Task 1.P1.2**: Implement the `utils/budget.py` module. It can be unit-tested in isolation.

*   **Parallel Track 1.2: Voice Input** (Works in `voice/`)
    -   \[ ] **Task 1.P2.1**: Implement `voice/capture.py` for mic input.
    -   \[ ] **Task 1.P2.2**: Implement `voice/transcribe.py` to wrap `whisper.cpp`. Can be tested independently.

*   **Parallel Track 1.3: TDD Tooling** (Works in `hooks/`, `edit/`)
    -   \[ ] **Task 1.P3.1**: Implement `hooks/autotest.py` to run `pytest`. Can be tested against a dummy project.
    -   \[ ] **Task 1.P3.2**: Implement `edit/patch.py` to apply diffs. Can be tested with static diff files.

---

### Milestone 2: Context Optimization (RAG)

This entire milestone is a self-contained parallel track that can be developed as soon as the interfaces from M1 are stable. The only serial step is the final integration.

*   **Core Thread (Integration)**
    -   \[ ] **Task 2.C1**: Modify `orchestrator/prompt_builder.py` to switch from hardcoded context to using the RAG system developed in the parallel track.

*   **Parallel Track 2.1: RAG System** (Works in `index/`, `scripts/`)
    -   \[ ] **Task 2.P1.1**: Implement `index/ast_chunker.py` using `tree-sitter`.
    -   \[ ] **Task 2.P1.2**: Implement `index/vector_store.py` with `lancedb`.
    -   \[ ] **Task 2.P1.3**: Implement `index/embedder.py`.
    -   \[ ] **Task 2.P1.4**: Build and test the complete pipeline with `scripts/index_repo.py` and `scripts/benchmark_rag.py`.

---

### Milestone 3: Mobile Companion

Similar to M2, this is highly parallelizable. The mobile app can be developed entirely independently of the core application, as long as it adheres to the defined API.

*   **Core Thread (Backend Server)**
    -   \[ ] **Task 3.C1**: Implement the WebSocket server in `orchestrator/server.py`.
    -   \[ ] **Task 3.C2**: Add JWT authentication to the server.
    -   \[ ] **Task 3.C3**: Document the final API in `docs/mobile_api.md`.

*   **Parallel Track 3.1: Mobile App** (Works in `mobile/`)
    -   \[ ] **Task 3.P1.1**: Set up the React Native project.
    -   \[ ] **Task 3.P1.2**: Implement the push-to-talk UI, audio recording, and WebSocket client logic.
    -   \[ ] **Task 3.P1.3**: Test against a mock WebSocket server or the live backend once available.

---

### Milestone 4 & 5: Advanced Features (Observability & Memory)

These later milestones also follow the same pattern, allowing for feature modules to be built in parallel.

*   **Core Thread (Integration)**
    -   \[ ] **Task 4.C1**: Integrate visual feedback hooks into the main TDD loop in the orchestrator.
    -   \[ ] **Task 5.C1**: Integrate the long-term memory module into the prompt builder.
    -   \[ ] **Task 5.C2**: Integrate the Git client into the final stage of the TDD loop.

*   **Parallel Track 4.1: Visual Toolkit** (Works in `hooks/`, `utils/`)
    -   \[ ] **Task 4.P1.1**: Implement `hooks/screenshot.py` and `hooks/video.py`.
    -   \[ ] **Task 4.P1.2**: Implement the `utils/ocr.py` fallback. Can be developed and tested in isolation.

*   **Parallel Track 5.1: Memory System** (Works in `orchestrator/memory.py`)
    -   \[ ] **Task 5.P1.1**: Implement the SQLite-based memory store and retrieval logic. Can be unit-tested independently.

*   **Parallel Track 5.2: Git Workflow Client** (Works in `utils/git_client.py`)
    -   \[ ] **Task 5.P2.1**: Implement the wrapper for `git` commands. Can be tested against a temporary local repository.

*EOF*
