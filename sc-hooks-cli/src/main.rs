mod config;
mod dispatch;
mod errors;
mod resolution;
#[cfg(test)]
mod test_support;
mod timeout;

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
    fn mode(&self) -> sc_hooks_core::dispatch::DispatchMode {
        if self.async_mode {
            sc_hooks_core::dispatch::DispatchMode::Async
        } else {
            sc_hooks_core::dispatch::DispatchMode::Sync
        }
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
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("internal panic: {panic_info}");
    }));

    let outcome = std::panic::catch_unwind(run);
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            eprintln!("{err}");
            std::process::exit(err.exit_code());
        }
        Err(_) => {
            std::process::exit(sc_hooks_core::exit_codes::INTERNAL_ERROR);
        }
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => {
            let config = config::load_default_config()?;
            let payload = read_optional_payload_from_stdin()?;
            let mode = args.mode();

            let handlers = resolution::resolve_chain(
                &config,
                &args.hook,
                args.event.as_deref(),
                mode,
                payload.as_ref(),
            )?;

            if handlers.is_empty() {
                return Ok(());
            }

            match dispatch::execute_chain(
                &handlers,
                &config,
                &args.hook,
                args.event.as_deref(),
                mode,
                payload.as_ref(),
            )? {
                dispatch::DispatchOutcome::Proceed => Ok(()),
                dispatch::DispatchOutcome::Blocked { reason } => Err(CliError::Blocked { reason }),
            }?;
        }
        Commands::Audit => {
            return Err(CliError::AuditFailure {
                message: "not yet implemented".to_string(),
            });
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
            print!("{}", sc_hooks_core::exit_codes::render_reference());
        }
    }

    Ok(())
}

fn read_optional_payload_from_stdin() -> Result<Option<serde_json::Value>, CliError> {
    use std::io::Read;

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|err| CliError::internal(format!("failed reading stdin payload: {err}")))?;

    if input.trim().is_empty() {
        return Ok(None);
    }

    let parsed =
        serde_json::from_str::<serde_json::Value>(&input).map_err(|err| CliError::PluginError {
            message: format!("invalid JSON payload on stdin: {err}"),
        })?;

    Ok(Some(parsed))
}
