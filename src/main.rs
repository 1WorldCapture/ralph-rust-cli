use clap::Parser;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Default system prompt content (equivalent to script's built-in PROMPT)
const DEFAULT_SYSTEM_PROMPT: &str = r#"Use bd (beads) for task tracking. Follow these steps:

1. Run 'bd ready' to find the next available task (not blocked by dependencies)
2. Run 'bd show <id>' to read the task details and acceptance criteria
3. Run 'bd update <id> --status in_progress' to claim the task
4. Implement the task according to the acceptance criteria. You need to read docs under `tasks` for better understanding of whole context.
5. Run quality gates (bun run build, cargo build if applicable)
6. Commit your changes with a descriptive message
7. Run `bd update <id> ...` to update beads more info. for future references: 
  - Run `bd update <id> --design ...`: update brief design solution summary
  - Run `bd update <id> --notes ...`: update brief summaries of code change or document updates
8. Run 'bd close <id>' to mark the task as complete

IMPORTANT:
- ONLY DO ONE TASK AT A TIME
- Do not start tasks that are blocked (have uncompleted dependencies)
- Verify all acceptance criteria before closing the task
"#;

/// Ralph CLI - A dispatcher for AI provider agents
#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Supported AI providers
const VALID_PROVIDERS: &[&str] = &["droid", "codex", "claude", "gemini"];

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Display version information
    Version,
    /// Execute a single AI provider call (equivalent to ralph-once.sh)
    Once {
        /// AI provider to use (default: droid)
        /// Available: droid, codex, claude, gemini
        #[arg(long, default_value = "droid")]
        provider: String,
    },
}

/// Get the Ralph configuration directory path (~/.Ralph/)
fn get_config_dir() -> io::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine home directory")
    })?;
    Ok(home.join(".Ralph"))
}

/// Get the system prompt file path (~/.Ralph/system-prompt.md)
fn get_system_prompt_path() -> io::Result<PathBuf> {
    Ok(get_config_dir()?.join("system-prompt.md"))
}

/// Ensure the configuration directory and default system prompt file exist.
/// Creates them if they don't exist.
fn ensure_config() -> io::Result<()> {
    let config_dir = get_config_dir()?;

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        eprintln!("Created configuration directory: {}", config_dir.display());
    }

    // Create default system prompt file if it doesn't exist
    let prompt_path = get_system_prompt_path()?;
    if !prompt_path.exists() {
        fs::write(&prompt_path, DEFAULT_SYSTEM_PROMPT)?;
        eprintln!("Created default system prompt: {}", prompt_path.display());
    }

    Ok(())
}

/// Read the system prompt from the configuration file.
/// This function assumes ensure_config() has been called first.
pub fn read_system_prompt() -> io::Result<String> {
    let prompt_path = get_system_prompt_path()?;
    fs::read_to_string(&prompt_path)
}

/// Validate that the provider is one of the supported providers.
fn validate_provider(provider: &str) -> Result<(), String> {
    if VALID_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(format!(
            "Invalid provider '{}'\nAvailable providers: {}",
            provider,
            VALID_PROVIDERS.join(", ")
        ))
    }
}

/// Execute a provider command with the given system prompt.
/// Returns the exit code from the provider process.
fn execute_provider(provider: &str, prompt: &str) -> io::Result<i32> {
    eprintln!("Using AI provider: {}", provider);

    let status = match provider {
        "droid" => Command::new("droid")
            .args(["exec", "--output-format", "stream-json", "--skip-permissions-unsafe"])
            .arg(prompt)
            .status()?,
        "codex" => Command::new("codex")
            .args(["exec", "--full-auto", "--json"])
            .arg(prompt)
            .status()?,
        "claude" => Command::new("claude")
            .args(["-p", "--output-format", "stream-json", "--dangerously-skip-permissions"])
            .arg(prompt)
            .status()?,
        "gemini" => Command::new("gemini")
            .args(["-p", "--output-format", "stream-json", "--yolo"])
            .arg(prompt)
            .status()?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown provider: {}", provider),
            ))
        }
    };

    Ok(status.code().unwrap_or(1))
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Always ensure config exists on startup
    if let Err(e) = ensure_config() {
        eprintln!("Warning: Failed to initialize configuration: {}", e);
    }

    match cli.command {
        Some(Commands::Version) => {
            println!("ralph {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(Commands::Once { provider }) => {
            // Validate provider
            if let Err(e) = validate_provider(&provider) {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }

            // Read system prompt
            let prompt = match read_system_prompt() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: Failed to read system prompt: {}", e);
                    return ExitCode::from(1);
                }
            };

            // Execute provider
            match execute_provider(&provider, &prompt) {
                Ok(code) => ExitCode::from(code as u8),
                Err(e) => {
                    eprintln!("Error: Failed to execute provider '{}': {}", provider, e);
                    ExitCode::from(1)
                }
            }
        }
        None => {
            // No subcommand provided, show help
            println!("ralph {} - A dispatcher for AI provider agents", env!("CARGO_PKG_VERSION"));
            println!();
            println!("Use 'ralph --help' for more information.");
            ExitCode::SUCCESS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_system_prompt_not_empty() {
        assert!(!DEFAULT_SYSTEM_PROMPT.is_empty());
        assert!(DEFAULT_SYSTEM_PROMPT.contains("bd"));
        assert!(DEFAULT_SYSTEM_PROMPT.contains("beads"));
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir().expect("Should get config dir");
        assert!(config_dir.ends_with(".Ralph"));
    }

    #[test]
    fn test_get_system_prompt_path() {
        let prompt_path = get_system_prompt_path().expect("Should get prompt path");
        assert!(prompt_path.ends_with("system-prompt.md"));
    }

    #[test]
    fn test_ensure_config_and_read() {
        // This test uses the actual home directory
        // ensure_config should not fail
        ensure_config().expect("ensure_config should succeed");

        // read_system_prompt should return content
        let content = read_system_prompt().expect("read_system_prompt should succeed");
        assert!(!content.is_empty());
    }

    #[test]
    fn test_validate_provider_valid() {
        assert!(validate_provider("droid").is_ok());
        assert!(validate_provider("codex").is_ok());
        assert!(validate_provider("claude").is_ok());
        assert!(validate_provider("gemini").is_ok());
    }

    #[test]
    fn test_validate_provider_invalid() {
        let result = validate_provider("invalid_provider");
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Invalid provider 'invalid_provider'"));
        assert!(err_msg.contains("Available providers: droid, codex, claude, gemini"));
    }

    #[test]
    fn test_validate_provider_empty() {
        let result = validate_provider("");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_providers_list() {
        assert_eq!(VALID_PROVIDERS.len(), 4);
        assert!(VALID_PROVIDERS.contains(&"droid"));
        assert!(VALID_PROVIDERS.contains(&"codex"));
        assert!(VALID_PROVIDERS.contains(&"claude"));
        assert!(VALID_PROVIDERS.contains(&"gemini"));
    }
}
