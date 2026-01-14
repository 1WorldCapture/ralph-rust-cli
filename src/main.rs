use clap::Parser;
use std::fs;
use std::io;
use std::path::PathBuf;

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

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Display version information
    Version,
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

fn main() {
    let cli = Cli::parse();

    // Always ensure config exists on startup
    if let Err(e) = ensure_config() {
        eprintln!("Warning: Failed to initialize configuration: {}", e);
    }

    match cli.command {
        Some(Commands::Version) => {
            println!("ralph {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            // No subcommand provided, show help
            println!("ralph {} - A dispatcher for AI provider agents", env!("CARGO_PKG_VERSION"));
            println!();
            println!("Use 'ralph --help' for more information.");
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
}
