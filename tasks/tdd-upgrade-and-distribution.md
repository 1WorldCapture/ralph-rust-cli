# Technical Design Document: Distribution, Upgrade, and CLI Structure

## Overview

This TDD addresses the open questions from the PRD (Section 6) before implementing US-05 (`ralph upgrade`). It covers:

1. Distribution and upgrade channels
2. Configuration directory structure and future extensibility
3. CLI structure and alias strategy

---

## 1. Distribution and Upgrade Channels

### 1.1 Decision: GitHub Releases as Primary Distribution

**Recommendation:** Use **GitHub Releases** as the primary distribution channel for v1.0.

**Rationale:**
- Simplest to implement and maintain for a new project
- Cross-platform support without additional infrastructure
- Natural integration with the existing repository (`https://github.com/lyonbot/ralph-cli`)
- Users can manually download if `ralph upgrade` fails
- No dependency on third-party package managers

**Future Considerations:**
- Homebrew tap can be added later as a secondary channel
- Cargo publish to crates.io can be added if community adoption grows
- These don't preclude GitHub Releases, they complement it

### 1.2 Release Artifact Strategy

Each GitHub Release should include pre-built binaries for common platforms:

| Platform | Target Triple | Artifact Name |
|----------|---------------|---------------|
| macOS (Intel) | `x86_64-apple-darwin` | `ralph-x86_64-apple-darwin.tar.gz` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `ralph-aarch64-apple-darwin.tar.gz` |
| Linux (x64) | `x86_64-unknown-linux-gnu` | `ralph-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `aarch64-unknown-linux-gnu` | `ralph-aarch64-unknown-linux-gnu.tar.gz` |
| Windows (x64) | `x86_64-pc-windows-msvc` | `ralph-x86_64-pc-windows-msvc.zip` |

Each archive contains:
- `ralph` (or `ralph.exe` on Windows) - the binary
- `ralph.sha256` - SHA256 checksum file

### 1.3 Upgrade Command Implementation

#### 1.3.1 High-Level Flow

```
ralph upgrade
    │
    ├─> GET /repos/{owner}/{repo}/releases/latest  (GitHub API)
    │
    ├─> Compare version (current vs latest)
    │   └─> If same: "Already up to date (v0.1.0)"
    │
    ├─> Select correct asset for platform
    │
    ├─> Download to temp directory
    │
    ├─> Verify SHA256 checksum
    │
    ├─> Replace current binary (self-replace)
    │
    └─> Confirm: "Upgraded from v0.1.0 to v0.2.0"
```

#### 1.3.2 GitHub API Endpoint

```
GET https://api.github.com/repos/lyonbot/ralph-cli/releases/latest
```

Response includes:
- `tag_name`: Version tag (e.g., "v0.2.0")
- `assets[]`: List of downloadable files with `browser_download_url`

**Rate Limiting:**
- Unauthenticated: 60 requests/hour (sufficient for upgrade use case)
- Consider caching "last checked" timestamp to avoid excessive checks

#### 1.3.3 Checksum Verification

**Decision:** Use **SHA256 checksums** for integrity verification.

Rationale:
- Simple to implement and verify
- Sufficient for detecting download corruption
- GPG signing can be added later if supply chain security becomes a priority

Implementation:
1. Download `ralph-{target}.tar.gz.sha256` alongside the binary
2. Compute SHA256 of downloaded archive
3. Compare against checksum file content
4. Abort if mismatch

#### 1.3.4 Self-Replacement Strategy

**Challenge:** A running process cannot directly replace itself on most operating systems.

**Solution (Cross-Platform):**

1. Download new binary to `<temp_dir>/ralph_new`
2. Rename current binary to `<install_dir>/ralph.old`
3. Move new binary to `<install_dir>/ralph`
4. Delete `ralph.old` (best effort, may require next run on Windows)
5. On failure, attempt rollback by restoring `ralph.old`

**Permission Handling:**
- Detect permission errors early (before download)
- Suggest solutions:
  ```
  Error: Cannot write to /usr/local/bin/ralph (permission denied)
  
  Solutions:
  1. Run with elevated permissions: sudo ralph upgrade
  2. Reinstall to a user-writable location: ~/.local/bin/
  3. Download manually from GitHub Releases
  ```

#### 1.3.5 Dependencies for Implementation

New Cargo dependencies needed:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
tempfile = "3"
```

### 1.4 Version Comparison

Use semantic versioning comparison:
- Strip leading "v" from tag (e.g., "v0.2.0" → "0.2.0")
- Compare using semver crate or manual implementation
- Only upgrade if remote version > local version

---

## 2. Configuration Directory Structure

### 2.1 Current Structure

```
~/.Ralph/
└── system-prompt.md       # User-editable system prompt
```

### 2.2 Decision: Reserve Space for Future Provider Configuration

**Recommendation:** Keep configuration minimal for v1.0, but design for extensibility.

**Proposed Future Structure (NOT implemented yet):**

```
~/.Ralph/
├── system-prompt.md       # Default system prompt (exists)
├── config.toml            # Optional: global config (future)
└── providers/             # Optional: provider overrides (future)
    ├── droid.toml
    ├── claude.toml
    └── ...
```

**For v1.0:** 
- Do NOT create `config.toml` or `providers/` directory
- Keep the simple `system-prompt.md` only
- Document the reserved structure in README for future reference

**Rationale:**
- YAGNI (You Aren't Gonna Need It) for v1.0
- Easy to add later without breaking existing installations
- Avoids premature complexity

### 2.3 Provider Configuration (Future Design Notes)

When needed, provider config could support:

```toml
# ~/.Ralph/providers/droid.toml (FUTURE - NOT FOR V1)
[command]
binary = "droid"
subcommand = "exec"
flags = ["--output-format", "stream-json"]

[flags.once]
additional = ["--skip-permissions-unsafe"]

[flags.loop]
additional = ["--auto", "medium"]
```

This allows users to customize provider invocations without modifying ralph source code. **Implementation deferred until there's demonstrated user need.**

---

## 3. CLI Structure and Alias Strategy

### 3.1 Current CLI Structure

```
ralph
├── version          # Show version
├── once             # Single execution (from US-03)
│   └── --provider   # droid|codex|claude|gemini
└── loop             # Loop execution (from US-04)
    ├── --provider   # droid|codex|claude|gemini
    └── --iterations # positive integer
```

### 3.2 Decision: Keep `ralph once/loop`, No Aliases

**Recommendation:** Maintain current structure, do NOT create `ralph-once`/`ralph-loop` aliases.

**Rationale:**
1. **Single binary simplicity**: One executable to install, upgrade, and manage
2. **Modern CLI conventions**: Subcommands are the standard pattern (git, cargo, docker)
3. **Cleaner PATH**: No clutter with multiple symlinks
4. **Easier upgrades**: Only one file to replace

**Migration from Scripts:**
- Users currently using `./scripts/ralph-once.sh` can use `ralph once`
- Document the mapping in README:
  ```
  ralph-once.sh --provider X  →  ralph once --provider X
  ralph-loop.sh --iterations N  →  ralph loop --iterations N
  ```

### 3.3 Alternative Considered: Symbolic Aliases

```bash
# NOT RECOMMENDED - Creates installation complexity
ln -s /usr/local/bin/ralph /usr/local/bin/ralph-once
ln -s /usr/local/bin/ralph /usr/local/bin/ralph-loop
```

If `ralph` detects it was invoked as `ralph-once`, it could behave as `ralph once`. This was rejected because:
- Adds complexity to upgrade (must update symlinks)
- Platform-specific symlink handling (Windows)
- Marginal benefit over explicit subcommand

### 3.4 Future: Default Subcommand

If usage patterns show `ralph once` is the most common command, consider:
```bash
ralph                    # Could default to `ralph once`
ralph -l                 # Short for `ralph loop`
```

**Decision:** Not implemented for v1.0. Gather usage feedback first.

---

## 4. Implementation Plan for US-05

Based on the decisions above, US-05 implementation should:

### 4.1 Add Dependencies
```toml
# Cargo.toml additions
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
tempfile = "3"
```

### 4.2 Add `upgrade` Subcommand
```rust
enum Commands {
    Version,
    Once { ... },
    Loop { ... },
    Upgrade,  // NEW
}
```

### 4.3 Core Functions to Implement

1. `get_latest_release()` - Fetch from GitHub API
2. `compare_versions(current, latest)` - SemVer comparison
3. `download_release(version, target)` - Download with progress
4. `verify_checksum(file, expected)` - SHA256 verification
5. `self_replace(new_binary)` - Atomic binary replacement
6. `suggest_permission_fix()` - User-friendly error messages

### 4.4 Error Handling

| Scenario | User Message |
|----------|--------------|
| Network error | "Failed to check for updates. Check your internet connection." |
| Already latest | "ralph is already up to date (v0.1.0)" |
| Permission denied | See Section 1.3.4 |
| Checksum mismatch | "Download verification failed. Please try again or download manually." |
| GitHub rate limit | "Too many requests. Please try again in an hour." |

---

## 5. Decision Summary

| Question | Decision | Rationale |
|----------|----------|-----------|
| Distribution channel | GitHub Releases | Simple, cross-platform, no infra needed |
| Upgrade mechanism | Self-replacing binary | Best UX, single command upgrade |
| Checksum verification | SHA256 | Sufficient security, simple implementation |
| Provider config | Deferred (YAGNI) | Keep v1.0 simple |
| CLI structure | `ralph once/loop` subcommands | Modern conventions, single binary |
| Symlink aliases | No | Complexity not justified |

---

## 6. Open Items for Future

- GPG signing of releases (if supply chain security becomes a concern)
- Homebrew formula (if macOS adoption grows)
- Cargo publish (if Rust community adoption grows)
- Provider configuration files (when customization is needed)
- Default subcommand behavior (based on usage patterns)

---

## Appendix A: GitHub Release Workflow Example

```yaml
# .github/workflows/release.yml (for future CI/CD)
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          # ... more targets
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release --target ${{ matrix.target }}
      - run: |
          cd target/${{ matrix.target }}/release
          tar czf ralph-${{ matrix.target }}.tar.gz ralph
          shasum -a 256 ralph-${{ matrix.target }}.tar.gz > ralph-${{ matrix.target }}.tar.gz.sha256
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            target/${{ matrix.target }}/release/ralph-${{ matrix.target }}.tar.gz
            target/${{ matrix.target }}/release/ralph-${{ matrix.target }}.tar.gz.sha256
```

---

*Document created: 2025-01-14*
*Last updated: 2025-01-14*
*Author: Technical Design for ralph-cli US-05*
