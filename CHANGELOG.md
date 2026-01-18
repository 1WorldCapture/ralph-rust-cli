# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.6] - 2026-01-18

### Fixed
- Fixed claude provider: add `--verbose` flag required for `--output-format=stream-json` with print mode

## [0.2.5] - 2026-01-15

### Fixed
- Fixed codex provider: use `--yolo` flag instead of `--sandbox` which now requires a value

## [0.2.4] - 2026-01-15

### Fixed
- Fixed version number not being updated in Cargo.toml for releases
- Use `codex --yolo` flag for non-interactive mode

## [0.2.2] - 2026-01-15

### Fixed
- `ralph upgrade` now points to the correct GitHub repository

## [0.2.1] - 2026-01-15

### Fixed
- Cross-compilation for aarch64-linux-gnu by switching from OpenSSL to rustls

## [0.2.0] - 2026-01-15

### Added
- `ralph once` command for single AI provider execution
- `ralph loop` command with iteration control and COMPLETE marker detection
- `ralph upgrade` command for self-updating via GitHub Releases
- Multi-provider support: droid, codex, claude, gemini
- Customizable system prompt at `~/.Ralph/system-prompt.md`
- GitHub Actions workflow for automated cross-platform releases

### Changed
- Improved README documentation with clearer overview and installation instructions

## [0.1.0] - 2026-01-14

### Added
- Initial Rust CLI structure with clap for argument parsing
- `ralph --version` flag to display current version
- `ralph version` subcommand for version display
- Basic project setup with SemVer versioning
