# PHASE 0: Bootstrap (Rust Edition)

This phase focuses on setting up the foundational structure of a Rust-based repository. The tasks are designed to be completed in parallel.

## Task 0.1: Initialize `Cargo.toml`

**Goal:** Create a `Cargo.toml` file to define the Rust project (crate), manage dependencies, and configure project settings. This file is the manifest for any Rust project.

**File to Create:** `Cargo.toml`

**Instructions:**

1.  Create a new file named `Cargo.toml` at the root of the repository.
2.  Add the `[package]` section with the following information:
    *   `name = "project-x"`
    *   `version = "0.1.0"`
    *   `edition = "2021"`
    *   `description = "AI-First Coding Assistant (Rust Edition)"`
    *   `authors = ["Your Name <your.email@example.com>"]`
3.  Add a `[dependencies]` section. For now, it can be empty or include a placeholder if desired. We will add dependencies in subsequent phases.
4.  Add a `[[bin]]` section to define the main executable:
    *   `name = "project-x"`
    *   `path = "src/main.rs"`

**Acceptance Criteria:** A valid `Cargo.toml` file exists with all the specified configurations. Running `cargo check` should pass.

---

## Task 0.2: Create `.gitignore` for Rust

**Goal:** Create a `.gitignore` file to prevent common temporary files, build artifacts, and local configurations from being committed to the repository.

**File to Create:** `.gitignore`

**Instructions:**

1.  Create a new file named `.gitignore` at the root of the repository.
2.  Add entries for common Rust, OS, and IDE files. A good starting point can be found at [github/gitignore/Rust.gitignore](https://github.com/github/gitignore/blob/main/Rust.gitignore).
3.  Include the following specific patterns:
    ```
    # Build artifacts
    /target/
    
    # Project-specific
    .idea/
    .vscode/
    *.db
    *.sqlite3
    /index/vecdb/
    /voice_recordings/
    ```

**Acceptance Criteria:** The `.gitignore` file is present and contains the specified patterns for a Rust project.

---

## Task 0.3: Set up `src` and `tests` directories

**Goal:** Create the initial directory structure for the source code and test suite, following Rust conventions.

**Files to Create:**
*   `src/main.rs`
*   `src/lib.rs`
*   `tests/cli.rs`

**Instructions:**

1.  Create a directory named `src` at the root of the repository. This is where all source code lives in a Rust project.
2.  Create `src/main.rs` for the main application binary. It should contain a simple "Hello, world!" function.
    ```rust
    fn main() {
        println!("Hello, world!");
    }
    ```
3.  Create `src/lib.rs` for the library part of the crate. It can be empty for now. This is where the core logic will be built.
4.  Create a `tests` directory for integration tests.
5.  Create a file `tests/cli.rs` with a single passing test.
    ```rust
    #[test]
    fn test_ci_is_working() {
        assert_eq!(true, true);
    }
    ```

**Acceptance Criteria:** The `src` and `tests` directories and specified files are created. Running `cargo run` should print "Hello, world!", and `cargo test` should pass.

---

## Task 0.4: Set up CI with GitHub Actions for Rust

**Goal:** Create a GitHub Actions workflow to automatically format, lint, and test the code on every push and pull request.

**File to Create:** `.github/workflows/ci.yml`

**Instructions:**

1.  Create a directory `.github/workflows` at the root of the repository.
2.  Create a file named `ci.yml` inside that directory.
3.  Define a workflow that triggers on `push` and `pull_request` events for the `main` branch.
4.  The workflow should have jobs for:
    *   **Formatting:** Runs `cargo fmt -- --check` to ensure code is formatted correctly.
    *   **Linting:** Runs `cargo clippy -- -D warnings` to catch common mistakes and enforce idioms.
    *   **Testing:** Runs `cargo test --all-features` to execute the test suite.
5.  Use a pre-made action like `actions-rs/toolchain` to set up the Rust environment.

**Acceptance Criteria:** A `ci.yml` workflow file is present. When pushed to GitHub, the actions should trigger and pass successfully for the initial project state. 