# Project-X – Hands-On Showcase ✨

Below are **copy-paste ready** commands that demonstrate the most interesting flows of Project-X.
Each block tells you *exactly* what will happen when you run it.

---
## 0. Build & verify everything
```bash
# (in project root)
$ cargo build            # compile all crates / binaries
$ cargo test -q          # 17 unit-tests should pass
```

---
## 1. Autonomous TDD loop – in a disposable sandbox
Generates code changes, runs the test-suite, retries until green, **without touching your repo**.
```bash
$ cargo run --bin project-x --release \
        -- --prompt "Add hello_world() to src/lib.rs" \
        --sandbox --dry-run
```
What you'll see:
* a new `./sandboxes/run_yyyy-mm-dd_HH-MM-SS` folder
* the AI proposes a unified diff (dry-run ➜ no files modified)
* cargo test is executed inside the sandbox

Remove `--dry-run` to actually write the patch.  Add `--keep-sandbox` to inspect the result afterwards.

---
## 2. Quick Gemini sanity check
Ask the LLM something short; prints raw answer and shows API-usage counters.
```bash
$ cargo run --bin project-x --release -- --test-llm "explain the builder pattern"
```

---
## 3. Voice-to-code pipeline
Record **5 s** of microphone audio, transcribe it, then ask Gemini for a code suggestion.
```bash
$ cargo run --bin project-x --release -- \
        --duration 5 --output temp.wav
```
Requires `ffmpeg` on your PATH and a working mic.

---
## 4. Git helper demo
Initialise a throw-away repo, stage a file, commit it and show a smart commit message suggestion.
```bash
# tmp dir so we don't pollute anything
$ tmp=$(mktemp -d) && cd "$tmp"
$ git init -q && echo "Hello" > README.md
# run the demo bin
$ cargo run --bin git-demo
```
Sample output includes:
```
➕ Added to staging: README.md
✅ Created commit: e4a1f2b3 - Initial commit
Suggested message: Update documentation: README.md
```

---
## 5. Memory manager demo
Stores a short conversation then prints a nicely formatted context string.
```bash
$ cargo run --bin memory-demo
```

---
## 6. Indexer helper
Scans the workspace, lists large / relevant files – handy inside the autonomous loop.
```bash
$ cargo run --bin indexer -- --help       # see options
$ cargo run --bin indexer                # run with defaults
```

---
### Tips
* Export `GEMINI_API_KEY` **or** `OPENROUTER_API_KEY` before running any LLM examples.
* Add `--dry-run` to almost any command to prevent file changes.
* All extra binaries live in `src/bin/`; read them to understand the APIs quickly.

Enjoy exploring Project-X! 🚀 