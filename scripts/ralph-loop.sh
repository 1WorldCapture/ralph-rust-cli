#!/bin/bash
set -euo pipefail

# Autonomous Ralph loop - Runs continuously until all tasks are done
# Supports multiple AI providers: droid, codex, claude, gemini

# Default values
PROVIDER="droid"
ITERATIONS=10

# Help message
show_help() {
  cat <<EOF
Usage: ./ralph-loop.sh [OPTIONS]

Autonomous Ralph loop - Runs continuously until all tasks are done

OPTIONS:
  --provider <name>      AI provider to use (default: droid)
                         Available: droid, codex, claude, gemini
  --iterations <count>   Number of iterations (default: 10)
  -h, --help             Show this help message

EXAMPLES:
  ./ralph-loop.sh                                # Use defaults (droid, 10 iterations)
  ./ralph-loop.sh --iterations 5                 # Run 5 iterations with droid
  ./ralph-loop.sh --provider claude              # Use Claude with 10 iterations
  ./ralph-loop.sh --provider gemini --iterations 20  # Use Gemini with 20 iterations
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --provider)
      PROVIDER="$2"
      shift 2
      ;;
    --iterations)
      ITERATIONS="$2"
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

# Validate iterations is a positive integer
if ! [[ "$ITERATIONS" =~ ^[0-9]+$ ]] || [ "$ITERATIONS" -lt 1 ]; then
  echo "Error: iterations must be a positive integer"
  exit 1
fi

PROMPT=$(cat <<'EOF'
Use bd (beads) for task tracking. Follow these steps:

1. Run 'bd ready' to find the next available task (not blocked by dependencies)
2. Run 'bd show <id>' to read the task details and acceptance criteria
3. Run 'bd update <id> --status in_progress' to claim the task
4. Implement the task according to the acceptance criteria. You need to read docs under `tasks` for better understanding of whole context.
5. Run quality gates (bun run build, cargo build if applicable)
6. Commit your changes with a descriptive message
7. Run 'bd close <id>' to mark the task as complete

IMPORTANT:
- ONLY DO ONE TASK AT A TIME
- Do not start tasks that are blocked (have uncompleted dependencies)
- Verify all acceptance criteria before closing the task
- If all tasks are complete or blocked, output <promise>COMPLETE</promise>
EOF
)

echo "Using AI provider: $PROVIDER"
echo "Max iterations: $ITERATIONS"
echo ""

for ((i=1; i<=ITERATIONS; i++)); do
  echo "=========================================="
  echo "Iteration $i / $ITERATIONS"
  echo "=========================================="

  # Build and execute command based on provider
  # Capture output for completion check
  case "$PROVIDER" in
    droid)
      # --auto medium: allows git commit but not push
      result=$(droid exec --auto medium --output-format stream-json "$PROMPT")
      ;;
    codex)
      # --full-auto: non-interactive mode with auto-approval
      # --sandbox: run in sandbox for safety
      result=$(codex exec --full-auto --sandbox --json "$PROMPT")
      ;;
    claude)
      # --dangerously-skip-permissions: skip permission prompts
      result=$(claude -p --output-format stream-json --dangerously-skip-permissions "$PROMPT")
      ;;
    gemini)
      # --yolo: skip permission prompts (equivalent to --approval-mode auto_edit)
      result=$(gemini -p --output-format stream-json --yolo "$PROMPT")
      ;;
  esac

  echo "$result"

  if [[ "$result" == *"<promise>COMPLETE</promise>"* ]]; then
    echo "All tasks complete after $i iterations."
    bd list --pretty
    exit 0
  fi
done

echo ""
echo "Ralph loop finished after $ITERATIONS iterations"
bd list --pretty
