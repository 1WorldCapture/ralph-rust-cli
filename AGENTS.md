# Agent Instructions 

## Project Basics

**What this repo is:** `ralph-cli` — a Rust CLI named `ralph` that replaces the legacy bash entrypoints in `scripts/` and acts as a dispatcher for external AI Provider CLIs.

**Source of truth:** Always read and implement against the PRD at `tasks/prd.md` (especially the user stories US-01..US-05). Every implementation task should reference the PRD; if anything is unclear, create a beads issue for clarification before coding.

**Legacy entrypoints (behavior to match):**
- `scripts/ralph-once.sh` (single run)
- `scripts/ralph-loop.sh` (loop run)

## `bd`

This project uses **bd** (beads) for issue tracking.

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

Pls. run `bd --help`/`bd quickstart`/`bd onboard` for further information.

## Landing the Plane (Session Completion)

If you pick up a task from bd, pls. follow these instructions for landing. For any other types of user prompt, you just ignore these.

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git commit` (no remote) or `git push` (origin exists) succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY if origin exists:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git commit` or `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
