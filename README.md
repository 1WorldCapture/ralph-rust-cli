# Ralph CLI

A Rust CLI dispatcher for AI provider agents (droid, codex, claude, gemini) that streamlines task-driven development workflows using `bd` (beads) for issue tracking.

## Overview

Ralph replaces the legacy bash entrypoints (`scripts/ralph-once.sh`, `scripts/ralph-loop.sh`) with a single, distributable Rust binary. It acts as a unified dispatcher that:

- **Dispatches tasks** to external AI CLI tools (droid, codex, claude, gemini)
- **Injects system prompts** that guide agents to follow a structured workflow using `bd`
- **Supports single-run and loop modes** for different automation needs
- **Self-upgrades** via `ralph upgrade`

## Installation

### From Source (Cargo)

```bash
git clone https://github.com/lyonbot/ralph-cli.git
cd ralph-cli
cargo build --release
# Binary available at target/release/ralph
```

### Manual Installation

Copy the built binary to a location in your `$PATH`:

```bash
cp target/release/ralph ~/.local/bin/
# or
sudo cp target/release/ralph /usr/local/bin/
```

## Usage

### Display Version

```bash
ralph --version
ralph version
```

### Single Execution (`once`)

Execute a single AI provider call, equivalent to the old `ralph-once.sh`:

```bash
# Use default provider (droid)
ralph once

# Specify a provider
ralph once --provider claude
ralph once --provider gemini
ralph once --provider codex
```

### Loop Execution (`loop`)

Run the AI provider in a loop until completion or iteration limit, equivalent to `ralph-loop.sh`:

```bash
# Use defaults (droid, 10 iterations)
ralph loop

# Custom iterations
ralph loop --iterations 5

# Custom provider and iterations
ralph loop --provider claude --iterations 20
```

The loop terminates early if the AI output contains `<promise>COMPLETE</promise>`. After completion (or reaching the iteration limit), ralph automatically runs `bd list --pretty` to display the task status.

### Self-Upgrade

Upgrade ralph to the latest released version:

```bash
ralph upgrade
```

If permission is denied (e.g., binary installed in `/usr/local/bin`), you may need elevated privileges:

```bash
sudo ralph upgrade
```

## Configuration

### System Prompt

On first run, ralph creates a configuration directory and default system prompt:

- **Config directory:** `~/.Ralph/`
- **System prompt file:** `~/.Ralph/system-prompt.md`

You can edit `system-prompt.md` to customize the instructions sent to AI providers. Changes take effect immediately without recompiling.

**Default system prompt** instructs the AI to:

1. Use `bd ready` to find available tasks
2. Claim tasks with `bd update <id> --status in_progress`
3. Implement according to acceptance criteria
4. Run quality gates (build, lint, test)
5. Commit changes and close tasks

## Supported Providers

| Provider | Description |
|----------|-------------|
| `droid`  | Factory Droid AI agent (default) |
| `codex`  | OpenAI Codex CLI |
| `claude` | Anthropic Claude CLI |
| `gemini` | Google Gemini CLI |

Each provider is invoked with specific flags optimized for autonomous operation. See the source code for exact command arguments.

## Requirements

- **bd (beads):** Task tracking CLI - must be installed and available in `$PATH`
- **AI Provider CLI:** At least one of droid, codex, claude, or gemini must be installed

## Development

### Build

```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Test

```bash
cargo test
```

### Project Structure

```
ralph-cli/
├── src/
│   ├── main.rs     # CLI entry point and command handling
│   └── upgrade.rs  # Self-upgrade functionality
├── scripts/        # Legacy bash scripts (for reference)
├── tasks/          # PRD and technical documentation
├── Cargo.toml      # Rust dependencies and metadata
└── AGENTS.md       # Agent instructions for this repo
```

## Contributing

1. Check for available tasks: `bd ready`
2. Claim a task: `bd update <id> --status in_progress`
3. Implement the feature/fix following the acceptance criteria in the task
4. Run quality gates: `cargo build && cargo test`
5. Commit with a descriptive message
6. Close the task: `bd close <id>`

See [AGENTS.md](AGENTS.md) for detailed contribution workflow.

## License

MIT License - see [Cargo.toml](Cargo.toml) for details.
