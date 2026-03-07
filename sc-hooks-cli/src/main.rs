mod config;
mod errors;
mod exit_codes;

use clap::{Args, Parser, Subcommand};

use crate::errors::CliError;

#[derive(Debug, Parser)]
#[command(name = "sc-hooks")]
#[command(about = "Universal hook dispatcher for AI-assisted development")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Normal execution (called by AI tool hooks)
    Run(RunArgs),

    /// Validate config + handlers + data flow
    Audit,

    /// Diagnostic trigger with synthetic/real payload
    Fire(FireArgs),

    /// Generate .claude/settings.json hook entries
    Install,

    /// Show resolved configuration
    Config,

    /// List available builtins and discovered plugins
    Handlers,

    /// Run compliance tests against a plugin
    Test(TestArgs),

    /// Show full exit code reference
    ExitCodes,
}

#[derive(Debug, Args)]
struct RunArgs {
    /// Hook type name
    hook: String,

    /// Optional event name
    event: Option<String>,

    /// Run only sync-mode handlers (default behavior)
    #[arg(long, conflicts_with = "async_mode")]
    sync: bool,

    /// Run only async-mode handlers
    #[arg(long = "async", conflicts_with = "sync")]
    async_mode: bool,
}

impl RunArgs {
    fn mode(&self) -> &'static str {
        if self.async_mode { "async" } else { "sync" }
    }
}

#[derive(Debug, Args)]
struct FireArgs {
    /// Hook type name
    hook: String,

    /// Optional event name
    event: Option<String>,
}

#[derive(Debug, Args)]
struct TestArgs {
    /// Plugin name
    plugin: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            let event = args.event.as_deref().unwrap_or("<none>");
            println!(
                "not yet implemented: run hook={} event={} mode={}",
                args.hook,
                event,
                args.mode()
            );
        }
        Commands::Audit => {
            println!("not yet implemented");
        }
        Commands::Fire(args) => {
            let event = args.event.as_deref().unwrap_or("<none>");
            println!(
                "not yet implemented: fire hook={} event={}",
                args.hook, event
            );
        }
        Commands::Install => {
            println!("not yet implemented");
        }
        Commands::Config => {
            let config = config::load_default_config()?;
            let rendered = config.to_pretty_toml()?;
            println!("{rendered}");
        }
        Commands::Handlers => {
            println!("not yet implemented");
        }
        Commands::Test(args) => {
            println!("not yet implemented: test plugin={}", args.plugin);
        }
        Commands::ExitCodes => {
            print!("{}", exit_codes::render_reference());
        }
    }

    Ok(())
}
