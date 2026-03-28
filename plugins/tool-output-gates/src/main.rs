use sc_hooks_sdk::runner::PluginRunner;
use tool_output_gates::ToolOutputGatesHandler;

fn main() {
    std::process::exit(PluginRunner::run_sync(&ToolOutputGatesHandler));
}

