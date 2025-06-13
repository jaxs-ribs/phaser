# Project X Agent Orchestrator Entrypoint

**Objective**: This document is the master instruction set for the Project X Orchestrator LLM. Read this file and execute the development plan by dispatching containerized, autonomous agents for each task.

## Orchestration Protocol

Your primary function is to manage the development lifecycle based on this document.

1.  **Execution Environment**: Each agent task **MUST** be executed in a fresh, containerized environment (e.g., using Docker). The container should be initialized with the latest state of the `main` branch before the task begins. This ensures task isolation and prevents state leakage.

2.  **Task Completion & Deliverable**: The sole deliverable for each completed agent task is a **Pull Request (PR)** against the `main` branch. The PR description must link back to the specific task definition in this `ENTRYPOINT.md` file.

3.  **Process Waves Sequentially**: The plan is divided into "Waves". You must complete all tasks within a wave (including merging all PRs) before proceeding to the next wave.

4.  **Dispatch Parallel Tasks**:
    *   Within each wave, you will find "Parallel Track" tasks. You are to spin up a containerized agent for **each** of these tasks **simultaneously**.
    *   Each agent works on its task, and upon completion, opens a PR. Because their work is restricted to different directories, there will be no merge conflicts.

5.  **Merge & Integrate (Batch Merge Strategy)**:
    *   **Collect all PRs**: Wait for all parallel agents in a wave to open their respective PRs. Do not merge any of them yet.
    *   **Verify CI**: Ensure that the CI checks on every single one of these PRs are passing.
    *   **Sequential Merge**: Once all parallel PRs are open and verified, merge them into `main` sequentially. Given the directory separation, these merges should be conflict-free.
    *   **Execute Core Thread**: Only after all parallel PRs for the wave are successfully merged, dispatch an agent to execute the "Core Thread" integration task(s), which will also result in a PR. Merge this final PR once its CI checks pass.

6.  **Advance to Next Wave**: Once the Core Thread PR for a wave is merged into `main`, you may proceed to the next wave.

---

## Wave 0: Bootstrap (Foundation)

**Action**: Execute these tasks sequentially. For each task, spin up a containerized agent, have it perform the work, and create a PR. Merge the PR before starting the next task.

### Agent Task 0.1: Initialize Repository
-   **Goal**: Initialize the git repository and create the foundational documentation and gitignore files.
-   **Context**: Root directory.
-   **Instructions**:
    1.  Run `git init`.
    2.  Create `README.md` and `DESIGN.md`.
    3.  Create a language-agnostic `.gitignore` file.
    4.  Create the full project directory structure.
-   **Definition of Done (DoD)**: A PR is opened containing the specified files and directories. CI should pass (trivial at this stage).

### Agent Task 0.2: Setup CI
-   **Goal**: Create a CI pipeline that lints and tests the project.
-   **Context**: `.github/workflows/`, `tests/`
-   **Instructions**:
    1.  Create `.github/workflows/ci.yml`. It should run a basic test command.
    2.  Create `tests/placeholder.test.sh` that exits with code 0.
    3.  The `ci.yml` file should execute this test script.
-   **Definition of Done (DoD)**: A PR is opened. The GitHub Action defined in the PR must run and pass successfully.

---

## Wave 1: Voice → Code (First Vertical Slice)

**Action**:
1.  First, dispatch a containerized agent for **Task 1.C1** to define interfaces and merge its PR.
2.  Then, simultaneously spin up containerized agents for all Parallel Tracks (**1.1, 1.2, 1.3**).
3.  Wait for all three agents to create their PRs and for CI to pass on all of them. Merge all three PRs.
4.  Finally, dispatch an agent for the Core Thread integration tasks and merge its PR.

### Core Thread Task 1.C1: Define Interfaces
-   **Goal**: Create the abstract API contracts to enable parallel work.
-   **Context**: `llm/`, `hooks/`, `voice/`.
-   **Instructions**:
    1.  In `llm/`, define an `LLMProvider` interface with methods for completions.
    2.  In `hooks/`, define the function signature for `autotest`.
    3.  In `voice/`, define the signatures for audio capture/transcription.
-   **Definition of Done (DoD)**: A PR is opened with placeholder files containing the specified interfaces. CI must pass.

### Parallel Track 1.1: LLM Client
-   **Agent Task Title**: Implement Gemini LLM Client and Budget Tracker
-   **Goal**: Create a working client for the Gemini API and a utility to track token usage.
-   **Context**: `llm/`, `utils/`.
-   **Instructions**:
    1.  Implement `llm/gemini_client` conforming to the `LLMProvider` interface.
    2.  Implement `utils/budget` to calculate cost based on tokens.
-   **Definition of Done (DoD)**: A PR is opened. `tests/test_llm.sh` is created and passes in CI, using a mock API to verify the client and budget tracker.

### Parallel Track 1.2: Voice Input
-   **Agent Task Title**: Implement Voice Capture and Transcription
-   **Goal**: Create a module that can capture microphone audio and transcribe it.
-   **Context**: `voice/`.
-   **Instructions**:
    1.  Implement `voice/capture`.
    2.  Implement `voice/transcribe` wrapping a speech-to-text engine.
-   **Definition of Done (DoD)**: A PR is opened. `tests/test_voice.sh` is created and passes in CI, using a pre-recorded `.wav` file to test the transcriber.

### Parallel Track 1.3: TDD Tooling
-   **Agent Task Title**: Implement Test Execution and Code Patching Tools
-   **Goal**: Create the tools for the autonomous TDD loop.
-   **Context**: `hooks/`, `edit/`.
-   **Instructions**:
    1.  Implement `hooks/autotest` to execute a test command.
    2.  Implement `edit/patch` to safely apply a unified diff.
-   **Definition of Done (DoD)**: A PR is opened. `tests/test_tools.sh` is created and passes in CI, verifying both the autotest hook and the patch tool against sample files.

### Core Thread Integration (Post-Parallel)
-   **Agent Task Title**: Integrate Voice-to-Code Components
-   **Goal**: Assemble the parallel components into the first working end-to-end product.
-   **Context**: `orchestrator/`, `demo/`.
-   **Instructions**:
    1.  Implement the core logic in `orchestrator/` that uses all the previously merged components.
    2.  Create the `demo/demo_voice_cli.sh` script that runs the full loop.
-   **Definition of Done (DoD)**: A PR is opened. The `demo_voice_cli.sh` script must run successfully in CI and result in a "tests passed" state.

---

## Wave 2: Context Optimization (RAG)

**Action**:
1.  Simultaneously spin up one containerized agent for the entire "Parallel Track 2.1".
2.  Wait for its PR, ensure CI passes, and merge it.
3.  Dispatch a final agent for the "Core Thread Task 2.C1" integration.

### Parallel Track 2.1: RAG System
-   **Agent Task Title**: Build the Tree-sitter RAG Pipeline
-   **Goal**: Implement a complete RAG system for intelligent context retrieval.
-   **Context**: `index/`, `scripts/`.
-   **Instructions**:
    1.  Implement the `ast_chunker`.
    2.  Implement the `vector_store`.
    3.  Implement the `embedder`.
    4.  Create `scripts/index_repo.py` to build an index.
-   **Definition of Done (DoD)**: A PR is opened. `tests/test_rag.sh` is created and passes in CI, verifying the full pipeline. `scripts/benchmark_rag.py` is also included.

### Core Thread Integration (Post-Parallel)
-   **Agent Task Title**: Integrate RAG into Orchestrator
-   **Goal**: Upgrade the orchestrator to use the new RAG pipeline.
-   **Context**: `orchestrator/`.
-   **Instructions**: Modify the `prompt_builder` logic to fetch context from the RAG system.
-   **Definition of Done (DoD)**: A PR is opened. The end-to-end demo script from Wave 1 must still pass, and logs in CI should confirm that context is now being retrieved from the RAG system. 