# Ralph CLI

A Rust CLI dispatcher for AI coding agents that streamlines task-driven development workflows.

## Overview

Ralph is a unified command-line tool designed for developers and teams who want to automate coding tasks using AI agents. It provides:

- **Multi-provider support** — Dispatch tasks to your preferred AI CLI (droid, codex, claude, gemini)
- **Task-driven workflow** — Integrates with `bd` (beads) for structured issue tracking and autonomous task execution
- **Flexible execution modes** — Single-run for one-off tasks, loop mode for iterative autonomous development
- **System prompt injection** — Customizable instructions that guide AI agents to follow your team's workflow
- **Self-upgrade capability** — Keep ralph up-to-date with `ralph upgrade`

**Target audience:** Developers, DevOps engineers, and teams looking to integrate AI coding assistants into their development pipelines with structured, reproducible workflows.

## Installation

### From Source (Cargo)

```bash
git clone https://github.com/lyonbot/ralph-cli.git
cd ralph-cli
cargo build --release
# Binary available at target/release/ralph
```

### Install via Cargo

If you have the source code locally, you can install directly to `~/.cargo/bin`:

```bash
cargo install --path .
```

This compiles and installs the `ralph` binary in one step. Ensure `~/.cargo/bin` is in your `$PATH`.

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
