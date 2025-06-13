#!/bin/bash

# Project-X Development Session
# Creates a comprehensive tmux session for Rust development

SESSION_NAME="project-x-dev"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Check if tmux is installed
if ! command -v tmux &> /dev/null; then
    echo "❌ tmux is not installed. Please install it first:"
    echo "   macOS: brew install tmux"
    echo "   Ubuntu: sudo apt-get install tmux"
    exit 1
fi

# Check if session already exists
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
    echo "📺 Session '$SESSION_NAME' already exists. Attaching..."
    tmux attach-session -t "$SESSION_NAME"
    exit 0
fi

echo "🚀 Starting Project-X development session..."
echo "📁 Project root: $PROJECT_ROOT"

# Create new session
cd "$PROJECT_ROOT"
tmux new-session -d -s "$SESSION_NAME" -x 120 -y 40

# Main window - Editor/Primary development
tmux rename-window -t "$SESSION_NAME:0" "main"
tmux send-keys -t "$SESSION_NAME:main" "clear && echo '🏠 Main Development Window'" Enter
tmux send-keys -t "$SESSION_NAME:main" "echo '📝 Open your editor here (code ., vim ., etc.)'" Enter

# Split main window - right side for quick commands
tmux split-window -t "$SESSION_NAME:main" -h -p 35
tmux send-keys -t "$SESSION_NAME:main.1" "clear && echo '⚡ Quick Commands'" Enter
tmux send-keys -t "$SESSION_NAME:main.1" "echo '• cargo build'" Enter
tmux send-keys -t "$SESSION_NAME:main.1" "echo '• cargo test'" Enter
tmux send-keys -t "$SESSION_NAME:main.1" "echo '• cargo run --bin project-x'" Enter

# Window 1 - Continuous testing
tmux new-window -t "$SESSION_NAME" -n "tests"
tmux send-keys -t "$SESSION_NAME:tests" "clear && echo '🧪 Continuous Testing'" Enter

# Check if cargo-watch is installed
if command -v cargo-watch &> /dev/null; then
    tmux send-keys -t "$SESSION_NAME:tests" "echo '🔄 Starting cargo-watch for tests...'" Enter
    tmux send-keys -t "$SESSION_NAME:tests" "cargo watch -x test" Enter
else
    tmux send-keys -t "$SESSION_NAME:tests" "echo '📦 cargo-watch not found. Install with: cargo install cargo-watch'" Enter
    tmux send-keys -t "$SESSION_NAME:tests" "echo '⚡ Manual testing: cargo test'" Enter
fi

# Window 2 - Voice-to-Code Pipeline Testing
tmux new-window -t "$SESSION_NAME" -n "voice"
tmux send-keys -t "$SESSION_NAME:voice" "clear && echo '🎤 Voice-to-Code Pipeline'" Enter
tmux send-keys -t "$SESSION_NAME:voice" "echo 'Available commands:'" Enter
tmux send-keys -t "$SESSION_NAME:voice" "echo '• cargo run --bin project-x -- --test-llm \"your prompt\"'" Enter
tmux send-keys -t "$SESSION_NAME:voice" "echo '• cargo run --bin project-x -- --show-usage'" Enter
tmux send-keys -t "$SESSION_NAME:voice" "echo '• GEMINI_API_KEY=your_key cargo run --bin project-x'" Enter

# Window 3 - RAG Indexing and Search
tmux new-window -t "$SESSION_NAME" -n "rag"
tmux send-keys -t "$SESSION_NAME:rag" "clear && echo '🔍 RAG Indexing & Search'" Enter
tmux send-keys -t "$SESSION_NAME:rag" "echo 'Available commands:'" Enter
tmux send-keys -t "$SESSION_NAME:rag" "echo '• cargo run --bin indexer -- --store --embeddings'" Enter
tmux send-keys -t "$SESSION_NAME:rag" "echo '• cargo run --bin indexer -- --search \"your query\"'" Enter
tmux send-keys -t "$SESSION_NAME:rag" "echo '• python3 scripts/benchmark_rag.py \"query\"'" Enter

# Split RAG window for memory and git demos
tmux split-window -t "$SESSION_NAME:rag" -v -p 50
tmux send-keys -t "$SESSION_NAME:rag.1" "clear && echo '🧠 Memory & Git Tools'" Enter
tmux send-keys -t "$SESSION_NAME:rag.1" "echo '• cargo run --bin memory-demo'" Enter
tmux send-keys -t "$SESSION_NAME:rag.1" "echo '• cargo run --bin git-demo'" Enter

# Window 4 - System monitoring/logs
tmux new-window -t "$SESSION_NAME" -n "monitor"
tmux send-keys -t "$SESSION_NAME:monitor" "clear && echo '📊 System Monitor'" Enter
tmux send-keys -t "$SESSION_NAME:monitor" "echo '🔧 Git status:'" Enter
tmux send-keys -t "$SESSION_NAME:monitor" "git status --short" Enter

# Split monitor window
tmux split-window -t "$SESSION_NAME:monitor" -v -p 60
tmux send-keys -t "$SESSION_NAME:monitor.1" "clear && echo '📝 Project Structure'" Enter
tmux send-keys -t "$SESSION_NAME:monitor.1" "tree -L 3 -I 'target|.git' || ls -la" Enter

# Split again for resource monitoring
tmux split-window -t "$SESSION_NAME:monitor.1" -h -p 50
tmux send-keys -t "$SESSION_NAME:monitor.2" "clear && echo '⚡ System Resources'" Enter
if command -v htop &> /dev/null; then
    tmux send-keys -t "$SESSION_NAME:monitor.2" "htop" Enter
elif command -v top &> /dev/null; then
    tmux send-keys -t "$SESSION_NAME:monitor.2" "top" Enter
else
    tmux send-keys -t "$SESSION_NAME:monitor.2" "echo 'System monitoring tools not available'" Enter
fi

# Set up key bindings and settings
tmux set-option -t "$SESSION_NAME" mouse on
tmux set-option -t "$SESSION_NAME" history-limit 10000

# Go back to main window
tmux select-window -t "$SESSION_NAME:main"
tmux select-pane -t "$SESSION_NAME:main.0"

# Print session info
echo ""
echo "✅ Development session '$SESSION_NAME' created successfully!"
echo ""
echo "📋 Session Layout:"
echo "   • main     - Primary development window"
echo "   • tests    - Continuous testing with cargo-watch"
echo "   • voice    - Voice-to-code pipeline testing"
echo "   • rag      - RAG indexing and search tools"
echo "   • monitor  - System monitoring and git status"
echo ""
echo "🎯 Quick Navigation:"
echo "   • Ctrl+B + 0-4  - Switch between windows"
echo "   • Ctrl+B + ←→   - Switch between panes"
echo "   • Ctrl+B + \"    - Split pane horizontally"
echo "   • Ctrl+B + %    - Split pane vertically"
echo "   • Ctrl+B + d    - Detach session"
echo ""
echo "🔗 Reconnect later with: tmux attach-session -t $SESSION_NAME"
echo "📱 Or run: ./scripts/start_dev_session.sh"
echo ""

# Attach to the session
echo "🎬 Attaching to session..."
sleep 1
tmux attach-session -t "$SESSION_NAME"