#!/bin/bash
set -euo pipefail

# Human-in-the-loop Ralph - Run once, watch, iterate
# Supports multiple AI providers: droid, codex, claude, gemini

# Default values
PROVIDER="droid"

# Help message
show_help() {
  cat <<EOF
Usage: ./ralph-once.sh [OPTIONS]

Human-in-the-loop Ralph - Run once, watch, iterate

OPTIONS:
  --provider <name>   AI provider to use (default: droid)
                      Available: droid, codex, claude, gemini
  -h, --help          Show this help message

EXAMPLES:
  ./ralph-once.sh                    # Use default provider (droid)
  ./ralph-once.sh --provider claude  # Use Claude
  ./ralph-once.sh --provider gemini  # Use Gemini
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --provider)
      PROVIDER="$2"
      shift 2
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    *)
      echo "Error: Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

# Validate provider
case "$PROVIDER" in
  droid|codex|claude|gemini)
    ;;
  *)
    echo "Error: Invalid provider '$PROVIDER'"
    echo "Available providers: droid, codex, claude, gemini"
    exit 1
    ;;
esac

PROMPT=$(cat <<'EOF'
Use bd (beads) for task tracking. Follow these steps:

1. Run 'bd ready' to find the next available task (not blocked by dependencies)
2. Run 'bd show <id>' to read the task details and acceptance criteria
3. Run 'bd update <id> --status in_progress' to claim the task
4. Implement the task according to the acceptance criteria. You need to read docs under `tasks` for better understanding of whole context.
5. Run quality gates (bun run build, cargo build if applicable)
6. Commit your changes with a descriptive message
7. Run `bd update <id> ...` to add more content for the issue: requirement/root cause/your design solution/etc.
8. Run 'bd close <id>' to mark the task as complete

IMPORTANT:
- ONLY DO ONE TASK AT A TIME
- Do not start tasks that are blocked (have uncompleted dependencies)
- Verify all acceptance criteria before closing the task
EOF
)

echo "Using AI provider: $PROVIDER"

# Build and execute command based on provider
case "$PROVIDER" in
  droid)
    droid exec --output-format stream-json --skip-permissions-unsafe "$PROMPT"
    ;;
  codex)
    codex exec --full-auto --json "$PROMPT"
    ;;
  claude)
    claude -p --output-format stream-json --dangerously-skip-permissions "$PROMPT"
    ;;
  gemini)
    gemini -p --output-format stream-json --yolo "$PROMPT"
    ;;
esac
