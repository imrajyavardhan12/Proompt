mod commands;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "proompt",
    version,
    about = "Proompt - Universal prompt enhancement tool"
)]
struct Cli {
    /// Prompt to enhance (text mode by default)
    prompt: Option<String>,

    /// Target platform (claude, openai, gemini, generic, midjourney, dalle, sd)
    #[arg(short, long)]
    platform: Option<String>,

    /// Enhance as image prompt
    #[arg(short = 'i', long = "image")]
    image: bool,

    /// Include SuperMemory context
    #[arg(short = 'm', long = "memory")]
    memory: bool,

    /// Use a viral template by ID
    #[arg(short = 't', long = "template")]
    template: Option<String>,

    /// Style hints for image prompts (comma-separated)
    #[arg(short = 's', long = "style")]
    style: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage templates
    Templates {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// Login for hosted mode
    Login,
    /// Logout
    Logout,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a config value
    Set {
        /// Config key (e.g., mode, default_platform, byok.provider)
        key: String,
        /// Config value
        value: String,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// List available templates
    List {
        /// Show only trending templates
        #[arg(long)]
        trending: bool,
    },
    /// Sync templates from remote repository
    Sync,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => commands::config::show()?,
            ConfigAction::Set { key, value } => commands::config::set(&key, &value)?,
        },
        Some(Commands::Templates { action }) => match action {
            TemplateAction::List { trending } => commands::template::list(trending)?,
            TemplateAction::Sync => commands::template::sync().await?,
        },
        Some(Commands::Login) => {
            output::info("Login flow for hosted mode (not yet implemented)");
        }
        Some(Commands::Logout) => {
            output::info("Logged out successfully");
        }
        None => {
            if let Some(template_id) = cli.template {
                commands::template::apply_from_cli(&template_id, cli.prompt.as_deref())?;
            } else if let Some(prompt) = cli.prompt {
                commands::enhance::run(
                    &prompt,
                    cli.platform.as_deref(),
                    cli.image,
                    cli.memory,
                    cli.style.as_deref(),
                )
                .await?;
            } else {
                // Check for piped input
                let stdin = commands::enhance::read_stdin()?;
                if let Some(prompt) = stdin {
                    commands::enhance::run(
                        &prompt,
                        cli.platform.as_deref(),
                        cli.image,
                        cli.memory,
                        cli.style.as_deref(),
                    )
                    .await?;
                } else {
                    output::banner();
                    output::usage_hint();
                }
            }
        }
    }

    Ok(())
}
